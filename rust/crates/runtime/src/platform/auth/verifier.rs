//! Runtime-local request/event verification backed by the validator cache.
//!
//! The runtime verifier checks v1 request and event proofs entirely against
//! in-process state: the shared [`AuthorizationProviderCache`] (verified contexts,
//! online issuer keys, and full revocation timestamps) plus a precompiled
//! route/permission index built once at startup from the source-owned API
//! artifacts. Cache-hit verification performs no SQLite, HTTP, Auth RPC, or
//! NATS registry I/O; unknown context digests are resolved from the registry
//! through coalesced registry reads and expiry-bounded revocation watches.

use std::sync::Arc;

use bytes::Bytes;
use futures_util::future::BoxFuture;
use trellis_protocol::{AuthorizationEventPublisher, PermissionAtom, VerifiedAuthorizationContext};
use trellis_rs::client::{
    AuthorizationProviderCache, AuthorizationRegistryBinding, AuthorizationVerificationCore,
    EventVerificationInput, RequestVerificationInput, RuntimeAuthorizationTrust,
};
use trellis_rs::service::{
    RequestContext, RequestValidation, RequestValidator, ServerError, VerifiedCaller,
};

use super::AuthorizationStateError;

#[derive(Clone, Copy, Debug)]
pub(crate) struct RuntimeAuthorizationRequestVerificationInput<'a> {
    pub(crate) subject: &'a str,
    pub(crate) payload: &'a [u8],
    pub(crate) session_key: &'a str,
    pub(crate) proof: &'a str,
    pub(crate) authorization_context: &'a str,
    pub(crate) iat: i64,
    pub(crate) request_id: &'a str,
    pub(crate) reply: Option<&'a str>,
    pub(crate) required_permission: &'a PermissionAtom,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct RuntimeAuthorizationEventVerificationInput<'a> {
    pub(crate) subject: &'a str,
    pub(crate) payload: &'a [u8],
    pub(crate) session_key: &'a str,
    pub(crate) proof: &'a str,
    pub(crate) authorization_context: &'a str,
    pub(crate) event_id: &'a str,
    pub(crate) event_time: &'a str,
}

fn provider_error(error: trellis_rs::client::TrellisClientError) -> AuthorizationStateError {
    AuthorizationStateError::Storage(error.to_string())
}

/// Verified request material handed to Auth RPC handlers.
pub(crate) struct VerifiedRequest {
    pub(crate) caller: VerifiedCaller,
    pub(crate) context: VerifiedAuthorizationContext,
}

/// Runtime-local verifier shared by the Auth RPC provider and built-in routers.
#[derive(Clone)]
pub(crate) struct RuntimeAuthVerifier {
    source: Arc<AuthorizationProviderCache>,
    verification: AuthorizationVerificationCore,
}

impl std::fmt::Debug for RuntimeAuthVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeAuthVerifier")
            .finish_non_exhaustive()
    }
}

impl RuntimeAuthVerifier {
    pub(crate) fn new(source: Arc<AuthorizationProviderCache>) -> Self {
        Self {
            source,
            verification: AuthorizationVerificationCore::new(),
        }
    }

    /// Require one additional exact permission from an already-cached current context.
    pub(crate) fn require_cached_permission(
        &self,
        context_digest: &str,
        permission: &PermissionAtom,
    ) -> Result<(), AuthorizationStateError> {
        self.require_healthy()?;
        if self
            .source
            .runtime_revocation_time(context_digest)
            .map_err(provider_error)?
            .is_some()
        {
            return Err(denied("request is not granted by the active authority"));
        }
        let context = self
            .source
            .runtime_verified_context_raw(context_digest)
            .map_err(provider_error)?
            .ok_or_else(|| denied("authorization context is not cached"))?;
        if context.allows(permission) {
            Ok(())
        } else {
            Err(denied("request is not granted by the active authority"))
        }
    }

    /// Verify a v1 request proof against the exact routed permission.
    ///
    /// `reply` must be the actual NATS reply inbox (`message.reply`); the
    /// proof is bound to it. Verification happens only after full proof, time,
    /// revocation and exact permission checks succeed.
    pub(crate) async fn verify_request(
        &self,
        input: RuntimeAuthorizationRequestVerificationInput<'_>,
    ) -> Result<VerifiedRequest, AuthorizationStateError> {
        let RuntimeAuthorizationRequestVerificationInput {
            subject,
            payload,
            session_key,
            proof,
            authorization_context,
            iat,
            request_id,
            reply,
            required_permission,
        } = input;
        let now = now_seconds()?;
        if session_key.is_empty() || proof.is_empty() || authorization_context.is_empty() {
            return Err(denied("request proof headers are missing"));
        }
        let reply = reply
            .filter(|reply| !reply.is_empty())
            .ok_or_else(|| denied("request reply is missing"))?;
        self.require_healthy()?;
        if self
            .source
            .runtime_revocation_time(authorization_context)
            .map_err(provider_error)?
            .is_some()
        {
            return Err(denied("request is not granted by the active authority"));
        }
        let context = match self
            .source
            .runtime_verified_context_raw(authorization_context)
            .map_err(provider_error)?
        {
            Some(context) => context,
            None => self
                .source
                .resolve_admission_context(authorization_context, now)
                .await
                .map_err(provider_error)?,
        };
        let mut policy = self.source.runtime_policy().map_err(provider_error)?;
        policy.now_unix_seconds = now;
        let verified = self
            .verification
            .verify_request(RequestVerificationInput {
                context: &context,
                session_key,
                context_digest: authorization_context,
                subject,
                payload,
                iat,
                request_id,
                reply_subject: Some(reply),
                proof,
                policy: &policy,
                required_permissions: std::slice::from_ref(required_permission),
            })
            .map_err(|error| {
                denied(format!(
                    "request is not granted by the active authority: {error}"
                ))
            })?;
        Ok(VerifiedRequest {
            caller: verified.caller().clone(),
            context: verified.context().clone(),
        })
    }

    ///
    /// Historical events must fall within the signed context window. Published
    /// context revocation invalidates every event bound to that context.
    pub(crate) async fn verify_event(
        &self,
        input: RuntimeAuthorizationEventVerificationInput<'_>,
    ) -> Result<AuthorizationEventPublisher, trellis_rs::service::EventVerificationFailure> {
        use trellis_rs::service::EventVerificationFailure;

        let RuntimeAuthorizationEventVerificationInput {
            subject,
            payload,
            session_key,
            proof,
            authorization_context,
            event_id,
            event_time,
        } = input;

        let now = now_seconds()
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
        if session_key.is_empty()
            || proof.is_empty()
            || authorization_context.is_empty()
            || event_id.is_empty()
            || event_time.is_empty()
        {
            return Err(EventVerificationFailure::Rejected(
                "event proof headers are missing".into(),
            ));
        }
        if !self.source.runtime_healthy() {
            return Err(EventVerificationFailure::Retryable(
                "authorization validator is not ready".into(),
            ));
        }
        let revoked_at = self
            .source
            .runtime_revocation_time(authorization_context)
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
        let historical_time =
            time::OffsetDateTime::parse(event_time, &time::format_description::well_known::Rfc3339)
                .map_err(|_| EventVerificationFailure::Rejected("event time is invalid".into()))?
                .unix_timestamp();
        let context = match self
            .source
            .runtime_verified_context_raw(authorization_context)
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?
        {
            Some(context) => context,
            None => {
                self.source
                    .runtime_resolve_event_context_for_verification(
                        authorization_context,
                        historical_time,
                    )
                    .await?
            }
        };
        let mut policy = self
            .source
            .runtime_policy()
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
        policy.now_unix_seconds = now;
        let verified_event = self
            .verification
            .verify_event(EventVerificationInput {
                context: &context,
                session_key,
                context_digest: authorization_context,
                subject,
                payload,
                event_id,
                event_time,
                proof,
                policy: &policy,
                required_permissions: &[],
                revoked_at,
            })
            .map_err(|error| {
                EventVerificationFailure::Rejected(format!(
                    "event is not granted by the active authority: {error}"
                ))
            })?;
        Ok(verified_event.publisher().clone())
    }

    fn require_healthy(&self) -> Result<(), AuthorizationStateError> {
        if self.source.runtime_healthy() {
            Ok(())
        } else {
            Err(denied("authorization validator is not ready"))
        }
    }
}

pub(crate) async fn start_read_only(
    config: &crate::RuntimeConfig,
    client: async_nats::Client,
    stop: crate::shutdown::StopHandle,
) -> Result<
    (
        RuntimeAuthVerifier,
        tokio::task::JoinHandle<Result<(), crate::supervisor::RuntimeError>>,
    ),
    crate::supervisor::RuntimeError,
> {
    let authorization = config
        .resolve_authorization()
        .map_err(crate::supervisor::RuntimeError::Config)?;
    let now = now_seconds()
        .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let http = config.http.as_ref().ok_or_else(|| {
        crate::supervisor::RuntimeError::Platform(
            "HTTP configuration is required for issuer resolution".into(),
        )
    })?;
    let policy = trellis_protocol::AuthorizationVerificationPolicy::new(
        now,
        authorization
            .allowed_clock_skew_seconds
            .try_into()
            .map_err(|_| crate::supervisor::RuntimeError::Platform("invalid clock skew".into()))?,
        authorization
            .context_lifetime_seconds
            .try_into()
            .map_err(|_| {
                crate::supervisor::RuntimeError::Platform("invalid context lifetime".into())
            })?,
        authorization.maximum_context_bytes,
        authorization.maximum_permissions,
    )
    .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let cache = AuthorizationProviderCache::attach_runtime(
        client,
        &AuthorizationRegistryBinding::from_runtime_parts(authorization.context_bucket.clone()),
        RuntimeAuthorizationTrust {
            trellis_origin: http
                .public_origin
                .clone()
                .unwrap_or_else(|| format!("http://localhost:{}", config.http_port())),
            issuer: None,
            policy,
        },
    )
    .await
    .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let watcher = cache.clone();
    let watcher_stop = stop.clone();
    let join = tokio::spawn(async move {
        let (sender, receiver) = tokio::sync::watch::channel(());
        tokio::spawn(async move {
            watcher_stop.stopped().await;
            drop(sender);
        });
        watcher
            .run_runtime(receiver)
            .await
            .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))
    });
    tokio::time::timeout(std::time::Duration::from_secs(30), cache.wait_until_ready())
        .await
        .map_err(|_| {
            crate::supervisor::RuntimeError::Platform(
                "authorization provider cache did not become ready".to_owned(),
            )
        })?
        .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let verifier = RuntimeAuthVerifier::new(Arc::new(cache));
    Ok((verifier, join))
}

pub(crate) async fn ensure_read_only(
    context: &crate::supervisor::RuntimeContext,
    stop: crate::shutdown::StopHandle,
) -> Result<
    Option<tokio::task::JoinHandle<Result<(), crate::supervisor::RuntimeError>>>,
    crate::supervisor::RuntimeError,
> {
    if context.platform_verifier.get().is_some() {
        return Ok(None);
    }
    let (verifier, join) =
        start_read_only(&context.config, context.trellis_nats.clone(), stop).await?;
    context.platform_verifier.set(verifier).map_err(|_| {
        crate::supervisor::RuntimeError::Platform(
            "runtime-local auth verifier was already installed".to_owned(),
        )
    })?;
    Ok(Some(join))
}

impl RequestValidator for RuntimeAuthVerifier {
    fn validate<'a>(
        &'a self,
        subject: &'a str,
        payload: &'a Bytes,
        context: &'a RequestContext,
    ) -> BoxFuture<'a, Result<RequestValidation, ServerError>> {
        Box::pin(async move {
            let session_key =
                context
                    .session_key
                    .clone()
                    .ok_or_else(|| ServerError::MissingSessionKey {
                        subject: subject.to_string(),
                    })?;
            let proof = context
                .proof
                .clone()
                .filter(|value| !value.is_empty())
                .ok_or_else(|| ServerError::MissingProof {
                    subject: subject.to_string(),
                })?;
            let authorization_context = context.authorization_context.clone().ok_or_else(|| {
                ServerError::MissingAuthorizationContext {
                    subject: subject.to_string(),
                }
            })?;
            let iat = context.iat.ok_or_else(|| {
                ServerError::Nats(format!("authenticated request for '{subject}' has no iat"))
            })?;
            let request_id = context.request_id.clone().ok_or_else(|| {
                ServerError::Nats(format!(
                    "authenticated request for '{subject}' has no request-id"
                ))
            })?;
            let required_permission = match context
                .required_permission
                .as_ref()
                .ok_or_else(|| {
                    ServerError::Nats(format!(
                        "authenticated request for '{subject}' has no exact route permission"
                    ))
                })?
                .permission_atom()
            {
                Ok(permission) => permission,
                Err(error) => {
                    tracing::debug!(subject, %error, "invalid generated route permission");
                    return Ok(RequestValidation::denied());
                }
            };
            let verified = match self
                .verify_request(RuntimeAuthorizationRequestVerificationInput {
                    subject,
                    payload,
                    session_key: &session_key,
                    proof: &proof,
                    authorization_context: &authorization_context,
                    iat,
                    request_id: &request_id,
                    reply: context.reply_to.as_deref(),
                    required_permission: &required_permission,
                })
                .await
            {
                Ok(verified) => verified,
                Err(error) => {
                    tracing::debug!(subject, %error, "local request verification denied");
                    return Ok(RequestValidation::denied());
                }
            };
            Ok(RequestValidation {
                allowed: true,
                caller: Some(verified.caller),
                inbox_prefix: Some(verified.context.inbox_prefix().to_owned()),
            })
        })
    }
}

/// Fail-closed validator for built-in routers in platform-less runtime modes.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct DenyAllValidator;

impl RequestValidator for DenyAllValidator {
    fn validate<'a>(
        &'a self,
        _subject: &'a str,
        _payload: &'a Bytes,
        _context: &'a RequestContext,
    ) -> BoxFuture<'a, Result<RequestValidation, ServerError>> {
        Box::pin(async move { Ok(RequestValidation::denied()) })
    }
}

fn now_seconds() -> Result<i64, AuthorizationStateError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_secs()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time exceeds i64".to_owned()))
}

fn denied(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}
