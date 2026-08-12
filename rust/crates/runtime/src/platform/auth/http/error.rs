use axum::http::header::CONTENT_TYPE;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

use super::super::AuthorizationStateError;

#[derive(Debug)]
pub(super) struct HttpError {
    pub(super) status: StatusCode,
    pub(super) code: &'static str,
}

impl HttpError {
    pub(super) fn bad_request(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }

    pub(super) fn unauthorized(code: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
        }
    }

    pub(super) fn forbidden(code: &'static str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
        }
    }

    pub(super) fn not_found(code: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
        }
    }

    pub(super) fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }

    pub(super) fn gone(code: &'static str) -> Self {
        Self {
            status: StatusCode::GONE,
            code,
        }
    }

    pub(super) fn bad_gateway(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code,
        }
    }

    pub(super) fn unavailable(code: &'static str) -> Self {
        Self {
            status: StatusCode::SERVICE_UNAVAILABLE,
            code,
        }
    }

    pub(super) fn internal(code: &'static str) -> Self {
        Self {
            status: StatusCode::INTERNAL_SERVER_ERROR,
            code,
        }
    }
}

impl From<AuthorizationStateError> for HttpError {
    fn from(error: AuthorizationStateError) -> Self {
        tracing::warn!(%error, "auth HTTP domain operation failed");
        match error {
            AuthorizationStateError::InvalidRecord(_) => Self::bad_request("invalid_request"),
            AuthorizationStateError::StorageConflict => Self::conflict("conflict"),
            AuthorizationStateError::PrincipalMissing
            | AuthorizationStateError::SessionMissing
            | AuthorizationStateError::AuthorityMissing => Self::not_found("not_found"),
            error if error.is_expected_denial() => Self::forbidden("not_authorized"),
            _ => Self::internal("internal_error"),
        }
    }
}

pub(super) fn map_issuance_error(error: AuthorizationStateError) -> HttpError {
    tracing::warn!(%error, "auth issuance denied");
    match error {
        AuthorizationStateError::AuthorityStale
        | AuthorizationStateError::MaterializationStale
        | AuthorizationStateError::ContextSnapshotChanged => {
            HttpError::unavailable("authorization_pending")
        }
        error if error.is_expected_denial() => HttpError::unauthorized("auth_required"),
        error => error.into(),
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        (
            self.status,
            [(CONTENT_TYPE, "application/json")],
            Json(json!({ "error": { "code": self.code } })),
        )
            .into_response()
    }
}
