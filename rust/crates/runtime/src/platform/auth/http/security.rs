use axum::body::Body;
use axum::http::header::{
    CONTENT_SECURITY_POLICY, COOKIE, ORIGIN, REFERRER_POLICY, X_CONTENT_TYPE_OPTIONS,
    X_FRAME_OPTIONS,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Request};
use axum::middleware::Next;
use axum::response::Response;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use serde_json::json;
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq;
use url::Url;

use super::super::ephemeral::AuthOAuthState;
use super::super::{LoginPortalRecord, LoginSettingsRecord};
use super::{digest_parts, HttpError};

pub(super) fn validate_redirect(
    redirect: &str,
    allowed_origins: &[String],
) -> Result<(), HttpError> {
    let redirect = Url::parse(redirect).map_err(|_| HttpError::bad_request("invalid_redirect"))?;
    if redirect.fragment().is_some() || redirect.username() != "" || redirect.password().is_some() {
        return Err(HttpError::bad_request("invalid_redirect"));
    }
    if !allowed_origins.contains(&canonical_origin(redirect.as_str())?) {
        return Err(HttpError::bad_request("redirect_origin_not_allowed"));
    }
    Ok(())
}

pub(super) fn canonical_origin(value: &str) -> Result<String, HttpError> {
    let url = Url::parse(value).map_err(|_| HttpError::bad_request("invalid_origin"))?;
    let host = url
        .host_str()
        .ok_or_else(|| HttpError::bad_request("invalid_origin"))?;
    let mut origin = format!("{}://{host}", url.scheme());
    if let Some(port) = url.port() {
        origin.push(':');
        origin.push_str(&port.to_string());
    }
    Ok(origin)
}

pub(super) fn oidc_portal_policy_digest(
    portal: &LoginPortalRecord,
    settings: &LoginSettingsRecord,
) -> Result<String, HttpError> {
    trellis_protocol::digest_json(&json!({
        "portalId": portal.portal_id,
        "portalVersion": portal.version,
        "disabled": portal.disabled,
        "removed": portal.removed,
        "providerIds": portal.provider_ids,
        "settingsVersion": settings.version,
        "federatedRegistrationEnabled": settings.federated_registration_enabled,
    }))
    .map_err(|_| HttpError::internal("oauth_policy_digest"))
}

pub(super) fn oauth_cookie_name(state_id: &str) -> String {
    format!("trellis_oauth_{}", digest_parts(&[state_id]))
}

pub(super) fn oauth_cookie_header(
    name: &str,
    value: &str,
    provider_id: &str,
    secure: bool,
    max_age_seconds: u64,
) -> Result<HeaderValue, HttpError> {
    let secure = if secure { "; Secure" } else { "" };
    HeaderValue::from_str(&format!(
        "{name}={value}; Path=/auth/callback/{provider_id}; Max-Age={max_age_seconds}; HttpOnly; SameSite=Lax{secure}"
    ))
    .map_err(|_| HttpError::internal("oauth_cookie"))
}

fn request_cookie(headers: &HeaderMap, name: &str) -> Option<String> {
    headers
        .get(COOKIE)?
        .to_str()
        .ok()?
        .split(';')
        .map(str::trim)
        .find_map(|cookie| {
            let (cookie_name, value) = cookie.split_once('=')?;
            (cookie_name == name).then(|| value.to_owned())
        })
}

pub(super) fn require_oauth_browser_binding(
    state_id: &str,
    state: &AuthOAuthState,
    headers: &HeaderMap,
) -> Result<(), HttpError> {
    let browser_binding = request_cookie(headers, &oauth_cookie_name(state_id))
        .ok_or_else(|| HttpError::bad_request("oauth_browser_binding_invalid"))?;
    let actual = URL_SAFE_NO_PAD.encode(Sha256::digest(browser_binding.as_bytes()));
    if bool::from(
        actual
            .as_bytes()
            .ct_eq(state.browser_binding_digest.as_bytes()),
    ) {
        Ok(())
    } else {
        Err(HttpError::bad_request("oauth_browser_binding_invalid"))
    }
}

pub(super) fn require_portal_origin(
    headers: &HeaderMap,
    public_origin: &str,
) -> Result<(), HttpError> {
    let Some(origin) = headers.get(ORIGIN) else {
        return Err(HttpError::forbidden("origin_required"));
    };
    if origin
        .to_str()
        .ok()
        .and_then(|origin| canonical_origin(origin).ok())
        .as_deref()
        != Some(canonical_origin(public_origin)?.as_str())
    {
        return Err(HttpError::forbidden("origin_mismatch"));
    }
    Ok(())
}

pub(super) fn require_selected_portal_origin(
    headers: &HeaderMap,
    portal: &LoginPortalRecord,
    public_origin: &str,
) -> Result<(), HttpError> {
    require_portal_origin(
        headers,
        portal.entry_url.as_deref().unwrap_or(public_origin),
    )
}

pub(super) async fn security_headers(request: Request<Body>, next: Next) -> Response {
    let mut response = next.run(request).await;
    let headers = response.headers_mut();
    headers.insert(X_CONTENT_TYPE_OPTIONS, HeaderValue::from_static("nosniff"));
    headers.insert(REFERRER_POLICY, HeaderValue::from_static("no-referrer"));
    headers.insert(X_FRAME_OPTIONS, HeaderValue::from_static("DENY"));
    headers.insert(
        HeaderName::from_static("permissions-policy"),
        HeaderValue::from_static("camera=(), microphone=(), geolocation=()"),
    );
    headers.insert(
        CONTENT_SECURITY_POLICY,
        HeaderValue::from_static(
            "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src 'self'",
        ),
    );
    response
}
