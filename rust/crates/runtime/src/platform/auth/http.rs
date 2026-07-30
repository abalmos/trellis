use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use axum::body::Body;
use axum::extract::Query;
use axum::extract::{Path, State};
use axum::http::header::{
    CACHE_CONTROL, CONTENT_SECURITY_POLICY, CONTENT_TYPE, COOKIE, ORIGIN, REFERRER_POLICY,
    SET_COOKIE, STRICT_TRANSPORT_SECURITY, X_CONTENT_TYPE_OPTIONS, X_FRAME_OPTIONS,
};
use axum::http::{HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{IntoResponse, Redirect, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use nats_jwt_rs::types::{Permission, Permissions};
use nats_jwt_rs::user::User;
use nats_jwt_rs::Claims;
use nkeys::{KeyPair, KeyPairType};
use openidconnect::core::{CoreAuthenticationFlow, CoreClient, CoreProviderMetadata};
use openidconnect::{
    AccessTokenHash, AuthorizationCode, ClientId, ClientSecret, CsrfToken, IssuerUrl, Nonce,
    OAuth2TokenResponse, PkceCodeChallenge, PkceCodeVerifier, RedirectUrl, Scope, TokenResponse,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest as _, Sha256};
use subtle::ConstantTimeEq;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use trellis_protocol::{
    canonicalize_json, parse_api_v1, parse_participant_v1, parse_session_proof_v1,
    resolve_participant_v1, session_proof_request_digest_v1, verify_session_proof_v1,
    AuthorizationPrincipalKindV1, GrantSetV1, ParticipantKindV1, SessionProofInputV1,
    SessionProofPolicyV1,
};
use trellis_rs::service::{
    EventConsumerOrdering, EventConsumerReplay, EventConsumerResourceBinding,
    JobsQueueResourceBinding, JobsResourceBinding, JobsSchemaRef, KvResourceBinding,
    ServiceResourceBindings, StoreResourceBinding,
};

const EMBEDDED_PORTAL_ASSETS: &[(&str, &[u8])] =
    include!(concat!(env!("OUT_DIR"), "/portal_assets.rs"));
const MAX_AUTH_REQUEST_BODY_BYTES: usize = 4 * 1024 * 1024;
use url::Url;

use super::ephemeral::{
    claim_oauth_state, AuthBrowserFlow, AuthBrowserFlowKind, AuthBrowserFlowState,
    AuthEphemeralRepository, AuthOAuthKind, AuthOAuthState, AuthOAuthStatus,
    BrowserConsentProposal, BROWSER_FLOW_FORMAT,
};
use super::{
    AccountFlowRepository, AccountFlowState, AccountRepository, AuthService, AuthSessionRepository,
    AuthorityDecision, AuthorityDecisionOutcome, AuthorityEvidenceScope, AuthorityKind,
    AuthorityProposalKind, AuthorityProposalRecord, AuthorityProposalRepository,
    AuthorityProposalState, AuthorityState, AuthorityTarget,
    AuthorizationMaterializationRepository, AuthorizationStateError, ClientBootstrapAdmission,
    CompleteIdentityLinkInput, CreateActivationReviewInput, CreateAuthorityProposalInput,
    CreateFederatedUserInput, CreateLocalUserInput, CreateSessionInput,
    DecideAuthorityProposalInput, DeploymentAuthorityRepository, DesiredAuthorityRecord,
    DeviceActivationReviewState, DeviceState, EnrollDeviceIdentityInput, EvidenceRepository,
    FirstAdminFederatedRegistration, FirstAdminRegistration, IdempotencyRepository,
    IdempotencyResultRecord, IdempotentOutcome, IdentityAuthorityRecord,
    IdentityAuthorityRepository, LocalAuthentication, LoginPortalRecord, LoginPortalRepository,
    LoginSettingsRecord, ParticipantBindingRecord, ParticipantBindingRepository,
    ParticipantBindingState, PostCommitActionKind, PostCommitActionRecord,
    PresentDeploymentAuthorityInput, PrincipalKind, PrincipalRepository,
    ProviderIdentityRepository, ProvisionedIdentityKind, ProvisionedIdentityState,
    ProvisioningRepository, ResourceBindingEvidence, ResourceProviderIdentity,
    RuntimeInstanceState, SessionRecord, SessionRepository, UserProfileRecord,
};

const FLOW_TTL_MS: i64 = 15 * 60_000;
const IDEMPOTENCY_TTL_MS: i64 = 24 * 60 * 60_000;

#[derive(Clone)]
pub(crate) struct OidcProvider {
    metadata: CoreProviderMetadata,
    client_id: ClientId,
    client_secret: Option<ClientSecret>,
    redirect_uri: RedirectUrl,
    scopes: Vec<Scope>,
}

pub(crate) async fn discover_oidc_providers(
    config: Option<&crate::config::OAuthConfig>,
    public_origin: &str,
) -> Result<BTreeMap<String, OidcProvider>, AuthorizationStateError> {
    let Some(config) = config else {
        return Ok(BTreeMap::new());
    };
    let redirect_base = config.redirect_base.as_deref().unwrap_or(public_origin);
    let http_client = openidconnect::reqwest::ClientBuilder::new()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .map_err(|error| {
            AuthorizationStateError::Storage(format!("failed to build OIDC HTTP client: {error}"))
        })?;
    let mut providers = BTreeMap::new();
    for (provider_id, provider) in &config.providers {
        if provider.provider_type != "oidc" {
            return Err(AuthorizationStateError::InvalidRecord(format!(
                "OAuth provider {provider_id} type must be oidc"
            )));
        }
        let issuer = provider.issuer.clone().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(format!(
                "OAuth provider {provider_id} has no issuer"
            ))
        })?;
        let metadata = CoreProviderMetadata::discover_async(
            IssuerUrl::new(issuer).map_err(|_| {
                AuthorizationStateError::InvalidRecord(format!(
                    "OAuth provider {provider_id} issuer is invalid"
                ))
            })?,
            &http_client,
        )
        .await
        .map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "OIDC discovery failed for {provider_id}: {error}"
            ))
        })?;
        let client_id = ClientId::new(provider.client_id.clone().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(format!(
                "OAuth provider {provider_id} has no client_id"
            ))
        })?);
        let secret = match (&provider.client_secret, &provider.client_secret_file) {
            (Some(secret), None) => Some(secret.clone()),
            (None, Some(path)) => Some(fs::read_to_string(path).map_err(|error| {
                AuthorizationStateError::Storage(format!(
                    "failed to read OAuth provider {provider_id} secret: {error}"
                ))
            })?),
            (None, None) => None,
            (Some(_), Some(_)) => {
                return Err(AuthorizationStateError::InvalidRecord(format!(
                    "OAuth provider {provider_id} has two client secrets"
                )))
            }
        };
        let redirect_uri = RedirectUrl::new(format!(
            "{}/auth/callback/{provider_id}",
            redirect_base.trim_end_matches('/')
        ))
        .map_err(|_| {
            AuthorizationStateError::InvalidRecord(format!(
                "OAuth provider {provider_id} redirect URI is invalid"
            ))
        })?;
        providers.insert(
            provider_id.clone(),
            OidcProvider {
                metadata,
                client_id,
                client_secret: secret.map(|secret| ClientSecret::new(secret.trim().to_owned())),
                redirect_uri,
                scopes: provider
                    .scopes
                    .clone()
                    .unwrap_or_else(|| {
                        vec![
                            "openid".to_owned(),
                            "profile".to_owned(),
                            "email".to_owned(),
                        ]
                    })
                    .into_iter()
                    .map(Scope::new)
                    .collect(),
            },
        );
    }
    Ok(providers)
}

pub(crate) trait AuthHttpRepository:
    AccountRepository
    + AccountFlowRepository
    + AuthSessionRepository
    + AuthorityProposalRepository
    + AuthorizationMaterializationRepository
    + DeploymentAuthorityRepository
    + EvidenceRepository
    + IdentityAuthorityRepository
    + IdempotencyRepository
    + LoginPortalRepository
    + ParticipantBindingRepository
    + PrincipalRepository
    + ProviderIdentityRepository
    + ProvisioningRepository
    + SessionRepository
    + Clone
    + Send
    + Sync
    + 'static
{
}

impl<T> AuthHttpRepository for T where
    T: AccountRepository
        + AccountFlowRepository
        + AuthSessionRepository
        + AuthorityProposalRepository
        + AuthorizationMaterializationRepository
        + DeploymentAuthorityRepository
        + EvidenceRepository
        + IdentityAuthorityRepository
        + IdempotencyRepository
        + LoginPortalRepository
        + ParticipantBindingRepository
        + PrincipalRepository
        + ProviderIdentityRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static
{
}

#[derive(Clone)]
pub(super) struct AuthHttpState<R, E> {
    service: AuthService<R>,
    ephemeral: E,
    issuer: NatsBootstrapIssuer,
    authorization_contexts: super::AuthorizationContextService,
    public_origin: String,
    allowed_redirect_origins: Vec<String>,
    websocket_nats_servers: Vec<String>,
    oidc_providers: BTreeMap<String, OidcProvider>,
    proof_policy: SessionProofPolicyV1,
    portal_override_dir: Option<PathBuf>,
}

pub(crate) struct AuthHttpOptions<R, E> {
    pub service: AuthService<R>,
    pub ephemeral: E,
    pub issuer: NatsBootstrapIssuer,
    pub authorization_contexts: super::AuthorizationContextService,
    pub public_origin: String,
    pub allowed_origins: Vec<String>,
    pub websocket_nats_servers: Vec<String>,
    pub oidc_providers: BTreeMap<String, OidcProvider>,
    pub rate_limit_max: u32,
    pub rate_limit_window_ms: u64,
    pub portal_override_dir: Option<PathBuf>,
}

#[derive(Clone)]
pub(crate) struct NatsBootstrapIssuer {
    signing_key: Arc<KeyPair>,
    auth_account: String,
    maximum_lifetime_seconds: i64,
}

impl NatsBootstrapIssuer {
    pub(crate) fn from_files(
        signing_seed_file: &std::path::Path,
        auth_user_creds_file: &std::path::Path,
        maximum_lifetime_seconds: u64,
    ) -> Result<Self, AuthorizationStateError> {
        let seed = fs::read_to_string(signing_seed_file).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to read auth issuer signing seed: {error}"
            ))
        })?;
        let signing_key = KeyPair::from_seed(seed.trim()).map_err(|error| {
            AuthorizationStateError::Storage(format!("invalid auth issuer signing seed: {error}"))
        })?;
        if signing_key.key_pair_type() != KeyPairType::Account {
            return Err(AuthorizationStateError::InvalidRecord(
                "auth issuer signing seed must be an account NKey".to_owned(),
            ));
        }
        let credentials = fs::read_to_string(auth_user_creds_file).map_err(|error| {
            AuthorizationStateError::Storage(format!("failed to read auth user creds: {error}"))
        })?;
        let jwt = credentials
            .lines()
            .skip_while(|line| *line != "-----BEGIN NATS USER JWT-----")
            .skip(1)
            .find(|line| !line.trim().is_empty())
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "auth user creds contain no user JWT".to_owned(),
                )
            })?;
        let claims = Claims::<User>::decode(jwt).map_err(|error| {
            AuthorizationStateError::InvalidRecord(format!("auth user JWT is invalid: {error}"))
        })?;
        let auth_account = claims
            .payload()
            .issuer_account
            .clone()
            .unwrap_or_else(|| claims.iss.clone());
        Ok(Self {
            signing_key: Arc::new(signing_key),
            auth_account,
            maximum_lifetime_seconds: i64::try_from(maximum_lifetime_seconds).map_err(|_| {
                AuthorizationStateError::InvalidRecord(
                    "maximum bootstrap JWT lifetime is too large".to_owned(),
                )
            })?,
        })
    }

    fn deny_all_user_jwt(
        &self,
        session_nkey: &str,
        expires_at_seconds: i64,
        now_seconds: i64,
    ) -> Result<IssuedBootstrapJwt, AuthorizationStateError> {
        let (kind, _) = nkeys::from_public_key(session_nkey).map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "sessionNkey is not a canonical NATS public key".to_owned(),
            )
        })?;
        if KeyPairType::from(kind) != KeyPairType::User {
            return Err(AuthorizationStateError::InvalidRecord(
                "sessionNkey must be a NATS User NKey".to_owned(),
            ));
        }
        let deny = Permission {
            allow: Vec::new(),
            deny: vec![">".to_owned()],
        };
        let mut claims = User::new_claims("trellis-session".to_owned(), session_nkey.to_owned());
        let expires_at = expires_at_seconds.min(
            now_seconds
                .checked_add(self.maximum_lifetime_seconds)
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(
                        "bootstrap JWT expiry overflows".to_owned(),
                    )
                })?,
        );
        claims.exp = Some(expires_at);
        let user = claims.payload_mut();
        user.issuer_account = Some(self.auth_account.clone());
        user.permissions.permissions = Permissions {
            publish: deny.clone(),
            subscribe: deny,
            resp: None,
        };
        let jwt = claims.encode(&self.signing_key).map_err(|error| {
            AuthorizationStateError::Storage(format!(
                "failed to sign session bootstrap JWT: {error}"
            ))
        })?;
        Ok(IssuedBootstrapJwt { jwt, expires_at })
    }
}

struct IssuedBootstrapJwt {
    jwt: String,
    expires_at: i64,
}

pub(crate) fn router<R, E>(
    options: AuthHttpOptions<R, E>,
) -> Result<Router, AuthorizationStateError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone + Send + Sync + 'static,
{
    let public_url = Url::parse(&options.public_origin).map_err(|_| {
        AuthorizationStateError::InvalidRecord("HTTP public origin is invalid".to_owned())
    })?;
    let use_hsts = public_url.scheme() == "https";
    let allowed_origins = if options.allowed_origins.is_empty() {
        vec![options.public_origin.clone()]
    } else {
        options.allowed_origins
    };
    let allowed_redirect_origins = allowed_origins
        .iter()
        .filter(|origin| origin.as_str() != "*")
        .chain(std::iter::once(&options.public_origin))
        .map(|origin| {
            canonical_origin(origin).map_err(|_| {
                AuthorizationStateError::InvalidRecord(format!("HTTP origin is invalid: {origin}"))
            })
        })
        .collect::<Result<Vec<_>, _>>()?;
    let cors = if allowed_origins.iter().any(|origin| origin == "*") {
        CorsLayer::new()
            .allow_origin(AllowOrigin::any())
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(AllowHeaders::mirror_request())
    } else {
        let origins = allowed_origins
            .iter()
            .map(|origin| {
                HeaderValue::from_str(origin).map_err(|_| {
                    AuthorizationStateError::InvalidRecord(format!(
                        "HTTP origin is invalid: {origin}"
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()?;
        CorsLayer::new()
            .allow_origin(AllowOrigin::list(origins))
            .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
            .allow_headers(AllowHeaders::mirror_request())
            .allow_credentials(true)
    };
    let state = AuthHttpState {
        service: options.service,
        ephemeral: options.ephemeral,
        issuer: options.issuer,
        authorization_contexts: options.authorization_contexts,
        public_origin: options.public_origin,
        allowed_redirect_origins,
        websocket_nats_servers: options.websocket_nats_servers,
        oidc_providers: options.oidc_providers,
        proof_policy: SessionProofPolicyV1::default(),
        portal_override_dir: options.portal_override_dir,
    };
    let mut routes = Router::new()
        .route("/auth/requests", post(start_auth::<R, E>))
        .route("/bootstrap/service", post(service_bootstrap::<R, E>))
        .route("/bootstrap/device", post(device_bootstrap::<R, E>))
        .route(
            "/auth/devices/activate/wait",
            post(wait_for_device_activation::<R, E>),
        )
        .route("/bootstrap/client", post(client_bootstrap::<R, E>))
        .route("/auth/context/refresh", post(refresh_context::<R, E>))
        .route(
            "/.well-known/trellis/authorization/trust/:key",
            get(read_trust_registry::<R, E>),
        )
        .route(
            "/.well-known/trellis/authorization/contexts/:digest",
            get(read_context_registry::<R, E>),
        )
        .route(
            "/.well-known/trellis/authorization/revocations",
            get(read_revocation_snapshot::<R, E>),
        )
        .route("/auth/flow/:flow_id", get(get_flow::<R, E>))
        .route("/auth/login/local", post(local_login::<R, E>))
        .route("/auth/sessions/logout", post(logout_session::<R, E>))
        .route(
            "/auth/flow/:flow_id/register/local",
            post(register_local::<R, E>),
        )
        .route(
            "/auth/account-flow/:flow_token",
            get(get_account_flow::<R, E>),
        )
        .route(
            "/auth/account-flow/:flow_token/local-password",
            post(complete_first_admin::<R, E>),
        )
        .route("/auth/login/:provider_id", get(start_oidc::<R, E>))
        .route(
            "/auth/account-flow/:flow_token/login/:provider_id",
            get(start_account_flow_oidc::<R, E>),
        )
        .route("/auth/callback/:provider_id", get(oidc_callback::<R, E>))
        .route(
            "/auth/flow/:flow_id/approval",
            post(decide_approval::<R, E>),
        )
        .route("/auth/flow/:flow_id/bind", post(bind_flow::<R, E>))
        .route("/_trellis/portal", get(portal_index::<R, E>))
        .route("/_trellis/portal/*path", get(portal_page::<R, E>))
        .route("/_trellis/assets/*path", get(portal_asset::<R, E>))
        .layer(RequestBodyLimitLayer::new(MAX_AUTH_REQUEST_BODY_BYTES))
        .layer(cors)
        .layer(middleware::from_fn(security_headers))
        .with_state(state);
    if use_hsts {
        routes = routes.layer(SetResponseHeaderLayer::if_not_present(
            STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }
    if options.rate_limit_max > 0 {
        let mut builder = GovernorConfigBuilder::default();
        builder
            .period(Duration::from_millis(
                (options.rate_limit_window_ms / u64::from(options.rate_limit_max)).max(1),
            ))
            .burst_size(options.rate_limit_max);
        let config = builder.finish().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("HTTP rate limit is invalid".to_owned())
        })?;
        routes = routes.layer(GovernorLayer {
            config: Arc::new(config),
        });
    }
    Ok(routes)
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct AuthStartRequest {
    request_id: String,
    issued_at: i64,
    session_public_key: String,
    session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Option<Value>,
    #[serde(default)]
    referenced_api_artifacts: Vec<Value>,
    redirect_target: String,
    proof: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AuthStartResponse {
    state: &'static str,
    flow_id: String,
    portal_url: String,
}

async fn start_auth<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<AuthStartResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let request: AuthStartRequest = serde_json::from_value(raw.clone()).map_err(|error| {
        tracing::warn!(%error, "invalid auth request shape");
        HttpError::bad_request("invalid_auth_request")
    })?;
    validate_redirect(&request.redirect_target, &state.allowed_redirect_origins)?;
    let request_digest = proof_request_digest(&raw).map_err(|error| {
        tracing::warn!(%error, "invalid auth request proof envelope");
        HttpError::bad_request("invalid_auth_request")
    })?;
    let input = SessionProofInputV1::user_auth_request(
        request.request_id.clone(),
        request.issued_at,
        request.session_public_key.clone(),
        request.session_nkey.clone(),
        request.participant_id.clone(),
        request.participant_artifact_digest.clone(),
        request.redirect_target.clone(),
        request_digest.clone(),
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let proof = parse_session_proof_v1(&request.proof)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let now = now_ms()?;
    let (portal, _) = select_login_portal(
        state.service.repository(),
        &request.participant_id,
        &request.redirect_target,
    )
    .await?;
    let verified = verify_session_proof_v1(
        &input,
        &proof,
        &request.session_public_key,
        now,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    if let Some(participant_value) = &request.participant_artifact {
        let participant = parse_participant_v1(participant_value)
            .map_err(|_| HttpError::bad_request("invalid_participant_artifact"))?;
        let mut apis = BTreeMap::new();
        let mut api_values = BTreeMap::new();
        for value in &request.referenced_api_artifacts {
            let api =
                parse_api_v1(value).map_err(|_| HttpError::bad_request("invalid_api_artifact"))?;
            api_values.insert(
                api.id().to_owned(),
                api.normalized_value()
                    .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
            );
            apis.insert(api.id().to_owned(), api);
        }
        let resolved = resolve_participant_v1(&participant, &apis)
            .map_err(|_| HttpError::bad_request("participant_resolution_failed"))?;
        let needs_digest = resolved
            .needs()
            .digest()
            .map_err(|_| HttpError::bad_request("participant_resolution_failed"))?;
        if resolved.participant_id() != request.participant_id
            || resolved.participant_digest() != request.participant_artifact_digest
            || needs_digest != request.participant_needs_digest
        {
            return Err(HttpError::bad_request("participant_binding_mismatch"));
        }
        state
            .service
            .repository()
            .put_participant_binding(ParticipantBindingRecord {
                participant_id: resolved.participant_id().to_owned(),
                participant_kind: resolved.participant_kind(),
                artifact_digest: resolved.participant_digest().to_owned(),
                needs_digest,
                participant_json: participant
                    .canonical_json()
                    .map_err(|_| HttpError::bad_request("invalid_participant_artifact"))?,
                api_artifacts_json: canonicalize_json(
                    &serde_json::to_value(api_values)
                        .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
                )
                .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
                resolved_at: now,
                state: ParticipantBindingState::Resolved,
                error: None,
            })
            .await?;
    }
    let binding = state
        .service
        .repository()
        .get_participant_binding(
            &request.participant_id,
            &request.participant_artifact_digest,
        )
        .await?
        .ok_or_else(|| HttpError::bad_request("participant_binding_unknown"))?;
    if binding.state != ParticipantBindingState::Resolved
        || binding.needs_digest != request.participant_needs_digest
    {
        return Err(HttpError::bad_request("participant_binding_mismatch"));
    }
    let consent = browser_consent(&binding)?;
    let replay = verified.replay_key();
    let flow_id = format!(
        "flow_{}",
        digest_parts(&[
            "user_auth_request",
            replay.signer_key_id(),
            replay.request_id(),
        ])
    );
    let flow = AuthBrowserFlow {
        format: BROWSER_FLOW_FORMAT.to_owned(),
        flow_id: flow_id.clone(),
        kind: AuthBrowserFlowKind::UserAuth,
        state: AuthBrowserFlowState::ChooseProvider,
        request_id: request.request_id,
        request_digest,
        participant_id: request.participant_id,
        participant_artifact_digest: request.participant_artifact_digest,
        participant_needs_digest: request.participant_needs_digest,
        consent,
        session_public_key: request.session_public_key,
        session_nkey: request.session_nkey,
        portal_id: portal.portal_id.clone(),
        redirect_target: Some(request.redirect_target),
        principal_id: None,
        claim_owner: None,
        claimed_at: None,
        durable_result_digest: None,
        completed_at: None,
        created_at: now,
        expires_at: checked_add(now, FLOW_TTL_MS)?,
        version: 1,
    };
    match state.ephemeral.create_browser_flow(flow.clone()).await {
        Ok(()) => {}
        Err(AuthorizationStateError::StorageConflict) => {
            let existing = state
                .ephemeral
                .get_browser_flow(&flow_id)
                .await?
                .ok_or_else(|| HttpError::conflict("proof_replay"))?;
            if existing.request_digest != flow.request_digest
                || existing.session_public_key != flow.session_public_key
            {
                return Err(HttpError::conflict("proof_replay"));
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(Json(AuthStartResponse {
        state: "flow",
        flow_id: flow_id.clone(),
        portal_url: portal_url(&portal, &state.public_origin, &flow_id)?,
    }))
}

async fn select_login_portal(
    repository: &impl LoginPortalRepository,
    participant_id: &str,
    redirect_target: &str,
) -> Result<(LoginPortalRecord, LoginSettingsRecord), HttpError> {
    let origin = canonical_origin(redirect_target)
        .map_err(|_| HttpError::bad_request("invalid_redirect_target"))?;
    for route in repository.list_portal_routes().await? {
        if route.deployment_id.is_some()
            || route
                .participant_id
                .as_deref()
                .is_some_and(|value| value != participant_id)
            || route.origin.as_deref().is_some_and(|value| value != origin)
        {
            continue;
        }
        if let Some((portal, settings)) = repository.get_login_portal(&route.portal_id).await? {
            if !portal.disabled && !portal.removed {
                return Ok((portal, settings));
            }
        }
    }
    repository
        .get_login_portal("builtin")
        .await?
        .filter(|(portal, _)| !portal.disabled && !portal.removed)
        .ok_or_else(|| HttpError::internal("builtin_portal_unavailable"))
}

async fn select_device_portal(
    repository: &impl LoginPortalRepository,
    participant_id: &str,
    deployment_id: &str,
) -> Result<LoginPortalRecord, HttpError> {
    for route in repository.list_portal_routes().await? {
        if route.origin.is_some()
            || route
                .participant_id
                .as_deref()
                .is_some_and(|value| value != participant_id)
            || route
                .deployment_id
                .as_deref()
                .is_some_and(|value| value != deployment_id)
        {
            continue;
        }
        if let Some((portal, _)) = repository.get_login_portal(&route.portal_id).await? {
            if !portal.disabled && !portal.removed {
                return Ok(portal);
            }
        }
    }
    repository
        .get_login_portal("builtin")
        .await?
        .map(|(portal, _)| portal)
        .filter(|portal| !portal.disabled && !portal.removed)
        .ok_or_else(|| HttpError::internal("builtin_portal_unavailable"))
}

fn portal_url(
    portal: &LoginPortalRecord,
    public_origin: &str,
    flow_id: &str,
) -> Result<String, HttpError> {
    let entry = portal.entry_url.as_deref().map_or_else(
        || {
            format!(
                "{}/_trellis/portal/auth",
                public_origin.trim_end_matches('/')
            )
        },
        ToOwned::to_owned,
    );
    let mut url = Url::parse(&entry).map_err(|_| HttpError::internal("portal_entry_invalid"))?;
    url.query_pairs_mut().append_pair("flowId", flow_id);
    Ok(url.into())
}

async fn portal_index<R, E>(State(state): State<AuthHttpState<R, E>>) -> Response
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    portal_file(&state, "200.html").await
}

async fn portal_page<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(path): Path<String>,
) -> Response
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let direct = format!("_trellis/portal/{path}");
    let response = portal_file(&state, &direct).await;
    if response.status() == StatusCode::NOT_FOUND {
        portal_file(&state, "200.html").await
    } else {
        response
    }
}

async fn portal_asset<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(path): Path<String>,
) -> Response
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    portal_file(&state, &format!("_trellis/assets/{path}")).await
}

async fn portal_file<R, E>(state: &AuthHttpState<R, E>, path: &str) -> Response {
    if std::path::Path::new(path)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let bytes = if let Some(directory) = &state.portal_override_dir {
        match tokio::fs::read(directory.join(path)).await {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        EMBEDDED_PORTAL_ASSETS
            .iter()
            .find_map(|(asset_path, bytes)| (*asset_path == path).then(|| bytes.to_vec()))
    };
    let Some(bytes) = bytes else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let content_type = match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("ico") => "image/x-icon",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    ([(CONTENT_TYPE, content_type)], bytes).into_response()
}

fn device_activation_url(
    portal: &LoginPortalRecord,
    public_origin: &str,
    review_id: &str,
) -> Result<String, HttpError> {
    let entry = portal.entry_url.as_deref().map_or_else(
        || {
            format!(
                "{}/_trellis/portal/device",
                public_origin.trim_end_matches('/')
            )
        },
        ToOwned::to_owned,
    );
    let mut url = Url::parse(&entry).map_err(|_| HttpError::internal("portal_entry_invalid"))?;
    url.query_pairs_mut().append_pair("flowId", review_id);
    Ok(url.into())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ServiceBootstrapRequest {
    request_id: String,
    issued_at: i64,
    deployment_id: String,
    instance_id: String,
    provisioned_identity_key_id: String,
    new_session_public_key: String,
    new_session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Option<Value>,
    referenced_api_artifacts: Option<Vec<Value>>,
    proof: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct DeviceBootstrapRequest {
    request_id: String,
    issued_at: i64,
    deployment_id: String,
    instance_id: String,
    device_identity_key_id: String,
    principal_id: Option<String>,
    identity_public_key: Option<String>,
    provisioning_secret: Option<String>,
    expected_secret_version: Option<u64>,
    new_session_public_key: String,
    new_session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Option<Value>,
    referenced_api_artifacts: Option<Vec<Value>>,
    challenge_digest: Option<String>,
    proof: Value,
}

struct BootstrapInput {
    request_id: String,
    issued_at: i64,
    deployment_id: String,
    instance_id: String,
    identity_key_id: String,
    principal_id: Option<String>,
    identity_public_key: Option<String>,
    provisioning_secret: Option<String>,
    expected_secret_version: Option<u64>,
    new_session_public_key: String,
    new_session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Option<Value>,
    referenced_api_artifacts: Option<Vec<Value>>,
    challenge_digest: Option<String>,
    proof: Value,
    request_digest: String,
    kind: ProvisionedIdentityKind,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapResponse {
    server_now: i64,
    state: &'static str,
    session: Option<SessionRecord>,
    authorization: Option<BootstrapAuthorization>,
    nats: Option<NatsBootstrapResponse>,
    authorization_context: Option<super::AuthorizationContextBundle>,
    activation: Option<BootstrapActivation>,
    proposal: Option<BootstrapProposal>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapProposal {
    proposal_id: String,
    proposal_kind: AuthorityProposalKind,
    proposal_digest: String,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapAuthorization {
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_json: String,
    effective_grants: GrantSetV1,
    resource_bindings: Vec<ResourceBindingEvidence>,
    resource_runtime: ServiceResourceBindings,
    effective_authority_expires_at: Option<i64>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientBootstrapRequest {
    request_id: String,
    issued_at: i64,
    session_id: String,
    session_nkey: String,
    expected_participant_digest: Option<String>,
    expected_needs_digest: Option<String>,
    proof: Value,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ClientBootstrapResponse {
    server_now: i64,
    session_id: String,
    inbox_prefix: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_json: String,
    profile: UserProfileRecord,
    effective_grants: GrantSetV1,
    resource_bindings: Vec<ResourceBindingEvidence>,
    resource_runtime: ServiceResourceBindings,
    effective_authority_expires_at: Option<i64>,
    nats: NatsBootstrapResponse,
    authorization_context: super::AuthorizationContextBundle,
}

async fn client_bootstrap<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(request): Json<ClientBootstrapRequest>,
) -> Result<Json<ClientBootstrapResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let now = now_ms()?;
    let session = state
        .service
        .repository()
        .get_session(&request.session_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("auth_required"))?;
    let mut digest_value = serde_json::to_value(&request)
        .map_err(|_| HttpError::bad_request("invalid_client_bootstrap"))?;
    if let Some(object) = digest_value.as_object_mut() {
        object.insert("proof".to_owned(), Value::Null);
    }
    let request_digest = session_proof_request_digest_v1(&digest_value)
        .map_err(|_| HttpError::bad_request("invalid_client_bootstrap"))?;
    let input = SessionProofInputV1::client_bootstrap(
        &request.request_id,
        request.issued_at,
        &request.session_id,
        &session.session_key_id,
        &session.session_public_key,
        &request.session_nkey,
        request.expected_participant_digest.clone(),
        request.expected_needs_digest.clone(),
        &request_digest,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof_v1(
        &input,
        &parse_session_proof_v1(&request.proof)
            .map_err(|_| HttpError::unauthorized("invalid_proof"))?,
        &session.session_public_key,
        now,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;

    if let Some(replay) = state
        .service
        .repository()
        .get_idempotency_result(
            "bootstrap.client",
            &session.session_key_id,
            &request.request_id,
        )
        .await?
    {
        if replay.request_digest != request_digest {
            return Err(HttpError::conflict("idempotency_collision"));
        }
        if replay.result.get("sessionId").and_then(Value::as_str)
            != Some(request.session_id.as_str())
        {
            return Err(HttpError::internal("invalid_client_bootstrap_replay"));
        }
    }

    let issuance = state
        .service
        .authorization()
        .resolve_issuable_state(&request.session_id, now)
        .await
        .map_err(map_issuance_error)?;
    let participant = state
        .service
        .repository()
        .get_participant_binding(
            &issuance.participant.id,
            &issuance.participant.artifact_digest,
        )
        .await?
        .filter(|binding| binding.needs_digest == issuance.participant.needs_digest)
        .ok_or_else(|| HttpError::conflict("participant_unavailable"))?;
    if issuance.principal.kind != AuthorizationPrincipalKindV1::User
        || !matches!(
            issuance.participant.kind,
            ParticipantKindV1::App | ParticipantKindV1::Agent
        )
        || request
            .expected_participant_digest
            .as_deref()
            .is_some_and(|digest| digest != issuance.participant.artifact_digest)
        || request
            .expected_needs_digest
            .as_deref()
            .is_some_and(|digest| digest != issuance.participant.needs_digest)
    {
        return Err(HttpError::unauthorized("auth_required"));
    }
    let profile = state
        .service
        .repository()
        .get_user_profile(&issuance.principal.id)
        .await?
        .ok_or_else(|| HttpError::internal("user_profile_missing"))?;
    let credential_expires_at = [
        issuance.session_expires_at,
        issuance.effective_authority_expires_at,
        issuance.delegation_expires_at,
    ]
    .into_iter()
    .flatten()
    .min()
    .ok_or_else(|| HttpError::internal("credential_expiry_missing"))?;
    let authorization_context = state
        .authorization_contexts
        .issue(
            super::AuthorizationContextIssueRequest {
                session_id: request.session_id.clone(),
                request_id: request.request_id.clone(),
                request_digest: request_digest.clone(),
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    let route = state.issuer.deny_all_user_jwt(
        &request.session_nkey,
        credential_expires_at / 1_000,
        now / 1_000,
    )?;
    let response = ClientBootstrapResponse {
        server_now: now,
        session_id: issuance.session_id.clone(),
        inbox_prefix: issuance.inbox_prefix.clone(),
        participant_id: issuance.participant.id.clone(),
        participant_artifact_digest: issuance.participant.artifact_digest.clone(),
        participant_needs_digest: issuance.participant.needs_digest.clone(),
        participant_json: participant.participant_json.clone(),
        profile,
        effective_grants: issuance.grant_set.clone(),
        resource_bindings: issuance.resource_bindings.clone(),
        resource_runtime: project_service_resource_bindings(
            &participant.participant_json,
            &issuance.resource_bindings,
            &issuance.participant.id,
        )?,
        effective_authority_expires_at: issuance.effective_authority_expires_at,
        nats: NatsBootstrapResponse {
            jwt: route.jwt,
            jwt_expires_at: route.expires_at,
            servers: state.websocket_nats_servers.clone(),
        },
        authorization_context,
    };
    let outcome = state
        .service
        .repository()
        .admit_client_bootstrap(ClientBootstrapAdmission {
            session_id: request.session_id.clone(),
            observed_at: now,
            idempotency: IdempotencyResultRecord {
                scope_key: session.session_key_id.clone(),
                purpose: "bootstrap.client".to_owned(),
                signer_id: session.session_key_id,
                request_id: request.request_id,
                request_digest,
                result: json!({ "sessionId": request.session_id }),
                created_at: now,
                expires_at: session.expires_at.unwrap_or(
                    now.checked_add(IDEMPOTENCY_TTL_MS)
                        .ok_or_else(|| HttpError::internal("idempotency_expiry_overflow"))?,
                ),
            },
        })
        .await?;
    match outcome {
        IdempotentOutcome::Applied(_) | IdempotentOutcome::Replayed(_) => Ok(Json(response)),
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextRefreshRequest {
    request_id: String,
    issued_at: i64,
    session_id: String,
    session_nkey: String,
    current_context_digest: RequiredNullableString,
    expected_participant_digest: Option<String>,
    expected_needs_digest: Option<String>,
    known_root_key_id: String,
    minimum_manifest_generation: i64,
    proof: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(transparent)]
struct RequiredNullableString(Option<String>);

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContextRefreshResponse {
    server_now: i64,
    authorization_context: super::AuthorizationContextBundle,
    bootstrap_jwt: String,
    bootstrap_jwt_expires_at: i64,
}

async fn read_trust_registry<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(key): Path<String>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    registry_response(
        &key,
        state
            .authorization_contexts
            .read_trust_registry(&key)
            .await?,
    )
}

async fn read_context_registry<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(digest): Path<String>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    registry_response(
        &digest,
        state
            .authorization_contexts
            .read_context_registry(&digest)
            .await?,
    )
}

async fn read_revocation_snapshot<R, E>(
    State(state): State<AuthHttpState<R, E>>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let records = state
        .authorization_contexts
        .read_revocation_snapshot()
        .await?;
    let payload = canonicalize_json(
        &serde_json::to_value(records)
            .map_err(|_| HttpError::internal("registry_response_failed"))?,
    )
    .map_err(|_| HttpError::internal("registry_response_failed"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "no-store")
        .body(Body::from(payload))
        .map_err(|_| HttpError::internal("registry_response_failed"))
}

fn registry_response(key: &str, value: Option<bytes::Bytes>) -> Result<Response, HttpError> {
    if key.is_empty()
        || !key
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
    {
        return Err(HttpError::bad_request("invalid_registry_key"));
    }
    let value = value.ok_or_else(|| HttpError::not_found("registry_entry_not_found"))?;
    Response::builder()
        .status(StatusCode::OK)
        .header(CONTENT_TYPE, "application/json")
        .header(CACHE_CONTROL, "public, max-age=60")
        .body(Body::from(value))
        .map_err(|_| HttpError::internal("registry_response_failed"))
}

async fn refresh_context<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(request): Json<ContextRefreshRequest>,
) -> Result<Json<ContextRefreshResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let now = now_ms()?;
    let session = state
        .service
        .repository()
        .get_session(&request.session_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("auth_required"))?;
    let session_nkey = KeyPair::from_public_key(&request.session_nkey)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let (_, session_nkey_bytes) = nkeys::from_public_key(&request.session_nkey)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let session_public_key = URL_SAFE_NO_PAD
        .decode(&session.session_public_key)
        .map_err(|_| HttpError::internal("invalid_session_key"))?;
    if session_nkey.key_pair_type() != KeyPairType::User
        || session_nkey_bytes
            .as_slice()
            .ct_eq(&session_public_key)
            .unwrap_u8()
            != 1
    {
        return Err(HttpError::unauthorized("invalid_proof"));
    }
    let digest_value = serde_json::to_value(&request)
        .map_err(|_| HttpError::bad_request("invalid_context_refresh"))?;
    let request_digest = session_proof_request_digest_v1(&digest_value)
        .map_err(|_| HttpError::bad_request("invalid_context_refresh"))?;
    let input = SessionProofInputV1::authorization_context_refresh(
        &request.request_id,
        request.issued_at,
        &request.session_id,
        &session.session_key_id,
        request.current_context_digest.0.clone(),
        request.expected_participant_digest.clone(),
        request.expected_needs_digest.clone(),
        &request.known_root_key_id,
        request.minimum_manifest_generation,
        &request_digest,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof_v1(
        &input,
        &parse_session_proof_v1(&request.proof)
            .map_err(|_| HttpError::unauthorized("invalid_proof"))?,
        &session.session_public_key,
        now,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    if request.known_root_key_id != state.authorization_contexts.root_key_id()
        || u64::try_from(request.minimum_manifest_generation)
            .ok()
            .is_none_or(|minimum| minimum > state.authorization_contexts.manifest_generation())
    {
        return Err(HttpError::conflict("context_refresh_mismatch"));
    }
    if let Some(current_digest) = request.current_context_digest.0.as_deref() {
        let current = state
            .authorization_contexts
            .require_current_context(&request.session_id, current_digest, now / 1_000)
            .await
            .map_err(map_issuance_error)?;
        if request
            .expected_participant_digest
            .as_deref()
            .is_some_and(|expected| expected != current.participant_artifact_digest)
            || request
                .expected_needs_digest
                .as_deref()
                .is_some_and(|expected| expected != current.participant_needs_digest)
        {
            return Err(HttpError::conflict("context_refresh_mismatch"));
        }
    }
    let authorization_context = state
        .authorization_contexts
        .issue(
            super::AuthorizationContextIssueRequest {
                session_id: request.session_id.clone(),
                request_id: request.request_id,
                request_digest,
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    let issued = state
        .authorization_contexts
        .require_current_context(
            &request.session_id,
            &authorization_context.context_digest,
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    if request
        .expected_participant_digest
        .as_deref()
        .is_some_and(|expected| expected != issued.participant_artifact_digest)
        || request
            .expected_needs_digest
            .as_deref()
            .is_some_and(|expected| expected != issued.participant_needs_digest)
    {
        return Err(HttpError::conflict("context_refresh_mismatch"));
    }
    let issuance = state
        .service
        .authorization()
        .resolve_issuable_state(&request.session_id, now)
        .await
        .map_err(map_issuance_error)?;
    let expires_at = [
        issuance.session_expires_at,
        issuance.effective_authority_expires_at,
        issuance.delegation_expires_at,
    ]
    .into_iter()
    .flatten()
    .min()
    .ok_or_else(|| HttpError::internal("credential_expiry_missing"))?;
    let route =
        state
            .issuer
            .deny_all_user_jwt(&request.session_nkey, expires_at / 1_000, now / 1_000)?;
    Ok(Json(ContextRefreshResponse {
        server_now: now,
        authorization_context,
        bootstrap_jwt: route.jwt,
        bootstrap_jwt_expires_at: route.expires_at,
    }))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogoutRequest {
    request_id: String,
    issued_at: i64,
    session_id: String,
    reason: Option<String>,
    proof: Value,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LogoutResponse {
    session_id: String,
    state: &'static str,
}

async fn logout_session<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(request): Json<LogoutRequest>,
) -> Result<Json<LogoutResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let now = now_ms()?;
    let session = state
        .service
        .repository()
        .get_session(&request.session_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("auth_required"))?;
    let mut digest_value =
        serde_json::to_value(&request).map_err(|_| HttpError::bad_request("invalid_logout"))?;
    if let Some(object) = digest_value.as_object_mut() {
        object.insert("proof".to_owned(), Value::Null);
    }
    let request_digest = session_proof_request_digest_v1(&digest_value)
        .map_err(|_| HttpError::bad_request("invalid_logout"))?;
    let input = SessionProofInputV1::session_self_control(
        &request.request_id,
        request.issued_at,
        &request.session_id,
        &session.session_key_id,
        &request_digest,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof_v1(
        &input,
        &parse_session_proof_v1(&request.proof)
            .map_err(|_| HttpError::unauthorized("invalid_proof"))?,
        &session.session_public_key,
        now,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let action = |kind, suffix: &str, payload| PostCommitActionRecord {
        action_id: format!(
            "act_{}",
            digest_parts(&[&request.session_id, &request.request_id, suffix])
        ),
        kind,
        payload,
        created_at: now,
        attempts: 0,
        next_attempt_at: now,
        claimed_until: None,
        last_error: None,
    };
    state
        .service
        .revoke_session(
            request.session_id.clone(),
            session.version,
            now,
            idempotency(
                &request.session_id,
                "session.logout",
                &session.session_key_id,
                &request.request_id,
                &request_digest,
                now,
            )?,
            vec![
                action(
                    PostCommitActionKind::Event,
                    "event",
                    json!({
                        "eventType": "Auth.Sessions.Revoked",
                        "eventId": format!(
                            "evt_{}",
                            digest_parts(&[
                                &request.session_id,
                                &request.request_id,
                                "event",
                            ])
                        ),
                        "occurredAt": now,
                        "sessionId": request.session_id,
                        "principalId": session.principal_id.clone(),
                        "participantId": session.participant_id.clone(),
                        "reason": request.reason,
                        "revokedBy": session.principal_id.clone(),
                    }),
                ),
                action(
                    PostCommitActionKind::Kick,
                    "kick",
                    json!({ "sessionId": request.session_id }),
                ),
            ],
        )
        .await?;
    Ok(Json(LogoutResponse {
        session_id: request.session_id,
        state: "revoked",
    }))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BootstrapActivation {
    state: &'static str,
    review_id: String,
    activation_url: String,
}

async fn service_bootstrap<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<BootstrapResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let request: ServiceBootstrapRequest = serde_json::from_value(raw.clone())
        .map_err(|_| HttpError::bad_request("invalid_service_bootstrap"))?;
    let request_digest = proof_request_digest(&raw)
        .map_err(|_| HttpError::bad_request("invalid_service_bootstrap"))?;
    bootstrap(
        &state,
        BootstrapInput {
            request_id: request.request_id,
            issued_at: request.issued_at,
            deployment_id: request.deployment_id,
            instance_id: request.instance_id,
            identity_key_id: request.provisioned_identity_key_id,
            principal_id: None,
            identity_public_key: None,
            provisioning_secret: None,
            expected_secret_version: None,
            new_session_public_key: request.new_session_public_key,
            new_session_nkey: request.new_session_nkey,
            participant_id: request.participant_id,
            participant_artifact_digest: request.participant_artifact_digest,
            participant_needs_digest: request.participant_needs_digest,
            participant_artifact: request.participant_artifact,
            referenced_api_artifacts: request.referenced_api_artifacts,
            challenge_digest: None,
            proof: request.proof,
            request_digest,
            kind: ProvisionedIdentityKind::Service,
        },
    )
    .await
    .map(Json)
}

async fn device_bootstrap<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<BootstrapResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let request: DeviceBootstrapRequest = serde_json::from_value(raw.clone())
        .map_err(|_| HttpError::bad_request("invalid_device_bootstrap"))?;
    let request_digest = proof_request_digest(&raw)
        .map_err(|_| HttpError::bad_request("invalid_device_bootstrap"))?;
    bootstrap(&state, device_bootstrap_input(request, request_digest))
        .await
        .map(Json)
}

fn device_bootstrap_input(
    request: DeviceBootstrapRequest,
    request_digest: String,
) -> BootstrapInput {
    BootstrapInput {
        request_id: request.request_id,
        issued_at: request.issued_at,
        deployment_id: request.deployment_id,
        instance_id: request.instance_id,
        identity_key_id: request.device_identity_key_id,
        principal_id: request.principal_id,
        identity_public_key: request.identity_public_key,
        provisioning_secret: request.provisioning_secret,
        expected_secret_version: request.expected_secret_version,
        new_session_public_key: request.new_session_public_key,
        new_session_nkey: request.new_session_nkey,
        participant_id: request.participant_id,
        participant_artifact_digest: request.participant_artifact_digest,
        participant_needs_digest: request.participant_needs_digest,
        participant_artifact: request.participant_artifact,
        referenced_api_artifacts: request.referenced_api_artifacts,
        challenge_digest: request.challenge_digest,
        proof: request.proof,
        request_digest,
        kind: ProvisionedIdentityKind::Device,
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct DeviceActivationWaitRequest {
    review_id: String,
    wait_ms: Option<u64>,
    bootstrap: DeviceBootstrapRequest,
}

async fn wait_for_device_activation<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(request): Json<DeviceActivationWaitRequest>,
) -> Result<Json<BootstrapResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let raw = serde_json::to_value(&request.bootstrap)
        .map_err(|_| HttpError::bad_request("invalid_device_bootstrap"))?;
    let request_digest = proof_request_digest(&raw)
        .map_err(|_| HttpError::bad_request("invalid_device_bootstrap"))?;
    let identity = state
        .service
        .repository()
        .get_provisioned_identity(&request.bootstrap.device_identity_key_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("identity_not_found"))?;
    if identity.kind != ProvisionedIdentityKind::Device
        || request
            .bootstrap
            .principal_id
            .as_deref()
            .is_some_and(|principal_id| principal_id != identity.principal_id)
        || identity.deployment_id != request.bootstrap.deployment_id
        || identity.instance_id != request.bootstrap.instance_id
    {
        return Err(HttpError::unauthorized("identity_mismatch"));
    }
    let proof_input = SessionProofInputV1::device_bootstrap(
        &request.bootstrap.request_id,
        request.bootstrap.issued_at,
        &request.bootstrap.deployment_id,
        &request.bootstrap.instance_id,
        &request.bootstrap.device_identity_key_id,
        &request.bootstrap.new_session_public_key,
        &request.bootstrap.new_session_nkey,
        &request.bootstrap.participant_id,
        &request.bootstrap.participant_artifact_digest,
        request.bootstrap.challenge_digest.clone(),
        &request_digest,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof_v1(
        &proof_input,
        &parse_session_proof_v1(&request.bootstrap.proof)
            .map_err(|_| HttpError::unauthorized("invalid_proof"))?,
        &identity.identity_public_key,
        now_ms()?,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;

    let mut review = state
        .service
        .repository()
        .get_activation_review(&request.review_id)
        .await?
        .ok_or_else(|| HttpError::not_found("activation_review_not_found"))?;
    if review.principal_id != identity.principal_id
        || review.deployment_id != identity.deployment_id
        || review.instance_id != identity.instance_id
    {
        return Err(HttpError::unauthorized("activation_review_mismatch"));
    }
    if review.state == DeviceActivationReviewState::Pending {
        let wait_ms = request.wait_ms.unwrap_or(0).min(30_000);
        if wait_ms > 0 {
            tokio::time::sleep(Duration::from_millis(wait_ms)).await;
            review = state
                .service
                .repository()
                .get_activation_review(&request.review_id)
                .await?
                .ok_or_else(|| HttpError::not_found("activation_review_not_found"))?;
        }
    }
    match review.state {
        DeviceActivationReviewState::Approved => bootstrap(
            &state,
            device_bootstrap_input(request.bootstrap, request_digest),
        )
        .await
        .map(Json),
        DeviceActivationReviewState::Pending => {
            let portal = select_device_portal(
                state.service.repository(),
                &request.bootstrap.participant_id,
                &request.bootstrap.deployment_id,
            )
            .await?;
            Ok(Json(BootstrapResponse {
                server_now: now_ms()?,
                state: "activation_pending",
                session: None,
                authorization: None,
                nats: None,
                authorization_context: None,
                activation: Some(BootstrapActivation {
                    state: "pending",
                    activation_url: device_activation_url(
                        &portal,
                        &state.public_origin,
                        &request.review_id,
                    )?,
                    review_id: request.review_id,
                }),
                proposal: None,
            }))
        }
        DeviceActivationReviewState::Rejected
        | DeviceActivationReviewState::Cancelled
        | DeviceActivationReviewState::Expired => Ok(Json(BootstrapResponse {
            server_now: now_ms()?,
            state: "activation_rejected",
            session: None,
            authorization: None,
            nats: None,
            authorization_context: None,
            activation: None,
            proposal: None,
        })),
    }
}

async fn bootstrap<R, E>(
    state: &AuthHttpState<R, E>,
    input: BootstrapInput,
) -> Result<BootstrapResponse, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let existing_identity = state
        .service
        .repository()
        .get_provisioned_identity(&input.identity_key_id)
        .await?;
    let verifying_public_key = if let Some(identity) = &existing_identity {
        identity.identity_public_key.clone()
    } else if input.kind == ProvisionedIdentityKind::Device {
        let public_key = input
            .identity_public_key
            .clone()
            .ok_or_else(|| HttpError::unauthorized("identity_not_found"))?;
        let derived = super::domain::validate_ed25519_public_key("identityPublicKey", &public_key)?;
        if derived != input.identity_key_id {
            return Err(HttpError::unauthorized("identity_key_mismatch"));
        }
        public_key
    } else {
        return Err(HttpError::unauthorized("identity_not_found"));
    };
    let proof_input = match input.kind {
        ProvisionedIdentityKind::Service => SessionProofInputV1::service_bootstrap(
            input.request_id.clone(),
            input.issued_at,
            input.deployment_id.clone(),
            input.instance_id.clone(),
            input.identity_key_id.clone(),
            input.new_session_public_key.clone(),
            input.new_session_nkey.clone(),
            input.participant_id.clone(),
            input.participant_artifact_digest.clone(),
            input.request_digest.clone(),
        ),
        ProvisionedIdentityKind::Device => SessionProofInputV1::device_bootstrap(
            input.request_id.clone(),
            input.issued_at,
            input.deployment_id.clone(),
            input.instance_id.clone(),
            input.identity_key_id.clone(),
            input.new_session_public_key.clone(),
            input.new_session_nkey.clone(),
            input.participant_id.clone(),
            input.participant_artifact_digest.clone(),
            input.challenge_digest.clone(),
            input.request_digest.clone(),
        ),
    }
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let proof = parse_session_proof_v1(&input.proof)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let verified = verify_session_proof_v1(
        &proof_input,
        &proof,
        &verifying_public_key,
        now_ms()?,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let replay = verified.replay_key();
    let now = now_ms()?;
    if existing_identity.is_none() {
        state
            .service
            .enroll_device_identity(EnrollDeviceIdentityInput {
                provisioning_secret: input
                    .provisioning_secret
                    .clone()
                    .ok_or_else(|| HttpError::unauthorized("provisioning_secret_required"))?,
                expected_version: input
                    .expected_secret_version
                    .ok_or_else(|| HttpError::bad_request("expected_secret_version_required"))?,
                principal_id: input
                    .principal_id
                    .clone()
                    .ok_or_else(|| HttpError::bad_request("principal_id_required"))?,
                deployment_id: input.deployment_id.clone(),
                instance_id: input.instance_id.clone(),
                identity_public_key: verifying_public_key,
                consumed_at: now,
                idempotency: idempotency(
                    &input.identity_key_id,
                    "device.identity.enroll",
                    replay.signer_key_id(),
                    replay.request_id(),
                    &input.request_digest,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
    }
    let identity = state
        .service
        .repository()
        .get_provisioned_identity(&input.identity_key_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("identity_not_found"))?;
    if identity.state != ProvisionedIdentityState::Active
        || identity.kind != input.kind
        || identity.deployment_id != input.deployment_id
        || identity.instance_id != input.instance_id
    {
        return Err(HttpError::unauthorized("identity_mismatch"));
    }
    let instance = state
        .service
        .repository()
        .get_runtime_instance(&input.instance_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("instance_not_found"))?;
    if instance.principal_id != identity.principal_id
        || instance.deployment_id != input.deployment_id
        || instance.state != RuntimeInstanceState::Active
    {
        return Err(HttpError::unauthorized("instance_mismatch"));
    }
    let binding = state
        .service
        .repository()
        .get_participant_binding(&input.participant_id, &input.participant_artifact_digest)
        .await?;
    let Some(binding) = binding else {
        if input.participant_artifact.is_none() && input.referenced_api_artifacts.is_none() {
            return Ok(bootstrap_state(now, "manifest_required", None));
        }
        let proposal = present_bootstrap_authority(
            state,
            &input,
            None,
            replay.signer_key_id(),
            replay.request_id(),
            now,
        )
        .await?;
        return Ok(bootstrap_proposal_state(now, proposal));
    };
    if binding.needs_digest != input.participant_needs_digest {
        return Err(HttpError::conflict("participant_needs_changed"));
    }
    let authority = state
        .service
        .repository()
        .get_deployment_authority(&input.deployment_id, &input.participant_id)
        .await?;
    let Some(authority) = authority else {
        let proposal = present_bootstrap_authority(
            state,
            &input,
            Some(&binding),
            replay.signer_key_id(),
            replay.request_id(),
            now,
        )
        .await?;
        return Ok(bootstrap_proposal_state(now, proposal));
    };
    if authority.participant_artifact_digest != input.participant_artifact_digest
        || authority.accepted_needs_digest != input.participant_needs_digest
    {
        let proposal = present_bootstrap_authority(
            state,
            &input,
            Some(&binding),
            replay.signer_key_id(),
            replay.request_id(),
            now,
        )
        .await?;
        return Ok(bootstrap_proposal_state(now, proposal));
    }
    match authority.state {
        AuthorityState::Accepted => {}
        AuthorityState::Pending => return Ok(bootstrap_state(now, "authority_pending", None)),
        AuthorityState::Rejected | AuthorityState::Revoked => {
            return Ok(bootstrap_state(now, "authority_rejected", None));
        }
        AuthorityState::Stale => return Ok(bootstrap_state(now, "migration_required", None)),
    }
    let deployment = state
        .service
        .repository()
        .get_deployment_evidence(&input.deployment_id)
        .await?
        .ok_or_else(|| HttpError::unauthorized("deployment_not_found"))?;
    if !deployment.active
        || deployment.participant_id != input.participant_id
        || deployment
            .expires_at
            .is_some_and(|expires_at| expires_at <= now)
    {
        return Ok(bootstrap_state(now, "disabled", None));
    }
    let activation = if input.kind == ProvisionedIdentityKind::Device {
        let device = state
            .service
            .repository()
            .get_device(&identity.principal_id, &input.deployment_id)
            .await?
            .ok_or_else(|| HttpError::unauthorized("device_not_found"))?;
        if device.state == DeviceState::Disabled {
            let challenge_digest = input
                .challenge_digest
                .clone()
                .ok_or_else(|| HttpError::bad_request("activation_challenge_required"))?;
            let review = state
                .service
                .create_activation_review(CreateActivationReviewInput {
                    principal_id: identity.principal_id.clone(),
                    deployment_id: input.deployment_id.clone(),
                    instance_id: input.instance_id.clone(),
                    request_digest: challenge_digest,
                    payload: json!({ "source": "device_bootstrap" }),
                    requested_at: now,
                    idempotency: idempotency(
                        &input.identity_key_id,
                        "device.activation.request",
                        replay.signer_key_id(),
                        replay.request_id(),
                        &input.request_digest,
                        now,
                    )?,
                    actions: Vec::new(),
                })
                .await?;
            let review_id = match review {
                IdempotentOutcome::Applied(review) => review.review_id,
                IdempotentOutcome::Replayed(value) => value
                    .get("reviewId")
                    .and_then(Value::as_str)
                    .ok_or_else(|| HttpError::internal("invalid_activation_replay"))?
                    .to_owned(),
            };
            let portal = select_device_portal(
                state.service.repository(),
                &input.participant_id,
                &input.deployment_id,
            )
            .await?;
            return Ok(BootstrapResponse {
                server_now: now,
                state: "activation_pending",
                session: None,
                authorization: None,
                nats: None,
                authorization_context: None,
                activation: Some(BootstrapActivation {
                    state: "pending",
                    activation_url: device_activation_url(
                        &portal,
                        &state.public_origin,
                        &review_id,
                    )?,
                    review_id,
                }),
                proposal: None,
            });
        } else if device.state != DeviceState::Active {
            return Err(HttpError::unauthorized("device_inactive"));
        } else {
            None
        }
    } else {
        None
    };
    let session = state
        .service
        .create_session(CreateSessionInput {
            principal_id: identity.principal_id,
            principal_kind: match input.kind {
                ProvisionedIdentityKind::Service => PrincipalKind::Service,
                ProvisionedIdentityKind::Device => PrincipalKind::Device,
            },
            participant_id: input.participant_id.clone(),
            participant_kind: binding.participant_kind,
            participant_artifact_digest: input.participant_artifact_digest,
            participant_needs_digest: input.participant_needs_digest,
            session_public_key: input.new_session_public_key,
            deployment_id: Some(input.deployment_id.clone()),
            instance_id: Some(input.instance_id),
            desired_authority: None,
            created_at: now,
            idempotency: idempotency(
                &input.identity_key_id,
                match input.kind {
                    ProvisionedIdentityKind::Service => "service.bootstrap",
                    ProvisionedIdentityKind::Device => "device.bootstrap",
                },
                replay.signer_key_id(),
                replay.request_id(),
                &input.request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let session = match session {
        IdempotentOutcome::Applied(session) => session,
        IdempotentOutcome::Replayed(value) => {
            let session_id = value
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_session_replay"))?;
            state
                .service
                .repository()
                .get_session(session_id)
                .await?
                .ok_or_else(|| HttpError::internal("session_missing"))?
        }
    };
    state
        .service
        .authorization()
        .reconcile_authority(
            &AuthorityTarget::new(AuthorityKind::Deployment, authority.authority_id)?,
            now,
        )
        .await?;
    let issuance = state
        .service
        .authorization()
        .resolve_issuable_state(&session.session_id, now)
        .await
        .map_err(map_issuance_error)?;
    let expires_at = [
        issuance.session_expires_at,
        issuance.effective_authority_expires_at,
        issuance.delegation_expires_at,
    ]
    .into_iter()
    .flatten()
    .min()
    .ok_or_else(|| HttpError::internal("credential_expiry_missing"))?;
    let authorization_context = state
        .authorization_contexts
        .issue(
            super::AuthorizationContextIssueRequest {
                session_id: session.session_id.clone(),
                request_id: replay.request_id().to_owned(),
                request_digest: input.request_digest.clone(),
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    let route =
        state
            .issuer
            .deny_all_user_jwt(&input.new_session_nkey, expires_at / 1_000, now / 1_000)?;
    Ok(BootstrapResponse {
        server_now: now,
        state: "ready",
        session: Some(session),
        authorization: Some(BootstrapAuthorization {
            participant_id: issuance.participant.id.clone(),
            participant_artifact_digest: issuance.participant.artifact_digest.clone(),
            participant_needs_digest: issuance.participant.needs_digest.clone(),
            participant_json: binding.participant_json.clone(),
            effective_grants: issuance.grant_set.clone(),
            resource_bindings: issuance.resource_bindings.clone(),
            resource_runtime: project_service_resource_bindings(
                &binding.participant_json,
                &issuance.resource_bindings,
                &issuance.participant.id,
            )?,
            effective_authority_expires_at: issuance.effective_authority_expires_at,
        }),
        nats: Some(NatsBootstrapResponse {
            jwt: route.jwt,
            jwt_expires_at: route.expires_at,
            servers: state.websocket_nats_servers.clone(),
        }),
        authorization_context: Some(authorization_context),
        activation,
        proposal: None,
    })
}

async fn present_bootstrap_authority<R, E>(
    state: &AuthHttpState<R, E>,
    input: &BootstrapInput,
    known_binding: Option<&ParticipantBindingRecord>,
    signer_id: &str,
    request_id: &str,
    now: i64,
) -> Result<AuthorityProposalRecord, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    if input.participant_artifact.is_some() != input.referenced_api_artifacts.is_some() {
        return Err(HttpError::bad_request(
            "incomplete_participant_presentation",
        ));
    }
    let (participant_artifact, referenced_api_artifacts) = match (
        input.participant_artifact.clone(),
        input.referenced_api_artifacts.clone(),
        known_binding,
    ) {
        (Some(participant), Some(apis), _) => (participant, apis),
        (None, None, Some(binding)) => {
            let participant = serde_json::from_str(&binding.participant_json)
                .map_err(|_| HttpError::internal("stored_participant_invalid"))?;
            let apis: BTreeMap<String, Value> =
                serde_json::from_str(&binding.api_artifacts_json)
                    .map_err(|_| HttpError::internal("stored_participant_invalid"))?;
            (participant, apis.into_values().collect())
        }
        (None, None, None) => return Err(HttpError::conflict("manifest_required")),
        _ => unreachable!(),
    };
    let outcome = state
        .service
        .present_deployment_authority(PresentDeploymentAuthorityInput {
            deployment_id: input.deployment_id.clone(),
            participant_artifact,
            referenced_api_artifacts,
            created_at: now,
            expires_at: None,
            idempotency: idempotency(
                &input.identity_key_id,
                "bootstrap.authority.plan",
                signer_id,
                request_id,
                &input.request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let proposal = match outcome {
        IdempotentOutcome::Applied(proposal) => proposal,
        IdempotentOutcome::Replayed(value) => {
            let proposal_id = value
                .get("proposalId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_proposal_replay"))?;
            state
                .service
                .repository()
                .get_authority_proposal(proposal_id)
                .await?
                .map(|value| value.0)
                .ok_or_else(|| HttpError::internal("proposal_missing"))?
        }
    };
    if proposal.participant_id != input.participant_id
        || proposal.participant_artifact_digest != input.participant_artifact_digest
        || proposal.participant_needs_digest != input.participant_needs_digest
    {
        return Err(HttpError::conflict("participant_presentation_mismatch"));
    }
    Ok(proposal)
}

fn bootstrap_proposal_state(now: i64, proposal: AuthorityProposalRecord) -> BootstrapResponse {
    let state = match proposal.state {
        AuthorityProposalState::Pending => match proposal.proposal_kind {
            AuthorityProposalKind::Migration => "migration_required",
            AuthorityProposalKind::Initial | AuthorityProposalKind::Update => "authority_pending",
        },
        AuthorityProposalState::Accepted => "dependency_pending",
        AuthorityProposalState::Rejected
        | AuthorityProposalState::Superseded
        | AuthorityProposalState::Expired => "authority_rejected",
    };
    let proposal = BootstrapProposal {
        proposal_id: proposal.proposal_id,
        proposal_kind: proposal.proposal_kind,
        proposal_digest: proposal.proposal_digest,
    };
    bootstrap_state(now, state, Some(proposal))
}

fn bootstrap_state(
    now: i64,
    state: &'static str,
    proposal: Option<BootstrapProposal>,
) -> BootstrapResponse {
    BootstrapResponse {
        server_now: now,
        state,
        session: None,
        authorization: None,
        nats: None,
        authorization_context: None,
        activation: None,
        proposal,
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserFlowResponse {
    flow_id: String,
    state: AuthBrowserFlowState,
    expires_at: i64,
    providers: Vec<String>,
    registration_enabled: bool,
    federated_registration_enabled: bool,
    consent_view: Value,
    consent_view_digest: String,
    user: Option<BrowserFlowUser>,
    redirect_target: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserFlowUser {
    origin: &'static str,
    id: String,
    name: Option<String>,
    email: Option<String>,
    image: Option<String>,
}

async fn get_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let flow = load_flow(&state.ephemeral, &flow_id).await?;
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    let providers = portal
        .provider_ids
        .iter()
        .filter(|provider| {
            (provider.as_str() == "local" && settings.local_login_enabled)
                || state.oidc_providers.contains_key(provider.as_str())
        })
        .cloned()
        .collect();
    let user = if let Some(principal_id) = &flow.principal_id {
        state
            .service
            .repository()
            .get_user_profile(principal_id)
            .await?
            .map(|profile| BrowserFlowUser {
                origin: "trellis",
                id: profile.principal_id,
                name: profile.display_name,
                email: profile.email,
                image: profile.image_url,
            })
    } else {
        None
    };
    let mut response = flow_response(flow);
    response.flow_id = flow_id;
    response.providers = providers;
    response.registration_enabled =
        portal.local_registration_enabled && settings.local_login_enabled;
    response.federated_registration_enabled = settings.federated_registration_enabled;
    response.user = user;
    Ok(Json(response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct LocalLoginRequest {
    flow_id: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AccountFlowResponse {
    status: &'static str,
    kind: Option<super::AccountFlowKind>,
    expires_at: Option<i64>,
}

async fn get_account_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_token): Path<String>,
) -> Result<Json<AccountFlowResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let Some((_, flow)) =
        load_account_flow_by_token(state.service.repository(), &flow_token).await?
    else {
        return Ok(Json(AccountFlowResponse {
            status: "expired",
            kind: None,
            expires_at: None,
        }));
    };
    let status = if flow.state == AccountFlowState::Consumed {
        "consumed"
    } else if flow.state != AccountFlowState::Pending || flow.expires_at < now_ms()? {
        "expired"
    } else {
        "pending"
    };
    Ok(Json(AccountFlowResponse {
        status,
        kind: Some(flow.kind),
        expires_at: Some(flow.expires_at),
    }))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct FirstAdminRequest {
    username: Option<String>,
    password: String,
    name: Option<String>,
    email: Option<String>,
}

async fn complete_first_admin<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_token): Path<String>,
    headers: HeaderMap,
    Json(request): Json<FirstAdminRequest>,
) -> Result<Json<Value>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    require_portal_origin(&headers, &state.public_origin)?;
    let (token_hash, flow) = load_account_flow_by_token(state.service.repository(), &flow_token)
        .await?
        .ok_or_else(|| HttpError::gone("account_flow_expired"))?;
    if flow.state != AccountFlowState::Pending || flow.expires_at < now_ms()? {
        return Err(HttpError::conflict("account_flow_consumed"));
    }
    let now = now_ms()?;
    let request_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&request)
            .map_err(|_| HttpError::bad_request("invalid_account_flow"))?,
    )
    .map_err(|_| HttpError::bad_request("invalid_account_flow"))?;
    if flow.kind == super::AccountFlowKind::PasswordReset {
        let outcome = state
            .service
            .complete_password_reset(
                flow_token,
                flow.version,
                &request.password,
                now,
                idempotency(
                    &token_hash,
                    "account.password.reset",
                    flow.target_principal_id
                        .as_deref()
                        .unwrap_or("account-flow"),
                    &token_hash,
                    &request_digest,
                    now,
                )?,
                session_revocation_actions(
                    flow.target_principal_id
                        .as_deref()
                        .unwrap_or("account-flow"),
                    &token_hash,
                    now,
                    json!({
                        "principalId": flow.target_principal_id,
                        "reason": "password_reset",
                    }),
                ),
            )
            .await?;
        return match outcome {
            IdempotentOutcome::Applied(flow) => Ok(Json(json!({
                "status": "updated",
                "userId": flow.target_principal_id,
            }))),
            IdempotentOutcome::Replayed(_) => Err(HttpError::conflict("account_flow_consumed")),
        };
    }
    if flow.kind != super::AccountFlowKind::FirstAdmin {
        return Err(HttpError::bad_request("account_flow_provider_required"));
    }
    let username = request
        .username
        .clone()
        .ok_or_else(|| HttpError::bad_request("username_required"))?;
    let participant_id = flow.payload["participantId"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let participant_artifact_digest = flow.payload["participantArtifactDigest"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let participant_needs_digest = flow.payload["participantNeedsDigest"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let binding = state
        .service
        .repository()
        .get_participant_binding(participant_id, participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let grant_set = binding.resolve()?.proposal().required().grant_set().clone();
    let outcome = state
        .service
        .complete_first_admin(FirstAdminRegistration {
            token: flow_token,
            expected_flow_version: flow.version,
            username: username.clone(),
            password: request.password,
            display_name: request.name.unwrap_or(username),
            email: request.email,
            image_url: None,
            participant_id: participant_id.to_owned(),
            participant_artifact_digest: participant_artifact_digest.to_owned(),
            participant_needs_digest: participant_needs_digest.to_owned(),
            grant_set,
            authority_expires_at: None,
            completed_at: now,
            idempotency: idempotency(
                &token_hash,
                "first_admin.complete",
                "system:first-admin",
                &token_hash,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    match outcome {
        IdempotentOutcome::Applied(account) => Ok(Json(json!({
            "status": "created",
            "userId": account.principal.principal_id,
        }))),
        IdempotentOutcome::Replayed(_) => Err(HttpError::conflict("account_flow_consumed")),
    }
}

async fn local_login<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    headers: HeaderMap,
    Json(request): Json<LocalLoginRequest>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &request.flow_id).await?;
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if portal.disabled
        || portal.removed
        || !settings.local_login_enabled
        || !portal
            .provider_ids
            .iter()
            .any(|provider| provider == "local")
    {
        return Err(HttpError::forbidden("local_login_disabled"));
    }
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let now = now_ms()?;
    let principal = match state
        .service
        .authenticate_local(&request.username, &request.password, now)
        .await?
    {
        LocalAuthentication::Authenticated { principal, .. } => principal,
        LocalAuthentication::Denied => return Err(HttpError::unauthorized("invalid_credentials")),
    };
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Authenticated;
    flow.principal_id = Some(principal.principal_id);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::ApprovalRequired;
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let mut response = flow_response(flow);
    response.providers = vec!["local".to_owned()];
    Ok(Json(response))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct LocalRegistrationRequest {
    username: String,
    password: String,
    name: Option<String>,
    email: Option<String>,
    idempotency_key: String,
}

async fn register_local<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<LocalRegistrationRequest>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if portal.disabled
        || portal.removed
        || !portal.local_registration_enabled
        || !settings.local_login_enabled
        || !portal
            .provider_ids
            .iter()
            .any(|provider| provider == "local")
    {
        return Err(HttpError::forbidden("local_registration_disabled"));
    }
    let now = now_ms()?;
    let request_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&request)
            .map_err(|_| HttpError::bad_request("invalid_registration"))?,
    )
    .map_err(|_| HttpError::bad_request("invalid_registration"))?;
    let account = state
        .service
        .create_local_user(CreateLocalUserInput {
            username: request.username,
            password: request.password,
            name: request.name,
            email: request.email,
            created_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.local.register",
                &flow.session_public_key,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let principal_id = match account {
        IdempotentOutcome::Applied(account) => account.principal.principal_id,
        IdempotentOutcome::Replayed(value) => value
            .get("principalId")
            .and_then(Value::as_str)
            .ok_or_else(|| HttpError::internal("invalid_registration_replay"))?
            .to_owned(),
    };
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Authenticated;
    flow.principal_id = Some(principal_id);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::ApprovalRequired;
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    Ok(Json(flow_response(flow)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct OidcStartQuery {
    flow_id: String,
}

async fn start_oidc<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(provider_id): Path<String>,
    Query(query): Query<OidcStartQuery>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let flow = load_flow(&state.ephemeral, &query.flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    if portal.disabled || portal.removed || !portal.provider_ids.contains(&provider_id) {
        return Err(HttpError::forbidden("provider_not_allowed"));
    }
    begin_oidc(
        &state,
        provider_id,
        flow.flow_id,
        AuthOAuthKind::Browser,
        Some((&portal, &settings)),
    )
    .await
}

async fn start_account_flow_oidc<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path((flow_token, provider_id)): Path<(String, String)>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let Some((_, flow)) =
        load_account_flow_by_token(state.service.repository(), &flow_token).await?
    else {
        return Err(HttpError::gone("account_flow_expired"));
    };
    if !matches!(
        flow.kind,
        super::AccountFlowKind::IdentityLink | super::AccountFlowKind::FirstAdmin
    ) || flow.state != AccountFlowState::Pending
        || flow.expires_at < now_ms()?
        || flow
            .target_provider_id
            .as_ref()
            .is_some_and(|target| target != &provider_id)
    {
        return Err(HttpError::conflict("account_flow_not_eligible"));
    }
    let portal = if flow.kind == super::AccountFlowKind::FirstAdmin {
        let portal = state
            .service
            .repository()
            .get_login_portal("builtin")
            .await?
            .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
        if portal.0.disabled || portal.0.removed || !portal.0.provider_ids.contains(&provider_id) {
            return Err(HttpError::forbidden("provider_not_allowed"));
        }
        Some(portal)
    } else {
        None
    };
    begin_oidc(
        &state,
        provider_id,
        flow_token,
        AuthOAuthKind::AccountFlow,
        portal.as_ref().map(|(portal, settings)| (portal, settings)),
    )
    .await
}

async fn begin_oidc<R, E>(
    state: &AuthHttpState<R, E>,
    provider_id: String,
    flow_id: String,
    kind: AuthOAuthKind,
    portal_policy: Option<(&LoginPortalRecord, &LoginSettingsRecord)>,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let provider = state
        .oidc_providers
        .get(&provider_id)
        .ok_or_else(|| HttpError::not_found("provider_not_found"))?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        provider.client_id.clone(),
        provider.client_secret.clone(),
    )
    .set_redirect_uri(provider.redirect_uri.clone());
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut authorization = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge);
    for scope in &provider.scopes {
        authorization = authorization.add_scope(scope.clone());
    }
    let (authorization_url, csrf, nonce) = authorization.url();
    let now = now_ms()?;
    let mut browser_binding = [0_u8; 32];
    getrandom::fill(&mut browser_binding)
        .map_err(|_| HttpError::internal("oauth_browser_binding_generation"))?;
    let browser_binding = URL_SAFE_NO_PAD.encode(browser_binding);
    let browser_binding_digest = URL_SAFE_NO_PAD.encode(Sha256::digest(&browser_binding));
    let (portal_id, portal_policy_digest) = match portal_policy {
        Some((portal, settings)) => (
            Some(portal.portal_id.clone()),
            Some(oidc_portal_policy_digest(portal, settings)?),
        ),
        None => (None, None),
    };
    state
        .ephemeral
        .create_oauth_state(AuthOAuthState {
            format: "trellis.auth-oauth-state.v1".to_owned(),
            state_id: csrf.secret().clone(),
            provider_id: provider_id.clone(),
            kind,
            flow_id,
            status: AuthOAuthStatus::Pending,
            pkce_verifier: pkce_verifier.secret().clone(),
            nonce: nonce.secret().clone(),
            redirect_uri: provider.redirect_uri.as_str().to_owned(),
            browser_binding_digest,
            portal_id,
            portal_policy_digest,
            claim_owner: None,
            result_digest: None,
            created_at: now,
            expires_at: checked_add(now, 15 * 60_000)?,
            version: 1,
        })
        .await?;
    let mut response = Redirect::temporary(authorization_url.as_str()).into_response();
    response.headers_mut().append(
        SET_COOKIE,
        oauth_cookie_header(
            &oauth_cookie_name(csrf.secret()),
            &browser_binding,
            &provider_id,
            state.public_origin.starts_with("https://"),
            15 * 60,
        )?,
    );
    Ok(response)
}

#[derive(Deserialize)]
struct OidcCallbackQuery {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

async fn oidc_callback<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(provider_id): Path<String>,
    Query(query): Query<OidcCallbackQuery>,
    headers: HeaderMap,
) -> Response
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let cookie_name = oauth_cookie_name(&query.state);
    let mut response = match oidc_callback_inner(&state, &provider_id, query, &headers).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    };
    if let Ok(cookie) = oauth_cookie_header(
        &cookie_name,
        "",
        &provider_id,
        state.public_origin.starts_with("https://"),
        0,
    ) {
        response.headers_mut().append(SET_COOKIE, cookie);
    }
    response
}

async fn oidc_callback_inner<R, E>(
    state: &AuthHttpState<R, E>,
    provider_id: &str,
    query: OidcCallbackQuery,
    headers: &HeaderMap,
) -> Result<Response, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let pending = state
        .ephemeral
        .get_oauth_state(&query.state)
        .await?
        .ok_or_else(|| HttpError::bad_request("oauth_state_invalid"))?;
    if pending.status != AuthOAuthStatus::Pending
        || pending.provider_id != provider_id
        || pending.expires_at < now_ms()?
    {
        return Err(HttpError::bad_request("oauth_state_invalid"));
    }
    require_oauth_browser_binding(&query.state, &pending, headers)?;
    let provider = state
        .oidc_providers
        .get(provider_id)
        .ok_or_else(|| HttpError::not_found("provider_not_found"))?;
    if pending.redirect_uri != provider.redirect_uri.as_str() {
        return Err(HttpError::bad_request("oauth_state_invalid"));
    }
    let federated_registration_enabled = if let Some(portal_id) = pending.portal_id.as_deref() {
        let (portal, settings) = state
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
        if portal.disabled
            || portal.removed
            || !portal.provider_ids.iter().any(|value| value == provider_id)
            || pending.portal_policy_digest.as_deref()
                != Some(oidc_portal_policy_digest(&portal, &settings)?.as_str())
        {
            return Err(HttpError::conflict("oauth_policy_changed"));
        }
        settings.federated_registration_enabled
    } else {
        false
    };
    match pending.kind {
        AuthOAuthKind::Browser => {
            let flow = load_flow(&state.ephemeral, &pending.flow_id).await?;
            if flow.state != AuthBrowserFlowState::ChooseProvider
                || pending.portal_id.as_deref() != Some(flow.portal_id.as_str())
            {
                return Err(HttpError::conflict("oauth_flow_changed"));
            }
        }
        AuthOAuthKind::AccountFlow => {
            let Some((_, flow)) =
                load_account_flow_by_token(state.service.repository(), &pending.flow_id).await?
            else {
                return Err(HttpError::gone("account_flow_expired"));
            };
            if !matches!(
                flow.kind,
                super::AccountFlowKind::IdentityLink | super::AccountFlowKind::FirstAdmin
            ) || flow.state != AccountFlowState::Pending
                || flow.expires_at < now_ms()?
                || flow
                    .target_provider_id
                    .as_deref()
                    .is_some_and(|target| target != provider_id)
                || (flow.kind == super::AccountFlowKind::FirstAdmin
                    && pending.portal_id.as_deref() != Some("builtin"))
                || (flow.kind == super::AccountFlowKind::IdentityLink
                    && pending.portal_id.is_some())
            {
                return Err(HttpError::conflict("oauth_flow_changed"));
            }
        }
    }
    let claim_owner = format!("callback_{}", URL_SAFE_NO_PAD.encode(getrandom_bytes()?));
    let mut oauth = claim_oauth_state(&state.ephemeral, &query.state, &claim_owner).await?;
    if query.error.is_some() {
        let expected = oauth.version;
        oauth.status = AuthOAuthStatus::Expired;
        oauth.version += 1;
        state.ephemeral.replace_oauth_state(expected, oauth).await?;
        return Err(HttpError::bad_request("oauth_denied"));
    }
    let code = query
        .code
        .ok_or_else(|| HttpError::bad_request("oauth_code_missing"))?;
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::ExchangeStarted;
    oauth.version += 1;
    state
        .ephemeral
        .replace_oauth_state(expected, oauth.clone())
        .await?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        provider.client_id.clone(),
        provider.client_secret.clone(),
    )
    .set_redirect_uri(provider.redirect_uri.clone());
    let http_client = openidconnect::reqwest::ClientBuilder::new()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| HttpError::internal("oauth_http_client"))?;
    let token = match client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|_| HttpError::bad_request("oauth_exchange_invalid"))?
        .set_pkce_verifier(PkceCodeVerifier::new(oauth.pkce_verifier.clone()))
        .request_async(&http_client)
        .await
    {
        Ok(token) => token,
        Err(_) => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::bad_gateway("oauth_exchange_failed"));
        }
    };
    let id_token = match token.id_token() {
        Some(id_token) => id_token,
        None => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::bad_gateway("oauth_id_token_missing"));
        }
    };
    let verifier = client.id_token_verifier();
    let claims = match id_token.claims(&verifier, &Nonce::new(oauth.nonce.clone())) {
        Ok(claims) => claims,
        Err(_) => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("oauth_id_token_invalid"));
        }
    };
    if let Some(expected) = claims.access_token_hash() {
        let actual = AccessTokenHash::from_token(
            token.access_token(),
            id_token
                .signing_alg()
                .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?,
            id_token
                .signing_key(&verifier)
                .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?,
        )
        .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?;
        if actual != *expected {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("oauth_access_token_mismatch"));
        }
    }
    let subject = claims.subject().as_str().to_owned();
    let now = now_ms()?;
    if oauth.kind == AuthOAuthKind::AccountFlow {
        let Some((token_hash, flow)) =
            load_account_flow_by_token(state.service.repository(), &oauth.flow_id).await?
        else {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::gone("account_flow_expired"));
        };
        if !matches!(
            flow.kind,
            super::AccountFlowKind::IdentityLink | super::AccountFlowKind::FirstAdmin
        ) || flow.state != AccountFlowState::Pending
            || flow.expires_at < now
            || flow
                .target_provider_id
                .as_ref()
                .is_some_and(|target| target != provider_id)
        {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::conflict("account_flow_not_eligible"));
        }
        if flow.kind == super::AccountFlowKind::FirstAdmin {
            if state
                .service
                .repository()
                .get_provider_identity(provider_id, &subject)
                .await?
                .is_some()
            {
                mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
                return Err(HttpError::unauthorized("federated_login_denied"));
            }
            let participant_id = flow.payload["participantId"]
                .as_str()
                .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
            let participant_artifact_digest = flow.payload["participantArtifactDigest"]
                .as_str()
                .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
            let participant_needs_digest = flow.payload["participantNeedsDigest"]
                .as_str()
                .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
            let binding = state
                .service
                .repository()
                .get_participant_binding(participant_id, participant_artifact_digest)
                .await?
                .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
            if binding.needs_digest != participant_needs_digest {
                return Err(HttpError::conflict("first_admin_target_changed"));
            }
            let grant_set = binding.resolve()?.proposal().required().grant_set().clone();
            let digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
            let outcome = state
                .service
                .complete_first_admin_federated(FirstAdminFederatedRegistration {
                    token: oauth.flow_id.clone(),
                    expected_flow_version: flow.version,
                    provider: provider_id.to_owned(),
                    provider_subject: subject,
                    display_name: None,
                    email: claims.email().map(|email| email.as_str().to_owned()),
                    image_url: None,
                    participant_id: participant_id.to_owned(),
                    participant_artifact_digest: participant_artifact_digest.to_owned(),
                    participant_needs_digest: participant_needs_digest.to_owned(),
                    grant_set,
                    authority_expires_at: None,
                    completed_at: now,
                    idempotency: idempotency(
                        &token_hash,
                        "first_admin.federated.complete",
                        "system:first-admin",
                        &oauth.state_id,
                        &digest,
                        now,
                    )?,
                    actions: Vec::new(),
                })
                .await?;
            let principal_id = match outcome {
                IdempotentOutcome::Applied(account) => account.principal.principal_id,
                IdempotentOutcome::Replayed(_) => {
                    return Err(HttpError::conflict("account_flow_consumed"));
                }
            };
            let expected = oauth.version;
            oauth.status = AuthOAuthStatus::Completed;
            oauth.result_digest = Some(digest_parts(&[&principal_id]));
            oauth.version += 1;
            state.ephemeral.replace_oauth_state(expected, oauth).await?;
            return Ok(Redirect::temporary(&format!(
                "{}/_trellis/portal/account/complete",
                state.public_origin.trim_end_matches('/')
            ))
            .into_response());
        }
        let principal_id = flow
            .target_principal_id
            .clone()
            .ok_or_else(|| HttpError::internal("identity_link_target_missing"))?;
        if state
            .service
            .repository()
            .get_provider_identity(provider_id, &subject)
            .await?
            .is_some()
        {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::conflict("provider_identity_already_linked"));
        }
        let digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
        let outcome = state
            .service
            .complete_identity_link(CompleteIdentityLinkInput {
                token: oauth.flow_id.clone(),
                expected_flow_version: flow.version,
                identity: super::ProviderIdentityLink {
                    provider: provider_id.to_owned(),
                    provider_subject: subject,
                    principal_id: principal_id.clone(),
                    linked_at: now,
                    last_seen_at: now,
                },
                completed_at: now,
                idempotency: idempotency(
                    &token_hash,
                    "account.identity.link",
                    &principal_id,
                    &oauth.state_id,
                    &digest,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        if matches!(outcome, IdempotentOutcome::Replayed(_)) {
            return Err(HttpError::conflict("account_flow_consumed"));
        }
        let expected = oauth.version;
        oauth.status = AuthOAuthStatus::Completed;
        oauth.result_digest = Some(digest_parts(&[&principal_id]));
        oauth.version += 1;
        state.ephemeral.replace_oauth_state(expected, oauth).await?;
        return Ok(Redirect::temporary(&format!(
            "{}/_trellis/portal/account/complete",
            state.public_origin.trim_end_matches('/')
        ))
        .into_response());
    }
    let principal_id = if let Some(identity) = state
        .service
        .repository()
        .get_provider_identity(provider_id, &subject)
        .await?
    {
        let principal = state
            .service
            .repository()
            .get_principal(&identity.principal_id)
            .await?
            .ok_or_else(|| HttpError::internal("provider_principal_missing"))?;
        if principal.state != super::PrincipalState::Active {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::forbidden("account_inactive"));
        }
        principal.principal_id
    } else {
        if !federated_registration_enabled {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("federated_login_denied"));
        }
        let claim_digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
        match state
            .service
            .create_federated_user(CreateFederatedUserInput {
                provider: provider_id.to_owned(),
                provider_subject: subject,
                name: None,
                email: claims.email().map(|email| email.as_str().to_owned()),
                image: None,
                created_at: now,
                idempotency: idempotency(
                    &oauth.state_id,
                    "oauth.account.bind",
                    provider_id,
                    &oauth.state_id,
                    &claim_digest,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?
        {
            IdempotentOutcome::Applied(account) => account.principal.principal_id,
            IdempotentOutcome::Replayed(value) => value
                .get("principalId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_account_replay"))?
                .to_owned(),
        }
    };
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::Completed;
    oauth.result_digest = Some(digest_parts(&[&principal_id]));
    oauth.version += 1;
    state
        .ephemeral
        .replace_oauth_state(expected, oauth.clone())
        .await?;
    let mut flow = load_flow(&state.ephemeral, &oauth.flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Authenticated;
    flow.principal_id = Some(principal_id);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::ApprovalRequired;
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    Ok(Redirect::temporary(&format!(
        "{}/_trellis/portal/auth?flowId={}",
        state.public_origin.trim_end_matches('/'),
        flow.flow_id
    ))
    .into_response())
}

async fn mark_oauth_restart_required(
    repository: &impl AuthEphemeralRepository,
    oauth: &mut AuthOAuthState,
) -> Result<(), HttpError> {
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::RestartRequired;
    oauth.version += 1;
    repository
        .replace_oauth_state(expected, oauth.clone())
        .await
        .map_err(Into::into)
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct ApprovalRequest {
    approved: bool,
    consent_view_digest: String,
    #[serde(default)]
    selected_optional_bundles: Vec<String>,
    idempotency_key: String,
}

async fn decide_approval<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ApprovalRequest>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &flow_id).await?;
    let (portal, _) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if flow.state != AuthBrowserFlowState::ApprovalRequired {
        return Err(HttpError::conflict("flow_not_awaiting_approval"));
    }
    let now = now_ms()?;
    let binding = state
        .service
        .repository()
        .get_participant_binding(&flow.participant_id, &flow.participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::internal("participant_binding_missing"))?;
    let current_consent = browser_consent(&binding)?;
    if request.consent_view_digest != flow.consent.consent_view_digest
        || current_consent != flow.consent
    {
        return Err(HttpError::conflict("consent_view_changed"));
    }
    if !request.approved {
        let expected = flow.version;
        flow.state = AuthBrowserFlowState::ApprovalDenied;
        flow.completed_at = Some(now);
        flow.version += 1;
        state
            .ephemeral
            .replace_browser_flow(expected, flow.clone())
            .await?;
        return Ok(Json(flow_response(flow)));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let current = state
        .service
        .repository()
        .get_identity_authority(&principal_id, &flow.participant_id)
        .await?;
    let (grant_set, capabilities, selected_optional_bundles) =
        select_browser_authority(&flow.consent, &request.selected_optional_bundles)?;
    let requests_reserved = capabilities
        .iter()
        .any(|capability| matches!(capability.as_str(), "admin" | "provision" | "activate"));
    if requests_reserved
        && (flow.participant_artifact_digest != administration_participant_digest()?
            || current.as_ref().is_none_or(|authority| {
                capabilities
                    .iter()
                    .any(|capability| !authority.desired_capabilities.contains(capability))
            }))
    {
        return Err(HttpError::forbidden("reserved_capability"));
    }
    let authority_id = current.as_ref().map_or_else(
        || {
            format!(
                "ida_{}",
                digest_parts(&[&principal_id, &flow.participant_id])
            )
        },
        |authority| authority.authority_id.clone(),
    );
    let request_value =
        serde_json::to_value(&request).map_err(|_| HttpError::bad_request("invalid_approval"))?;
    let request_digest = trellis_protocol::digest_json(&request_value)
        .map_err(|_| HttpError::bad_request("invalid_approval"))?;
    let signer_id =
        super::domain::validate_ed25519_public_key("sessionPublicKey", &flow.session_public_key)?;
    let proposal = state
        .service
        .create_authority_proposal(CreateAuthorityProposalInput {
            authority_kind: AuthorityKind::Identity,
            authority_id: authority_id.clone(),
            deployment_id: None,
            proposal_kind: if current.is_some() {
                AuthorityProposalKind::Update
            } else {
                AuthorityProposalKind::Initial
            },
            participant_id: flow.participant_id.clone(),
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
            grant_set: grant_set.clone(),
            capabilities: capabilities.clone(),
            base_authority_version: current.as_ref().map(|authority| authority.version),
            payload: json!({
                "source": "browser_approval",
                "flowId": flow_id,
                "consentViewDigest": flow.consent.consent_view_digest,
                "proposalDigest": flow.consent.proposal_digest,
                "selectedOptionalBundles": selected_optional_bundles,
                "baseAuthorityVersion": current.as_ref().map(|authority| authority.version),
            }),
            created_at: now,
            expires_at: current.as_ref().and_then(|authority| authority.expires_at),
            idempotency: idempotency(
                &flow_id,
                "browser.authority.propose",
                &signer_id,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let proposal_id = match proposal {
        IdempotentOutcome::Applied(proposal) => proposal.proposal_id,
        IdempotentOutcome::Replayed(value) => value
            .get("proposalId")
            .and_then(Value::as_str)
            .ok_or_else(|| HttpError::internal("invalid_proposal_replay"))?
            .to_owned(),
    };
    let (proposal, _) = state
        .service
        .repository()
        .get_authority_proposal(&proposal_id)
        .await?
        .ok_or_else(|| HttpError::internal("proposal_missing"))?;
    let desired = DesiredAuthorityRecord::Identity(IdentityAuthorityRecord {
        authority_id: authority_id.clone(),
        principal_id,
        participant_id: flow.participant_id.clone(),
        participant_artifact_digest: flow.participant_artifact_digest.clone(),
        accepted_needs_digest: flow.participant_needs_digest.clone(),
        desired_grant_set: grant_set,
        desired_capabilities: capabilities,
        state: AuthorityState::Accepted,
        version: current
            .as_ref()
            .map_or(1, |authority| authority.version + 1),
        created_at: current
            .as_ref()
            .map_or(now, |authority| authority.created_at),
        updated_at: now,
        expires_at: current.as_ref().and_then(|authority| authority.expires_at),
        decision: Some(AuthorityDecision {
            decided_at: now,
            decided_by: flow.session_public_key.clone(),
            reason: None,
        }),
    });
    let durable_result_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&desired).map_err(|_| HttpError::internal("authority_encode"))?,
    )
    .map_err(|_| HttpError::internal("authority_digest"))?;
    state
        .service
        .decide_authority_proposal(DecideAuthorityProposalInput {
            proposal_id,
            expected_version: proposal.version,
            expected_base_authority_version: None,
            outcome: AuthorityDecisionOutcome::Accepted,
            decided_by: flow.session_public_key.clone(),
            reason: None,
            desired_authority: Some(desired),
            decided_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.authority.accept",
                &signer_id,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let authority_target =
        AuthorityTarget::new(AuthorityKind::Identity, authority_id).map_err(HttpError::from)?;
    super::ensure_authority_dependencies(
        state.service.repository(),
        AuthorityEvidenceScope {
            target: authority_target.clone(),
            participant_id: flow.participant_id.clone(),
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
        },
        &binding,
        now,
    )
    .await?;
    state
        .service
        .authorization()
        .reconcile_authority(&authority_target, now)
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Approved;
    flow.durable_result_digest = Some(durable_result_digest);
    flow.completed_at = Some(now);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    Ok(Json(flow_response(flow)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct BindRequest {
    idempotency_key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BindResponse {
    server_now: i64,
    session: SessionRecord,
    nats: NatsBootstrapResponse,
    authorization_context: super::AuthorizationContextBundle,
    redirect_target: Option<String>,
}

#[derive(Clone, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct NatsBootstrapResponse {
    jwt: String,
    jwt_expires_at: i64,
    servers: Vec<String>,
}

async fn bind_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<BindRequest>,
) -> Result<Json<BindResponse>, HttpError>
where
    R: AuthHttpRepository,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &flow_id).await?;
    let (portal, _) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if !matches!(
        flow.state,
        AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
    ) {
        return Err(HttpError::conflict("flow_not_approved"));
    }
    let binding = state
        .service
        .repository()
        .get_participant_binding(&flow.participant_id, &flow.participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::conflict("participant_unavailable"))?;
    if binding.needs_digest != flow.participant_needs_digest {
        return Err(HttpError::conflict("participant_needs_changed"));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let now = now_ms()?;
    let signer_id =
        super::domain::validate_ed25519_public_key("sessionPublicKey", &flow.session_public_key)?;
    let digest = digest_parts(&[&flow_id, &request.idempotency_key]);
    let outcome = state
        .service
        .create_session(CreateSessionInput {
            principal_id,
            principal_kind: PrincipalKind::User,
            participant_id: flow.participant_id.clone(),
            participant_kind: binding.participant_kind,
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
            session_public_key: flow.session_public_key.clone(),
            deployment_id: None,
            instance_id: None,
            desired_authority: None,
            created_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.session.bind",
                &signer_id,
                &request.idempotency_key,
                &digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let session = match outcome {
        IdempotentOutcome::Applied(session) => session,
        IdempotentOutcome::Replayed(value) => {
            let session_id = value
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_session_replay"))?;
            state
                .service
                .repository()
                .get_session(session_id)
                .await?
                .ok_or_else(|| HttpError::internal("session_missing"))?
        }
    };
    if flow.state == AuthBrowserFlowState::Approved {
        let expected = flow.version;
        flow.state = AuthBrowserFlowState::Consumed;
        flow.claim_owner = Some(session.session_id.clone());
        flow.claimed_at = Some(now);
        flow.version += 1;
        state
            .ephemeral
            .replace_browser_flow(expected, flow.clone())
            .await?;
    }
    let issuance = state
        .service
        .authorization()
        .resolve_issuable_state(&session.session_id, now)
        .await
        .map_err(map_issuance_error)?;
    let expires_at = [
        issuance.session_expires_at,
        issuance.effective_authority_expires_at,
    ]
    .into_iter()
    .flatten()
    .min()
    .ok_or_else(|| HttpError::internal("credential_expiry_missing"))?;
    let route =
        state
            .issuer
            .deny_all_user_jwt(&flow.session_nkey, expires_at / 1_000, now / 1_000)?;
    let authorization_context = state
        .authorization_contexts
        .issue(
            super::AuthorizationContextIssueRequest {
                session_id: session.session_id.clone(),
                request_id: request.idempotency_key,
                request_digest: digest,
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    Ok(Json(BindResponse {
        server_now: now,
        session,
        nats: NatsBootstrapResponse {
            jwt: route.jwt,
            jwt_expires_at: route.expires_at,
            servers: state.websocket_nats_servers,
        },
        authorization_context,
        redirect_target: flow.redirect_target,
    }))
}

fn flow_response(flow: AuthBrowserFlow) -> BrowserFlowResponse {
    BrowserFlowResponse {
        flow_id: flow.flow_id,
        state: flow.state,
        expires_at: flow.expires_at,
        providers: Vec::new(),
        registration_enabled: false,
        federated_registration_enabled: false,
        consent_view: flow.consent.consent_view,
        consent_view_digest: flow.consent.consent_view_digest,
        user: None,
        redirect_target: flow.redirect_target,
    }
}

fn browser_consent(
    binding: &ParticipantBindingRecord,
) -> Result<BrowserConsentProposal, HttpError> {
    let resolved = binding.resolve()?;
    let proposal = resolved.proposal();
    let participant: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|_| HttpError::internal("consent_encode"))?;
    let optional_grant_bundles = resolved
        .optional_apis()
        .iter()
        .map(|used| (used.alias().to_owned(), used.grant_set().clone()))
        .collect();
    let optional_capability_definitions = proposal
        .optional()
        .capabilities()
        .iter()
        .map(|capability| {
            (
                format!("{}::{}", capability.api(), capability.name()),
                GrantSetV1::new(capability.allows().to_vec()),
            )
        })
        .collect();
    let consent_view = json!({
        "participant": {
            "id": binding.participant_id,
            "digest": binding.artifact_digest,
            "displayName": participant.get("displayName").and_then(Value::as_str).unwrap_or(&binding.participant_id),
            "description": participant.get("description").and_then(Value::as_str).unwrap_or("Trellis participant"),
        },
        "required": {
            "permissions": proposal.required().grant_set().permissions(),
            "capabilities": proposal.required().capabilities().iter().map(|capability| format!("{}::{}", capability.api(), capability.name())).collect::<Vec<_>>(),
        },
        "optionalBundles": resolved.optional_apis().iter().map(|used| json!({
            "id": used.alias(),
            "api": used.api(),
            "apiDigest": used.api_digest(),
            "permissions": used.grant_set().permissions(),
        })).collect::<Vec<_>>(),
    });
    BrowserConsentProposal::new(
        binding.participant_id.clone(),
        binding.artifact_digest.clone(),
        binding.needs_digest.clone(),
        consent_view,
        proposal.required().grant_set().clone(),
        optional_grant_bundles,
        proposal
            .required()
            .capabilities()
            .iter()
            .map(|capability| capability.name().to_owned())
            .collect(),
        optional_capability_definitions,
    )
    .map_err(Into::into)
}

fn select_browser_authority(
    consent: &BrowserConsentProposal,
    selected_optional_bundles: &[String],
) -> Result<(GrantSetV1, Vec<String>, BTreeSet<String>), HttpError> {
    let selected = selected_optional_bundles
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    if selected.len() != selected_optional_bundles.len() {
        return Err(HttpError::bad_request("duplicate_optional_bundle"));
    }
    let mut permissions = consent.required_grant_set.permissions().to_vec();
    for bundle_id in &selected {
        let bundle = consent
            .optional_grant_bundles
            .get(bundle_id)
            .ok_or_else(|| HttpError::bad_request("unknown_optional_bundle"))?;
        permissions.extend_from_slice(bundle.permissions());
    }
    let grant_set = GrantSetV1::new(permissions);
    let mut capabilities = consent.required_capabilities.clone();
    for (qualified_name, definition) in &consent.optional_capability_definitions {
        if definition
            .permissions()
            .iter()
            .all(|permission| grant_set.permissions().contains(permission))
        {
            let name = qualified_name
                .rsplit_once("::")
                .map_or(qualified_name.as_str(), |(_, name)| name);
            capabilities.push(name.to_owned());
        }
    }
    capabilities.sort();
    capabilities.dedup();
    Ok((grant_set, capabilities, selected))
}

async fn load_flow(
    repository: &impl AuthEphemeralRepository,
    flow_id: &str,
) -> Result<AuthBrowserFlow, HttpError> {
    let flow = repository
        .get_browser_flow(flow_id)
        .await?
        .ok_or_else(|| HttpError::not_found("flow_not_found"))?;
    if flow.expires_at < now_ms()? && flow.state != AuthBrowserFlowState::Expired {
        let expected = flow.version;
        let mut expired = flow;
        expired.state = AuthBrowserFlowState::Expired;
        expired.completed_at = Some(now_ms()?);
        expired.version += 1;
        repository.replace_browser_flow(expected, expired).await?;
        return Err(HttpError::gone("flow_expired"));
    }
    Ok(flow)
}

fn idempotency(
    scope: &str,
    purpose: &str,
    signer_id: &str,
    request_id: &str,
    request_digest: &str,
    now: i64,
) -> Result<IdempotencyResultRecord, HttpError> {
    Ok(IdempotencyResultRecord {
        scope_key: digest_parts(&[scope, purpose, signer_id, request_id]),
        purpose: purpose.to_owned(),
        signer_id: signer_id.to_owned(),
        request_id: request_id.to_owned(),
        request_digest: request_digest.to_owned(),
        result: json!({}),
        created_at: now,
        expires_at: checked_add(now, IDEMPOTENCY_TTL_MS)?,
    })
}

fn validate_redirect(redirect: &str, allowed_origins: &[String]) -> Result<(), HttpError> {
    let redirect = Url::parse(redirect).map_err(|_| HttpError::bad_request("invalid_redirect"))?;
    if redirect.fragment().is_some() || redirect.username() != "" || redirect.password().is_some() {
        return Err(HttpError::bad_request("invalid_redirect"));
    }
    if !allowed_origins.contains(&canonical_origin(redirect.as_str())?) {
        return Err(HttpError::bad_request("redirect_origin_not_allowed"));
    }
    Ok(())
}

fn canonical_origin(value: &str) -> Result<String, HttpError> {
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

fn oidc_portal_policy_digest(
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

fn oauth_cookie_name(state_id: &str) -> String {
    format!("trellis_oauth_{}", digest_parts(&[state_id]))
}

fn oauth_cookie_header(
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

fn require_oauth_browser_binding(
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

fn require_portal_origin(headers: &HeaderMap, public_origin: &str) -> Result<(), HttpError> {
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

fn require_selected_portal_origin(
    headers: &HeaderMap,
    portal: &LoginPortalRecord,
    public_origin: &str,
) -> Result<(), HttpError> {
    require_portal_origin(
        headers,
        portal.entry_url.as_deref().unwrap_or(public_origin),
    )
}

fn now_ms() -> Result<i64, HttpError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|_| HttpError::internal("clock_before_epoch"))?
        .as_millis()
        .try_into()
        .map_err(|_| HttpError::internal("clock_overflow"))
}

#[allow(clippy::result_large_err)]
fn administration_participant_digest() -> Result<String, HttpError> {
    let value: Value = serde_json::from_str(include_str!(
        "../../../../trellis/artifacts/trellis.admin.participant.json"
    ))
    .map_err(|_| HttpError::internal("invalid_administration_participant"))?;
    parse_participant_v1(&value)
        .and_then(|participant| participant.digest())
        .map_err(|_| HttpError::internal("invalid_administration_participant"))
}

fn checked_add(value: i64, duration: i64) -> Result<i64, HttpError> {
    value
        .checked_add(duration)
        .filter(|value| *value <= super::MAX_PROTOCOL_INTEGER as i64)
        .ok_or_else(|| HttpError::internal("timestamp_overflow"))
}

fn digest_parts(parts: &[&str]) -> String {
    let mut digest = Sha256::new();
    for part in parts {
        digest.update((part.len() as u64).to_be_bytes());
        digest.update(part.as_bytes());
    }
    URL_SAFE_NO_PAD.encode(digest.finalize())
}

fn session_revocation_actions(
    scope: &str,
    request_id: &str,
    now: i64,
    payload: Value,
) -> Vec<PostCommitActionRecord> {
    [
        (PostCommitActionKind::Event, "event"),
        (PostCommitActionKind::Kick, "kick"),
    ]
    .into_iter()
    .map(|(kind, suffix)| PostCommitActionRecord {
        action_id: digest_parts(&[scope, request_id, suffix]),
        kind,
        payload: payload.clone(),
        created_at: now,
        attempts: 0,
        next_attempt_at: now,
        claimed_until: None,
        last_error: None,
    })
    .collect()
}

#[allow(clippy::result_large_err)]
fn proof_request_digest(raw: &Value) -> Result<String, trellis_protocol::ProtocolError> {
    session_proof_request_digest_v1(raw)
}

fn first_admin_token_hash(token: &str) -> Result<String, HttpError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(token)
        .map_err(|_| HttpError::bad_request("invalid_account_flow"))?;
    if decoded.len() != 32 || URL_SAFE_NO_PAD.encode(&decoded) != token {
        return Err(HttpError::bad_request("invalid_account_flow"));
    }
    Ok(URL_SAFE_NO_PAD.encode(Sha256::digest(decoded)))
}

async fn load_account_flow_by_token(
    repository: &impl AccountFlowRepository,
    token: &str,
) -> Result<Option<(String, super::AccountFlowRecord)>, HttpError> {
    let mut hashes = vec![URL_SAFE_NO_PAD.encode(Sha256::digest(token.as_bytes()))];
    if let Ok(hash) = first_admin_token_hash(token) {
        hashes.insert(0, hash);
    }
    for hash in hashes {
        if let Some(flow) = repository.get_account_flow_by_hash(&hash).await? {
            return Ok(Some((hash, flow)));
        }
    }
    Ok(None)
}

fn getrandom_bytes() -> Result<[u8; 16], HttpError> {
    let mut bytes = [0_u8; 16];
    getrandom::fill(&mut bytes).map_err(|_| HttpError::internal("entropy_unavailable"))?;
    Ok(bytes)
}

async fn security_headers(request: Request<axum::body::Body>, next: Next) -> Response {
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

#[derive(Debug)]
struct HttpError {
    status: StatusCode,
    code: &'static str,
}

impl HttpError {
    fn bad_request(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_REQUEST,
            code,
        }
    }
    fn unauthorized(code: &'static str) -> Self {
        Self {
            status: StatusCode::UNAUTHORIZED,
            code,
        }
    }
    fn forbidden(code: &'static str) -> Self {
        Self {
            status: StatusCode::FORBIDDEN,
            code,
        }
    }
    fn not_found(code: &'static str) -> Self {
        Self {
            status: StatusCode::NOT_FOUND,
            code,
        }
    }
    fn conflict(code: &'static str) -> Self {
        Self {
            status: StatusCode::CONFLICT,
            code,
        }
    }
    fn gone(code: &'static str) -> Self {
        Self {
            status: StatusCode::GONE,
            code,
        }
    }
    fn bad_gateway(code: &'static str) -> Self {
        Self {
            status: StatusCode::BAD_GATEWAY,
            code,
        }
    }
    fn internal(code: &'static str) -> Self {
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

fn map_issuance_error(error: AuthorizationStateError) -> HttpError {
    tracing::warn!(%error, "auth issuance denied");
    if error.is_expected_denial() {
        HttpError::unauthorized("auth_required")
    } else {
        error.into()
    }
}

fn project_service_resource_bindings(
    participant_json: &str,
    evidence: &[ResourceBindingEvidence],
    participant_id: &str,
) -> Result<ServiceResourceBindings, HttpError> {
    let participant: Value = serde_json::from_str(participant_json)
        .map_err(|_| HttpError::internal("participant_artifact_invalid"))?;
    let mut resources = ServiceResourceBindings::default();
    let mut job_queues = BTreeMap::new();
    let mut jobs_namespace = None;
    let mut jobs_work_stream = None;

    for binding in evidence {
        match &binding.provider_identity {
            ResourceProviderIdentity::Kv { bucket } => {
                let config = participant_resource(&participant, "kv", &binding.local_name)?;
                resources.kv.insert(
                    binding.local_name.clone(),
                    KvResourceBinding {
                        bucket: bucket.clone(),
                        history: config.get("history").and_then(Value::as_i64).unwrap_or(1),
                        max_value_bytes: optional_i64(config, "maxValueBytes")?,
                        ttl_ms: config.get("ttlMs").and_then(Value::as_i64).unwrap_or(0),
                    },
                );
            }
            ResourceProviderIdentity::Store { bucket } => {
                let config = participant_resource(&participant, "store", &binding.local_name)?;
                resources.store.insert(
                    binding.local_name.clone(),
                    StoreResourceBinding {
                        name: bucket.clone(),
                        max_object_bytes: optional_i64(config, "maxObjectBytes")?,
                        max_total_bytes: optional_i64(config, "maxTotalBytes")?,
                        ttl_ms: config.get("ttlMs").and_then(Value::as_i64).unwrap_or(0),
                    },
                );
            }
            ResourceProviderIdentity::State { .. } => {}
            ResourceProviderIdentity::JobQueue {
                namespace,
                work_stream,
                publish_prefix,
                updates_prefix,
                work_subject,
                consumer,
            } => {
                let config = participant
                    .get("jobQueues")
                    .and_then(Value::as_object)
                    .and_then(|queues| queues.get(&binding.local_name))
                    .ok_or_else(|| HttpError::internal("job_queue_binding_invalid"))?;
                if jobs_namespace
                    .as_ref()
                    .is_some_and(|current| current != namespace)
                    || jobs_work_stream
                        .as_ref()
                        .is_some_and(|current| current != work_stream)
                {
                    return Err(HttpError::internal("job_resource_identity_mismatch"));
                }
                jobs_namespace = Some(namespace.clone());
                jobs_work_stream = Some(work_stream.clone());
                job_queues.insert(
                    binding.local_name.clone(),
                    JobsQueueResourceBinding {
                        queue_type: binding.local_name.clone(),
                        publish_prefix: publish_prefix.clone(),
                        updates_prefix: updates_prefix.clone(),
                        work_subject: work_subject.clone(),
                        consumer_name: consumer.clone(),
                        payload: required_schema_ref(config, "payload")?,
                        update: optional_schema_ref(config, "update")?,
                        result: optional_schema_ref(config, "result")?,
                        max_deliver: config
                            .get("maxDeliver")
                            .and_then(Value::as_i64)
                            .unwrap_or(5),
                        backoff_ms: config
                            .get("backoffMs")
                            .map(|_| required_i64_array(config, "backoffMs"))
                            .transpose()?
                            .unwrap_or_else(|| vec![5_000, 30_000, 120_000, 600_000]),
                        ack_wait_ms: config
                            .get("ackWaitMs")
                            .and_then(Value::as_i64)
                            .unwrap_or(300_000),
                        default_deadline_ms: optional_i64(config, "defaultDeadlineMs")?,
                        progress: config
                            .get("progress")
                            .and_then(Value::as_bool)
                            .unwrap_or(false),
                        logs: config.get("logs").and_then(Value::as_bool).unwrap_or(false),
                        dlq: config.get("dlq").and_then(Value::as_bool).unwrap_or(false),
                        key_concurrency: optional_policy(config, "keyConcurrency")?,
                        queue: optional_policy(config, "queue")?,
                    },
                );
            }
            ResourceProviderIdentity::EventConsumer {
                stream,
                consumer,
                filter_subjects,
            } => {
                let config = participant
                    .get("eventConsumers")
                    .and_then(Value::as_object)
                    .and_then(|consumers| consumers.get(&binding.local_name))
                    .ok_or_else(|| HttpError::internal("event_consumer_binding_invalid"))?;
                let max_deliver = config
                    .get("maxDeliver")
                    .and_then(Value::as_i64)
                    .unwrap_or(6);
                let backoff_ms = config
                    .get("backoffMs")
                    .map(|_| required_i64_array(config, "backoffMs"))
                    .transpose()?
                    .unwrap_or_else(|| {
                        [5_000, 30_000, 120_000, 600_000, 1_800_000]
                            .into_iter()
                            .take(max_deliver.saturating_sub(1) as usize)
                            .collect()
                    });
                resources.event_consumers.insert(
                    binding.local_name.clone(),
                    EventConsumerResourceBinding {
                        stream: stream.clone(),
                        consumer_name: consumer.clone(),
                        filter_subjects: filter_subjects.clone(),
                        replay: match config
                            .get("replay")
                            .and_then(Value::as_str)
                            .unwrap_or("new")
                        {
                            "new" => EventConsumerReplay::New,
                            "all" => EventConsumerReplay::All,
                            _ => EventConsumerReplay::Unknown,
                        },
                        ordering: match config
                            .get("ordering")
                            .and_then(Value::as_str)
                            .unwrap_or("strict")
                        {
                            "strict" => EventConsumerOrdering::Strict,
                            "parallel" => EventConsumerOrdering::Parallel,
                            _ => EventConsumerOrdering::Unknown,
                        },
                        ack_wait_ms: config
                            .get("ackWaitMs")
                            .and_then(Value::as_i64)
                            .unwrap_or(300_000),
                        max_deliver,
                        backoff_ms,
                    },
                );
            }
        }
    }
    if !job_queues.is_empty() {
        resources.jobs = Some(JobsResourceBinding {
            service_name: participant_id.to_owned(),
            namespace: jobs_namespace
                .ok_or_else(|| HttpError::internal("job_namespace_missing"))?,
            work_stream: jobs_work_stream,
            queues: job_queues,
        });
    }
    Ok(resources)
}

fn participant_resource<'a>(
    participant: &'a Value,
    family: &str,
    local_name: &str,
) -> Result<&'a Value, HttpError> {
    participant
        .get("resources")
        .and_then(Value::as_object)
        .and_then(|resources| resources.get(family))
        .and_then(Value::as_object)
        .and_then(|resources| resources.get(local_name))
        .ok_or_else(|| HttpError::internal("resource_binding_invalid"))
}

fn optional_i64(value: &Value, field: &str) -> Result<Option<i64>, HttpError> {
    match value.get(field) {
        None | Some(Value::Null) => Ok(None),
        Some(value) => value
            .as_i64()
            .map(Some)
            .ok_or_else(|| HttpError::internal("resource_policy_invalid")),
    }
}

fn required_i64_array(value: &Value, field: &str) -> Result<Vec<i64>, HttpError> {
    value
        .get(field)
        .and_then(Value::as_array)
        .ok_or_else(|| HttpError::internal("resource_policy_invalid"))?
        .iter()
        .map(|value| {
            value
                .as_i64()
                .ok_or_else(|| HttpError::internal("resource_policy_invalid"))
        })
        .collect()
}

fn required_schema_ref(value: &Value, field: &str) -> Result<JobsSchemaRef, HttpError> {
    optional_schema_ref(value, field)?
        .ok_or_else(|| HttpError::internal("job_schema_reference_missing"))
}

fn optional_schema_ref(value: &Value, field: &str) -> Result<Option<JobsSchemaRef>, HttpError> {
    let Some(value) = value.get(field) else {
        return Ok(None);
    };
    let schema = value
        .get("schema")
        .and_then(Value::as_str)
        .ok_or_else(|| HttpError::internal("job_schema_reference_invalid"))?;
    Ok(Some(JobsSchemaRef {
        schema: schema.to_owned(),
    }))
}

fn optional_policy<T: serde::de::DeserializeOwned>(
    value: &Value,
    field: &str,
) -> Result<Option<T>, HttpError> {
    let mut policy = value.get(field).cloned();
    if let Some(object) = policy.as_mut().and_then(Value::as_object_mut) {
        if field == "keyConcurrency" {
            object.entry("maxActive").or_insert(json!(1));
            object.entry("heartbeatIntervalMs").or_insert(json!(30_000));
            object.entry("heartbeatTtlMs").or_insert(json!(120_000));
            object.entry("stalePolicy").or_insert(json!("fail-stale"));
        } else if field == "queue" {
            object.entry("maxQueuedPerKey").or_insert(json!(0));
            object.entry("whenFull").or_insert(json!("reject"));
        }
    }
    policy
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| HttpError::internal("job_policy_invalid"))
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

#[cfg(test)]
mod tests {
    use axum::response::IntoResponse;

    use super::{
        canonical_origin, first_admin_token_hash, oauth_cookie_header, oauth_cookie_name,
        oidc_portal_policy_digest, project_service_resource_bindings,
        require_oauth_browser_binding, select_browser_authority, validate_redirect,
        ApprovalRequest, NatsBootstrapIssuer, EMBEDDED_PORTAL_ASSETS,
    };
    use crate::platform::auth::AuthorizationStateError;
    use crate::platform::auth::{
        ephemeral::{AuthOAuthKind, AuthOAuthState, AuthOAuthStatus, BrowserConsentProposal},
        LoginPortalRecord, LoginSettingsRecord, ResourceBindingEvidence, ResourceBindingState,
        ResourceProviderIdentity,
    };
    use axum::http::header::COOKIE;
    use axum::http::{HeaderMap, HeaderValue};
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;
    use base64::Engine as _;
    use nats_jwt_rs::user::User;
    use nats_jwt_rs::Claims;
    use nkeys::KeyPair;
    use sha2::{Digest as _, Sha256};
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use trellis_protocol::{
        ApiSurfaceKindV1, GrantSetV1, ParticipantResourceKindV1, PermissionActionV1,
        PermissionAtomV1, PermissionTargetV1,
    };

    const DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn permission(target: PermissionTargetV1, action: PermissionActionV1) -> PermissionAtomV1 {
        PermissionAtomV1::new(target, action).unwrap()
    }

    fn consent_with_view(view: serde_json::Value) -> BrowserConsentProposal {
        let required = permission(
            PermissionTargetV1::api_surface("example.api@v1", ApiSurfaceKindV1::Rpc, "Read")
                .unwrap(),
            PermissionActionV1::Call,
        );
        let optional = permission(
            PermissionTargetV1::api_surface("example.api@v1", ApiSurfaceKindV1::Event, "Updated")
                .unwrap(),
            PermissionActionV1::Subscribe,
        );
        BrowserConsentProposal::new(
            "app-1".to_owned(),
            DIGEST.to_owned(),
            DIGEST.to_owned(),
            view,
            GrantSetV1::new(vec![required]),
            BTreeMap::from([("events".to_owned(), GrantSetV1::new(vec![optional]))]),
            vec!["read".to_owned()],
            BTreeMap::new(),
        )
        .unwrap()
    }

    #[test]
    fn browser_security_boundaries_are_exact() {
        let allowed = vec!["https://app.example".to_owned()];
        assert!(validate_redirect("https://app.example/complete", &allowed).is_ok());
        assert!(validate_redirect("https://evil.example/complete", &allowed).is_err());
        assert!(validate_redirect("https://app.example/complete#secret", &allowed).is_err());
        assert_eq!(
            canonical_origin("https://app.example:443/path").unwrap(),
            "https://app.example"
        );

        let token = URL_SAFE_NO_PAD.encode([7_u8; 32]);
        let digest = first_admin_token_hash(&token).unwrap();
        assert_eq!(digest.len(), 43);
        assert!(!digest.contains(&token));
    }

    #[test]
    fn browser_approval_accepts_only_server_owned_optional_bundles() {
        let consent = consent_with_view(serde_json::json!({ "title": "Read data" }));
        let (required, capabilities, selected) = select_browser_authority(&consent, &[]).unwrap();
        assert_eq!(required, consent.required_grant_set);
        assert_eq!(capabilities, vec!["read"]);
        assert!(selected.is_empty());

        let (with_optional, _, selected) =
            select_browser_authority(&consent, &["events".to_owned()]).unwrap();
        assert_eq!(with_optional.permissions().len(), 2);
        assert!(selected.contains("events"));
        assert_eq!(
            select_browser_authority(&consent, &["unknown".to_owned()])
                .unwrap_err()
                .code,
            "unknown_optional_bundle"
        );
    }

    #[test]
    fn browser_approval_wire_ignores_caller_authored_machine_authority() {
        for atom in [
            permission(
                PermissionTargetV1::api_surface("unrelated.api@v1", ApiSurfaceKindV1::Rpc, "Admin")
                    .unwrap(),
                PermissionActionV1::Call,
            ),
            permission(
                PermissionTargetV1::api_surface(
                    "unrelated.api@v1",
                    ApiSurfaceKindV1::Event,
                    "Published",
                )
                .unwrap(),
                PermissionActionV1::Publish,
            ),
            permission(
                PermissionTargetV1::api_surface(
                    "unrelated.api@v1",
                    ApiSurfaceKindV1::State,
                    "records",
                )
                .unwrap(),
                PermissionActionV1::Write,
            ),
            permission(
                PermissionTargetV1::participant_resource(
                    "app-1",
                    ParticipantResourceKindV1::Kv,
                    "secrets",
                )
                .unwrap(),
                PermissionActionV1::Write,
            ),
        ] {
            assert_eq!(
                serde_json::from_value::<ApprovalRequest>(serde_json::json!({
                    "approved": true,
                    "consentViewDigest": DIGEST,
                    "selectedOptionalBundles": [],
                    "idempotencyKey": "request-1",
                    "grantSet": GrantSetV1::new(vec![atom]),
                }))
                .unwrap()
                .selected_optional_bundles,
                Vec::<String>::new(),
            );
        }
        assert!(
            serde_json::from_value::<ApprovalRequest>(serde_json::json!({
                "approved": true,
                "consentViewDigest": DIGEST,
                "selectedOptionalBundles": [],
                "idempotencyKey": "request-1",
                "capabilities": ["admin"],
            }))
            .is_ok()
        );
    }

    #[tokio::test]
    async fn http_errors_never_expose_internal_causes() {
        let secret = "postgres://admin:secret@internal/auth";
        let response = super::HttpError::from(AuthorizationStateError::Storage(secret.to_owned()))
            .into_response();
        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let encoded = String::from_utf8(body.to_vec()).unwrap();
        assert!(!encoded.contains(secret));
        assert!(encoded.contains("internal_error"));
    }

    #[test]
    fn consent_wording_does_not_change_machine_authority() {
        let before = consent_with_view(serde_json::json!({ "title": "Read data" }));
        let after = consent_with_view(serde_json::json!({ "title": "View data" }));
        assert_eq!(before.proposal_digest, after.proposal_digest);
        assert_ne!(before.consent_view_digest, after.consent_view_digest);
    }

    #[test]
    fn oauth_cookie_binds_one_browser_and_uses_callback_security_policy() {
        let state_id = "oauth-state";
        let secret = "browser-secret";
        let state = AuthOAuthState {
            format: "trellis.auth-oauth-state.v1".to_owned(),
            state_id: state_id.to_owned(),
            provider_id: "provider".to_owned(),
            kind: AuthOAuthKind::Browser,
            flow_id: "flow".to_owned(),
            status: AuthOAuthStatus::Pending,
            pkce_verifier: "verifier".to_owned(),
            nonce: "nonce".to_owned(),
            redirect_uri: "https://auth.example/auth/callback/provider".to_owned(),
            browser_binding_digest: URL_SAFE_NO_PAD.encode(Sha256::digest(secret.as_bytes())),
            portal_id: Some("builtin".to_owned()),
            portal_policy_digest: Some(super::digest_parts(&["policy"])),
            claim_owner: None,
            result_digest: None,
            created_at: 1,
            expires_at: 2,
            version: 1,
        };
        let mut headers = HeaderMap::new();
        assert!(require_oauth_browser_binding(state_id, &state, &headers).is_err());
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("{}=wrong", oauth_cookie_name(state_id))).unwrap(),
        );
        assert!(require_oauth_browser_binding(state_id, &state, &headers).is_err());
        headers.insert(
            COOKIE,
            HeaderValue::from_str(&format!("{}={secret}", oauth_cookie_name(state_id))).unwrap(),
        );
        require_oauth_browser_binding(state_id, &state, &headers).unwrap();
        assert!(require_oauth_browser_binding("another-state", &state, &headers).is_err());

        let cookie = oauth_cookie_header("binding", secret, "provider", true, 900)
            .unwrap()
            .to_str()
            .unwrap()
            .to_owned();
        assert!(cookie.contains("Path=/auth/callback/provider"));
        assert!(cookie.contains("Max-Age=900"));
        assert!(cookie.contains("HttpOnly"));
        assert!(cookie.contains("SameSite=Lax"));
        assert!(cookie.contains("Secure"));
    }

    #[test]
    fn oauth_portal_policy_digest_tracks_policy_not_wording() {
        let portal = LoginPortalRecord {
            portal_id: "builtin".to_owned(),
            display_name: "Trellis".to_owned(),
            entry_url: None,
            builtin: true,
            disabled: false,
            removed: false,
            local_registration_enabled: false,
            provider_ids: vec!["oidc".to_owned()],
            created_at: 1,
            updated_at: 1,
            version: 1,
        };
        let settings = LoginSettingsRecord {
            portal_id: "builtin".to_owned(),
            default_provider_id: Some("oidc".to_owned()),
            local_login_enabled: true,
            federated_registration_enabled: false,
            provider_selection_enabled: false,
            updated_at: 1,
            version: 1,
        };
        let digest = oidc_portal_policy_digest(&portal, &settings).unwrap();
        let mut wording = portal.clone();
        wording.display_name = "Renamed portal".to_owned();
        assert_eq!(
            oidc_portal_policy_digest(&wording, &settings).unwrap(),
            digest
        );
        let mut registration = settings.clone();
        registration.federated_registration_enabled = true;
        registration.version += 1;
        assert_ne!(
            oidc_portal_policy_digest(&portal, &registration).unwrap(),
            digest
        );
        let mut provider = portal.clone();
        provider.provider_ids.clear();
        provider.version += 1;
        assert_ne!(
            oidc_portal_policy_digest(&provider, &settings).unwrap(),
            digest
        );
    }

    #[test]
    fn embedded_portal_contains_fallback_and_assets() {
        assert!(EMBEDDED_PORTAL_ASSETS
            .iter()
            .any(|(path, bytes)| *path == "200.html" && !bytes.is_empty()));
        assert!(EMBEDDED_PORTAL_ASSETS
            .iter()
            .any(|(path, bytes)| path.starts_with("_trellis/assets/") && !bytes.is_empty()));
    }

    #[test]
    fn bootstrap_projects_exact_physical_resource_binding() {
        let participant = serde_json::json!({
            "resources": {
                "kv": {
                    "cache": {
                        "history": 2,
                        "ttlMs": 60000,
                        "maxValueBytes": 4096
                    }
                }
            },
            "jobQueues": {},
            "eventConsumers": {}
        });
        let evidence = ResourceBindingEvidence {
            resource_kind: "kv".to_owned(),
            local_name: "cache".to_owned(),
            binding_id: "bind_cache".to_owned(),
            owner_participant_id: "example".to_owned(),
            provider_identity: ResourceProviderIdentity::Kv {
                bucket: "KV_EXAMPLE_CACHE".to_owned(),
            },
            state: ResourceBindingState::Available,
            materialized_at: 1,
            error: None,
        };

        let projected =
            project_service_resource_bindings(&participant.to_string(), &[evidence], "example")
                .expect("resource binding");
        assert_eq!(projected.kv["cache"].bucket, "KV_EXAMPLE_CACHE");
        assert_eq!(projected.kv["cache"].history, 2);
    }

    #[test]
    fn bootstrap_jwt_is_session_keyed_and_deny_all() {
        let signing_key = KeyPair::new_account();
        let auth_account = KeyPair::new_account().public_key();
        let session_key = KeyPair::new_user().public_key();
        let issuer = NatsBootstrapIssuer {
            signing_key: Arc::new(signing_key),
            auth_account: auth_account.clone(),
            maximum_lifetime_seconds: 300,
        };
        let jwt = issuer.deny_all_user_jwt(&session_key, 10_000, 100).unwrap();
        assert_eq!(jwt.expires_at, 400);
        let claims = Claims::<User>::decode(&jwt.jwt).unwrap();
        assert_eq!(
            claims.payload().issuer_account.as_deref(),
            Some(auth_account.as_str())
        );
        assert_eq!(serde_json::to_value(&claims).unwrap()["sub"], session_key);
        assert_eq!(claims.exp, Some(400));
        assert!(claims
            .payload()
            .permissions
            .permissions
            .publish
            .allow
            .is_empty());
        assert_eq!(
            claims.payload().permissions.permissions.publish.deny,
            vec![">".to_owned()]
        );
        assert!(claims
            .payload()
            .permissions
            .permissions
            .subscribe
            .allow
            .is_empty());
    }
}
