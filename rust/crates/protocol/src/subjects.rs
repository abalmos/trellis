use std::collections::BTreeMap;

use serde::Serialize;

use crate::{
    identifiers::{api_error, validate_logical_name, validate_version},
    ProtocolError,
};

/// Subjects derived for every communication surface in one API artifact.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedApiSubjectsV1 {
    /// RPC subjects keyed by logical name.
    pub rpc: BTreeMap<String, String>,
    /// Operation subjects keyed by logical name.
    pub operations: BTreeMap<String, String>,
    /// Event base and wildcard subjects keyed by logical name.
    pub events: BTreeMap<String, DerivedEventSubjectsV1>,
    /// Feed subjects keyed by logical name.
    pub feeds: BTreeMap<String, String>,
}

/// Base and wildcard subscription subjects for one event.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DerivedEventSubjectsV1 {
    /// Event subject before parameter tokens are appended.
    pub base: String,
    /// Subscription subject with one wildcard per event parameter.
    pub wildcard: String,
}

/// Derive an RPC subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_rpc_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("rpc", version, logical_name)
}

/// Derive an operation subject from its version and logical name.
///
/// # Errors
///
/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or
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
/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_event_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("events", version, logical_name)
}

/// Derive an event subscription subject with one wildcard per parameter.
///
/// Parameter order is defined by the event artifact; this function appends one
/// wildcard token for each parameter without reordering it.
///
/// # Errors
///
/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or
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
/// Returns [`ProtocolError::ApiValidation`] for an invalid `vN` version or
/// logical surface name.
pub fn derive_feed_subject(version: &str, logical_name: &str) -> Result<String, ProtocolError> {
    derive_subject("feed", version, logical_name)
}

fn derive_subject(
    family: &str,
    version: &str,
    logical_name: &str,
) -> Result<String, ProtocolError> {
    validate_version("/version", version, api_error)?;
    validate_logical_name("/name", logical_name, api_error)?;
    Ok(format!("{family}.{version}.{logical_name}"))
}

#[cfg(test)]
mod tests {
    use super::derive_rpc_subject;

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
}
