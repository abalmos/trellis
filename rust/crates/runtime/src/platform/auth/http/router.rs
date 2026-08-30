use std::sync::Arc;
use std::time::Duration;

use axum::http::header::CONTENT_SECURITY_POLICY;
use axum::http::{HeaderValue, Method};
use axum::middleware;
use axum::routing::{get, post};
use axum::Router;
use tower_governor::governor::GovernorConfigBuilder;
use tower_governor::GovernorLayer;
use tower_http::compression::predicate::{DefaultPredicate, NotForContentType, Predicate};
use tower_http::compression::CompressionLayer;
use tower_http::cors::{AllowHeaders, AllowOrigin, CorsLayer};
use tower_http::limit::RequestBodyLimitLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use url::Url;

use super::bootstrap::{device_bootstrap, service_bootstrap};
use super::browser::{
    bind_flow, complete_first_admin, console_index, console_page, decide_approval,
    get_account_flow, get_flow, local_login, oidc_callback, portal_asset, portal_index,
    portal_page, register_local, start_account_flow_oidc, start_auth, start_oidc,
};
use super::security::{canonical_origin, security_headers};
use super::well_known::refresh_context;
use super::{
    AccountRepository, AuthEphemeralRepository, AuthHttpOptions, AuthHttpState,
    AuthorityEvidenceRepository, AuthorityRepository, AuthorizationStateError, ContextRepository,
    DeploymentRepository, OutboxRepository, PortalRepository, ProvisioningRepository,
    SessionRepository, MAX_AUTH_REQUEST_BODY_BYTES,
};

const LOCAL_LOGIN_ROUTE: &str = "/auth/login/local";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteMethod {
    Get,
    Post,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum RouteHandler {
    StartAuth,
    ServiceBootstrap,
    DeviceBootstrap,
    ContextRefresh,
    GetFlow,
    LocalLogin,
    RegisterLocal,
    GetAccountFlow,
    CompleteFirstAdmin,
    StartOidc,
    StartAccountFlowOidc,
    OidcCallback,
    DecideApproval,
    BindFlow,
    PortalIndex,
    PortalPage,
    PortalAsset,
    ConsoleIndex,
    ConsolePage,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct RouteDefinition {
    method: RouteMethod,
    path: &'static str,
    handler: RouteHandler,
}

const ROUTES: &[RouteDefinition] = &[
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/requests",
        handler: RouteHandler::StartAuth,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/bootstrap/service",
        handler: RouteHandler::ServiceBootstrap,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/bootstrap/device",
        handler: RouteHandler::DeviceBootstrap,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/context/refresh",
        handler: RouteHandler::ContextRefresh,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/auth/flow/:flow_id",
        handler: RouteHandler::GetFlow,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: LOCAL_LOGIN_ROUTE,
        handler: RouteHandler::LocalLogin,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/flow/:flow_id/register/local",
        handler: RouteHandler::RegisterLocal,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/auth/account-flow/:flow_token",
        handler: RouteHandler::GetAccountFlow,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/account-flow/:flow_token/local-password",
        handler: RouteHandler::CompleteFirstAdmin,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/auth/login/:provider_id",
        handler: RouteHandler::StartOidc,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/auth/account-flow/:flow_token/login/:provider_id",
        handler: RouteHandler::StartAccountFlowOidc,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/auth/callback/:provider_id",
        handler: RouteHandler::OidcCallback,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/flow/:flow_id/approval",
        handler: RouteHandler::DecideApproval,
    },
    RouteDefinition {
        method: RouteMethod::Post,
        path: "/auth/flow/:flow_id/bind",
        handler: RouteHandler::BindFlow,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/login",
        handler: RouteHandler::PortalIndex,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/login/*path",
        handler: RouteHandler::PortalPage,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/assets/login/*path",
        handler: RouteHandler::PortalAsset,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/console",
        handler: RouteHandler::ConsoleIndex,
    },
    RouteDefinition {
        method: RouteMethod::Get,
        path: "/console/*path",
        handler: RouteHandler::ConsolePage,
    },
];

fn add_route<R, E>(
    routes: Router<AuthHttpState<R, E>>,
    route: RouteDefinition,
) -> Router<AuthHttpState<R, E>>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone + Send + Sync + 'static,
{
    match (route.method, route.handler) {
        (RouteMethod::Post, RouteHandler::StartAuth) => {
            routes.route(route.path, post(start_auth::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::ServiceBootstrap) => {
            routes.route(route.path, post(service_bootstrap::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::DeviceBootstrap) => {
            routes.route(route.path, post(device_bootstrap::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::ContextRefresh) => {
            routes.route(route.path, post(refresh_context::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::GetFlow) => {
            routes.route(route.path, get(get_flow::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::LocalLogin) => {
            routes.route(route.path, post(local_login::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::RegisterLocal) => {
            routes.route(route.path, post(register_local::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::GetAccountFlow) => {
            routes.route(route.path, get(get_account_flow::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::CompleteFirstAdmin) => {
            routes.route(route.path, post(complete_first_admin::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::StartOidc) => {
            routes.route(route.path, get(start_oidc::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::StartAccountFlowOidc) => {
            routes.route(route.path, get(start_account_flow_oidc::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::OidcCallback) => {
            routes.route(route.path, get(oidc_callback::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::DecideApproval) => {
            routes.route(route.path, post(decide_approval::<R, E>))
        }
        (RouteMethod::Post, RouteHandler::BindFlow) => {
            routes.route(route.path, post(bind_flow::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::PortalIndex) => {
            routes.route(route.path, get(portal_index::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::PortalPage) => {
            routes.route(route.path, get(portal_page::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::PortalAsset) => {
            routes.route(route.path, get(portal_asset::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::ConsoleIndex) => {
            routes.route(route.path, get(console_index::<R, E>))
        }
        (RouteMethod::Get, RouteHandler::ConsolePage) => {
            routes.route(route.path, get(console_page::<R, E>))
        }
        _ => unreachable!("auth route inventory method and handler disagree"),
    }
}

pub(crate) fn router<R, E>(
    options: AuthHttpOptions<R, E>,
) -> Result<Router, AuthorizationStateError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone + Send + Sync + 'static,
{
    let public_url = Url::parse(&options.public_origin).map_err(|_| {
        AuthorizationStateError::InvalidRecord("HTTP public origin is invalid".to_owned())
    })?;
    let use_hsts = public_url.scheme() == "https";
    let content_security_policy = content_security_policy(&options.websocket_nats_servers)?;
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
        native_nats_servers: options.native_nats_servers,
        websocket_nats_servers: options.websocket_nats_servers,
        oidc_providers: options.oidc_providers,
        proof_policy: trellis_protocol::SessionProofPolicy::default(),
        browser_proof_policy: trellis_protocol::SessionProofPolicy::new(300_000, 300_000)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        portal_override_dir: options.portal_override_dir,
    };
    let mut api_routes: Router<AuthHttpState<R, E>> = Router::new();
    let mut browser_routes: Router<AuthHttpState<R, E>> = Router::new();
    for route in ROUTES {
        if is_browser_route(route.handler) {
            browser_routes = add_route::<R, E>(browser_routes, *route);
        } else {
            api_routes = add_route::<R, E>(api_routes, *route);
        }
    }
    let mut api_routes = api_routes.with_state(state.clone());
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
        api_routes = api_routes.layer(GovernorLayer {
            config: Arc::new(config),
        });
    }
    let mut routes = api_routes
        .merge(browser_routes.with_state(state))
        .layer(RequestBodyLimitLayer::new(MAX_AUTH_REQUEST_BODY_BYTES))
        .layer(cors)
        .layer(browser_compression_layer())
        .layer(middleware::from_fn(security_headers))
        .layer(SetResponseHeaderLayer::overriding(
            CONTENT_SECURITY_POLICY,
            content_security_policy,
        ));
    if use_hsts {
        routes = routes.layer(SetResponseHeaderLayer::if_not_present(
            axum::http::header::STRICT_TRANSPORT_SECURITY,
            HeaderValue::from_static("max-age=31536000; includeSubDomains"),
        ));
    }
    Ok(routes)
}

fn is_browser_route(handler: RouteHandler) -> bool {
    matches!(
        handler,
        RouteHandler::PortalIndex
            | RouteHandler::PortalPage
            | RouteHandler::PortalAsset
            | RouteHandler::ConsoleIndex
            | RouteHandler::ConsolePage
    )
}

fn browser_compression_layer() -> CompressionLayer<impl Predicate> {
    CompressionLayer::new().compress_when(
        DefaultPredicate::new().and(NotForContentType::const_new("application/json")),
    )
}

fn content_security_policy(
    websocket_nats_servers: &[String],
) -> Result<HeaderValue, AuthorizationStateError> {
    let mut connect_sources = vec!["'self'".to_owned()];
    for server in websocket_nats_servers {
        let url = Url::parse(server).map_err(|_| {
            AuthorizationStateError::InvalidRecord(
                "NATS WebSocket URL is invalid for browser CSP".to_owned(),
            )
        })?;
        if !matches!(url.scheme(), "ws" | "wss")
            || !url.username().is_empty()
            || url.password().is_some()
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "NATS WebSocket URL is invalid for browser CSP".to_owned(),
            ));
        }
        connect_sources.push(url.origin().ascii_serialization());
    }
    connect_sources.sort();
    connect_sources.dedup();
    let policy = format!(
        "default-src 'none'; base-uri 'none'; frame-ancestors 'none'; form-action 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self' 'unsafe-inline'; img-src 'self' data:; connect-src {}",
        connect_sources.join(" ")
    );
    HeaderValue::from_str(&policy).map_err(|_| {
        AuthorizationStateError::InvalidRecord(
            "browser content security policy is invalid".to_owned(),
        )
    })
}

#[cfg(test)]
mod tests {
    use axum::body::Body;
    use axum::http::header::{ACCEPT_ENCODING, CONTENT_ENCODING, CONTENT_TYPE};
    use axum::http::{Method, Request, StatusCode};
    use axum::routing::{get, post};
    use axum::Router;
    use tower::ServiceExt;

    use super::{browser_compression_layer, content_security_policy, RouteMethod, ROUTES};

    fn route_inventory() -> Vec<(Method, String)> {
        ROUTES
            .iter()
            .map(|route| {
                (
                    match route.method {
                        RouteMethod::Get => Method::GET,
                        RouteMethod::Post => Method::POST,
                    },
                    route.path.to_owned(),
                )
            })
            .collect()
    }

    fn allowlist() -> Vec<(Method, String)> {
        include_str!("allowlist.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(|line| {
                let (method, path) = line.split_once(' ').expect("valid auth route allowlist");
                (
                    method.parse().expect("valid auth route method"),
                    path.to_owned(),
                )
            })
            .collect()
    }

    fn probe_path(path: &str) -> String {
        path.replace(":key", "probe")
            .replace(":flow_id", "flow")
            .replace(":flow_token", "token")
            .replace(":provider_id", "provider")
            .replace("*path", "index.html")
    }

    fn production_route_shape() -> Router {
        let mut router = Router::new();
        for route in ROUTES {
            router = match route.method {
                RouteMethod::Get => {
                    router.route(route.path, get(|| async { StatusCode::NO_CONTENT }))
                }
                RouteMethod::Post => {
                    router.route(route.path, post(|| async { StatusCode::NO_CONTENT }))
                }
            };
        }
        router
    }

    #[tokio::test]
    async fn final_auth_allowlist_matches_production_routes() {
        let routes = route_inventory();
        assert_eq!(routes, allowlist());

        let app = production_route_shape();
        for (method, path) in routes {
            let response = app
                .clone()
                .oneshot(
                    Request::builder()
                        .method(method.clone())
                        .uri(probe_path(&path))
                        .body(Body::empty())
                        .unwrap(),
                )
                .await
                .unwrap();
            assert_ne!(response.status(), StatusCode::NOT_FOUND, "{method} {path}");
        }
    }

    #[tokio::test]
    async fn browser_responses_are_compressed() {
        let app = Router::new()
            .route("/", get(|| async { "x".repeat(32_768) }))
            .route(
                "/json",
                get(|| async { ([(CONTENT_TYPE, "application/json")], "x".repeat(32_768)) }),
            )
            .layer(browser_compression_layer());
        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header(ACCEPT_ENCODING, "br")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(response.headers()[CONTENT_ENCODING], "br");

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/json")
                    .header(ACCEPT_ENCODING, "br")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert!(!response.headers().contains_key(CONTENT_ENCODING));
    }

    #[test]
    fn browser_csp_allows_wasm_and_exact_nats_websocket_origins() {
        let policy = content_security_policy(&[
            "ws://localhost:8080/nats".to_owned(),
            "wss://nats.example.test/connect".to_owned(),
        ])
        .unwrap();
        let policy = policy.to_str().unwrap();
        assert!(policy.contains("script-src 'self' 'wasm-unsafe-eval'"));
        assert!(policy.contains("connect-src 'self' ws://localhost:8080 wss://nats.example.test"));
        assert!(content_security_policy(&["https://nats.example.test".to_owned()]).is_err());
    }
}
