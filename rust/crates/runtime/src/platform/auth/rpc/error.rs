use serde_json::{json, Value};
use ulid::Ulid;

use super::super::AuthorizationStateError;

pub(super) fn public_rpc_error(_subject: &str, error: &AuthorizationStateError) -> Value {
    let (error_type, code, message) = match error {
        AuthorizationStateError::InvalidRecord(_) => {
            ("AuthError", "invalid_request", "The request is invalid.")
        }
        AuthorizationStateError::StorageConflict => (
            "AuthError",
            "conflict",
            "The request conflicts with current authentication state.",
        ),
        AuthorizationStateError::PrincipalMissing
        | AuthorizationStateError::SessionMissing
        | AuthorizationStateError::AuthorityMissing => (
            "AuthError",
            "not_found",
            "The requested authentication record was not found.",
        ),
        error if error.is_expected_denial() => (
            "AuthError",
            "not_authorized",
            "The request is not authorized.",
        ),
        _ => (
            "UnexpectedError",
            "internal_error",
            "The request could not be completed.",
        ),
    };
    if error_type == "AuthError" {
        json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "reason": code,
        })
    } else {
        json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "context": { "code": code },
        })
    }
}
