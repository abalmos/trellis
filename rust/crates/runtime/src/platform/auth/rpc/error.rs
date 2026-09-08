use serde_json::{json, Value};
use ulid::Ulid;

use super::super::AuthorizationStateError;

pub(super) fn public_rpc_error(_subject: &str, error: &AuthorizationStateError) -> Value {
    let (error_type, code, message) = match error {
        AuthorizationStateError::WrongPrincipalKind => (
            "AuthError",
            "wrong_principal_kind",
            "This operation requires a user login.",
        ),
        AuthorizationStateError::CurrentIssuerConflict => (
            "AuthError",
            "issuer_current",
            "Select a replacement signing issuer before revoking this key.",
        ),
        AuthorizationStateError::RevisionConflict { .. } => (
            "AuthError",
            "revision_conflict",
            "The current revision differs from expectedRevision.",
        ),
        AuthorizationStateError::InvalidRecord(_) => {
            ("AuthError", "invalid_request", "The request is invalid.")
        }
        AuthorizationStateError::PortalPolicyChanged | AuthorizationStateError::StorageConflict => {
            (
                "AuthError",
                "conflict",
                "The request conflicts with current authentication state.",
            )
        }
        AuthorizationStateError::IdentityMissing => (
            "AuthError",
            "identity_not_found",
            "The requested identity was not found.",
        ),
        AuthorizationStateError::PrincipalMissing
        | AuthorizationStateError::ParticipantMissing
        | AuthorizationStateError::NotFound
        | AuthorizationStateError::IssuerMissing
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_and_principal_kind_denials_are_not_malformed_requests() {
        for (error, reason) in [
            (AuthorizationStateError::NotAuthorized, "not_authorized"),
            (
                AuthorizationStateError::WrongPrincipalKind,
                "wrong_principal_kind",
            ),
            (
                AuthorizationStateError::InvalidRecord("malformed".into()),
                "invalid_request",
            ),
        ] {
            let response = public_rpc_error("rpc.v1.Auth.Sessions.Logout", &error);
            assert_eq!(response["type"], "AuthError");
            assert_eq!(response["reason"], reason);
        }
    }
}
