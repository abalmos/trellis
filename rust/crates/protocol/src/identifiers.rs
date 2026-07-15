use std::cmp::Ordering;

use crate::ProtocolError;

pub(crate) fn compare_protocol_strings(left: &str, right: &str) -> Ordering {
    left.encode_utf16().cmp(right.encode_utf16())
}

pub(crate) fn sort_deduplicate(values: &mut Vec<String>) {
    values.sort_by(|left, right| compare_protocol_strings(left, right));
    values.dedup();
}

pub(crate) fn validate_protocol_identifier(path: &str, value: &str) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Err(api_error(path, "must not be empty"));
    }
    if value.trim() != value {
        return Err(api_error(
            path,
            "must not have leading or trailing whitespace",
        ));
    }
    if value.chars().any(|character| character.is_ascii_control()) {
        return Err(api_error(path, "must not contain ASCII control characters"));
    }
    Ok(())
}

pub(crate) fn validate_nonempty_text(path: &str, value: &str) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Err(api_error(path, "must not be empty"));
    }
    Ok(())
}

pub(crate) fn validate_api_id(path: &str, value: &str) -> Result<(), ProtocolError> {
    validate_protocol_identifier(path, value)?;
    let Some((lineage, major)) = value.rsplit_once("@v") else {
        return Err(api_error(path, "must end in one '@vN' major suffix"));
    };
    if lineage.is_empty() || lineage.contains("@v") {
        return Err(api_error(
            path,
            "must contain one non-empty lineage before '@vN'",
        ));
    }
    validate_positive_decimal(path, major)
}

pub(crate) fn validate_version(path: &str, value: &str) -> Result<(), ProtocolError> {
    let Some(major) = value.strip_prefix('v') else {
        return Err(api_error(path, "must be a positive 'vN' version"));
    };
    validate_positive_decimal(path, major)
}

pub(crate) fn validate_logical_name(path: &str, value: &str) -> Result<(), ProtocolError> {
    validate_protocol_identifier(path, value)?;
    if value.starts_with('.')
        || value.ends_with('.')
        || value.contains("..")
        || value.split('.').any(|token| {
            token.is_empty() || token.contains(['*', '>']) || token.chars().any(char::is_whitespace)
        })
    {
        return Err(api_error(
            path,
            "must be dot-separated non-empty NATS-safe tokens",
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

fn validate_positive_decimal(path: &str, value: &str) -> Result<(), ProtocolError> {
    let bytes = value.as_bytes();
    if bytes.is_empty()
        || !matches!(bytes[0], b'1'..=b'9')
        || !bytes[1..].iter().all(u8::is_ascii_digit)
    {
        return Err(api_error(path, "must use a positive decimal major version"));
    }
    Ok(())
}
