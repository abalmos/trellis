use std::cmp::Ordering;

use crate::ProtocolError;

pub(crate) type ValidationErrorFactory = fn(String, String) -> ProtocolError;

pub(crate) fn compare_protocol_strings(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

pub(crate) fn sort_deduplicate(values: &mut Vec<String>) {
    values.sort_by(|left, right| compare_protocol_strings(left, right));
    values.dedup();
}

pub(crate) fn validate_protocol_identifier(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Err(error(path.to_owned(), "must not be empty".to_owned()));
    }
    if value.trim() != value {
        return Err(error(
            path.to_owned(),
            "must not have leading or trailing whitespace".to_owned(),
        ));
    }
    if value.chars().any(|character| character.is_ascii_control()) {
        return Err(error(
            path.to_owned(),
            "must not contain ASCII control characters".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_nonempty_text(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Err(error(path.to_owned(), "must not be empty".to_owned()));
    }
    Ok(())
}

pub(crate) fn validate_api_id_at(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    validate_protocol_identifier(path, value, error)?;
    if value.len() > 128 {
        return Err(error(
            path.to_owned(),
            "must be at most 128 bytes".to_owned(),
        ));
    }
    let Some((lineage, major)) = value.rsplit_once("@v") else {
        return Err(error(
            path.to_owned(),
            "must end in one '@vN' major suffix".to_owned(),
        ));
    };
    if lineage.is_empty()
        || !lineage.as_bytes().iter().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'_' | b'-')
        })
        || !lineage.as_bytes()[0].is_ascii_lowercase() && !lineage.as_bytes()[0].is_ascii_digit()
        || !lineage.as_bytes()[lineage.len() - 1].is_ascii_lowercase()
            && !lineage.as_bytes()[lineage.len() - 1].is_ascii_digit()
        || lineage.as_bytes().windows(2).any(|pair| {
            matches!(pair[0], b'.' | b'_' | b'-') && matches!(pair[1], b'.' | b'_' | b'-')
        })
    {
        return Err(error(
            path.to_owned(),
            "must use lowercase alphanumeric lineage tokens separated by '.', '_', or '-' before '@vN'".to_owned(),
        ));
    }
    validate_positive_decimal(path, major, error)
}

/// Validate one stable `lineage@vN` API identifier.
///
/// # Errors
///
/// Returns [`ProtocolError::ApiValidation`] when `value` is not a valid API ID.
pub fn validate_api_id(value: &str) -> Result<(), ProtocolError> {
    validate_api_id_at("/id", value, api_error)
}

pub(crate) fn validate_version(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    let Some(major) = value.strip_prefix('v') else {
        return Err(error(
            path.to_owned(),
            "must be a positive 'vN' version".to_owned(),
        ));
    };
    validate_positive_decimal(path, major, error)
}

pub(crate) fn validate_logical_name(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    validate_protocol_identifier(path, value, error)?;
    if value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
        || value.split('.').any(|token| {
            token.is_empty() || token.contains(['*', '>']) || token.chars().any(char::is_whitespace)
        })
    {
        return Err(error(
            path.to_owned(),
            "must be dot-separated non-empty NATS-safe tokens".to_owned(),
        ));
    }
    Ok(())
}

pub(crate) fn api_error(path: impl Into<String>, message: impl Into<String>) -> ProtocolError {
    ProtocolError::ApiValidation {
        path: path.into(),
        message: message.into(),
    }
}

pub(crate) fn participant_error(
    path: impl Into<String>,
    message: impl Into<String>,
) -> ProtocolError {
    ProtocolError::ParticipantValidation {
        path: path.into(),
        message: message.into(),
    }
}

fn validate_positive_decimal(
    path: &str,
    value: &str,
    error: ValidationErrorFactory,
) -> Result<(), ProtocolError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || !matches!(bytes[0], b'1'..=b'9')
        || !bytes[1..].iter().all(u8::is_ascii_digit)
    {
        return Err(error(
            path.to_owned(),
            "must use a positive decimal major version".to_owned(),
        ));
    }
    Ok(())
}
