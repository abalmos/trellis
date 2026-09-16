use serde_json::{json, Value};
use ulid::Ulid;

use super::super::AuthorizationStateError;

pub(super) fn public_rpc_error(subject: &str, error: &AuthorizationStateError) -> Value {
    if subject.starts_with("rpc.v1.core.Resources.") {
        let (error_type, code, message) = match error {
            AuthorizationStateError::RevisionConflict { .. } => (
                "trellis.core@v1::ValidationError",
                "revision_conflict",
                "The current revision differs from expectedRevision.",
            ),
            AuthorizationStateError::NotFound => (
                "trellis.core@v1::ValidationError",
                "not_found",
                "The requested resource was not found.",
            ),
            AuthorizationStateError::InvalidRecord(_)
            | AuthorizationStateError::StorageConflict => (
                "trellis.core@v1::ValidationError",
                "invalid_request",
                "The resource request is invalid.",
            ),
            error if error.is_expected_denial() => (
                "trellis.core@v1::ValidationError",
                "not_authorized",
                "The request is not authorized.",
            ),
            _ => (
                "trellis.core@v1::UnexpectedError",
                "internal_error",
                "The request could not be completed.",
            ),
        };
        return json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "context": { "code": code },
        });
    }
    let (error_type, code, message) = match error {
        AuthorizationStateError::ApprovalRequired { .. } => (
            "trellis.auth@v1::AuthError",
            "approval_required",
            "Approval of the current deployment consent request is required.",
        ),
        AuthorizationStateError::WrongPrincipalKind => (
            "trellis.auth@v1::AuthError",
            "wrong_principal_kind",
            "This operation requires a user login.",
        ),
        AuthorizationStateError::CurrentIssuerConflict => (
            "trellis.auth@v1::AuthError",
            "issuer_current",
            "Select a replacement signing issuer before revoking this key.",
        ),
        AuthorizationStateError::RevisionConflict { .. } => (
            "trellis.auth@v1::AuthError",
            "revision_conflict",
            "The current revision differs from expectedRevision.",
        ),
        AuthorizationStateError::InvalidRecord(_) => (
            "trellis.auth@v1::AuthError",
            "invalid_request",
            "The request is invalid.",
        ),
        AuthorizationStateError::PortalPolicyChanged | AuthorizationStateError::StorageConflict => {
            (
                "trellis.auth@v1::AuthError",
                "conflict",
                "The request conflicts with current authentication state.",
            )
        }
        AuthorizationStateError::IdentityMissing => (
            "trellis.auth@v1::AuthError",
            "identity_not_found",
            "The requested identity was not found.",
        ),
        AuthorizationStateError::PrincipalMissing
        | AuthorizationStateError::ParticipantMissing
        | AuthorizationStateError::NotFound
        | AuthorizationStateError::IssuerMissing
        | AuthorizationStateError::SessionMissing
        | AuthorizationStateError::AuthorityMissing => (
            "trellis.auth@v1::AuthError",
            "not_found",
            "The requested authentication record was not found.",
        ),
        error if error.is_expected_denial() => (
            "trellis.auth@v1::AuthError",
            "not_authorized",
            "The request is not authorized.",
        ),
        _ => (
            "trellis.auth@v1::UnexpectedError",
            "internal_error",
            "The request could not be completed.",
        ),
    };
    if error_type.ends_with("::AuthError") || error_type == "AuthError" {
        let mut response = json!({
            "id": format!("err_{}", Ulid::new()),
            "type": error_type,
            "message": message,
            "reason": code,
            "code": code,
            "field": null,
            "retryable": false,
        });
        if let AuthorizationStateError::ApprovalRequired { consent_request } = error {
            response["consentRequest"] = consent_request.clone();
        }
        response
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
            let response = public_rpc_error("rpc.v1.auth.Sessions.Logout", &error);
            assert_eq!(response["type"], "trellis.auth@v1::AuthError");
            assert_eq!(response["reason"], reason);
        }
    }

    #[test]
    fn deployment_approval_error_preserves_the_server_consent_request() {
        let consent_request = json!({
            "participantId": "acme.service@v1",
            "installedRevision": "2",
            "expectedGrantRevision": "3",
            "decisionDigest": "digest"
        });
        let response = public_rpc_error(
            "rpc.v1.auth.Deployments.Apply",
            &AuthorizationStateError::ApprovalRequired {
                consent_request: consent_request.clone(),
            },
        );
        assert_eq!(response["type"], "trellis.auth@v1::AuthError");
        assert_eq!(response["code"], "approval_required");
        assert_eq!(response["consentRequest"], consent_request);
    }
}
