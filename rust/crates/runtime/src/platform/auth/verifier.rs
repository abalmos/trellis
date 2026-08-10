//! Runtime-local request/event verification backed by the validator cache.
//!
//! The runtime verifier checks v2 request and event proofs entirely against
//! in-process state: the shared [`AuthorizationProviderCache`] (verified contexts,
//! manifest, and full revocation timestamps) plus a precompiled
//! route/permission index built once at startup from the source-owned API
//! artifacts. Cache-hit verification performs no SQLite, HTTP, Auth RPC, or
//! NATS registry I/O; unknown context digests are resolved from the registry
//! at most once per digest and then cached.

use std::sync::Arc;

use bytes::Bytes;
use futures_util::future::BoxFuture;
#[cfg(test)]
use trellis_protocol::{ApiSurfaceKindV1, PermissionActionV1, PermissionTargetV1};
use trellis_protocol::{
    AuthorizationEventPublisherV2, PermissionAtomV1, VerifiedAuthorizationContextV1,
};
use trellis_rs::client::{
    AuthorizationProviderCache, AuthorizationRegistryBinding, AuthorizationVerificationCore,
    RuntimeAuthorizationIoCounters, RuntimeAuthorizationTrust,
};
use trellis_rs::service::{
    RequestContext, RequestValidation, RequestValidator, ServerError, VerifiedCaller,
};

use super::AuthorizationStateError;

type AuthorizationValidatorIoCounters = RuntimeAuthorizationIoCounters;

/// Context and trust state consumed by the local verifier.
///
/// The runtime implements this over [`AuthorizationProviderCache`]; tests use
/// an in-memory stub to prove hot-path I/O accounting without a live registry.
pub(crate) trait ValidatorContextSource: Send + Sync {
    /// Return the verified context for a digest without lifecycle filtering;
    /// request revocation is checked separately via [`Self::revocation_time`]
    /// and both request/event windows are enforced by the protocol verifiers.
    fn verified_context(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, AuthorizationStateError>;

    /// Return the full revocation timestamp for a digest, when revoked.
    fn revocation_time(&self, digest: &str) -> Result<Option<i64>, AuthorizationStateError>;

    /// Return the verification policy bound to the current trust material.
    fn policy(
        &self,
    ) -> Result<trellis_protocol::AuthorizationVerificationPolicyV1, AuthorizationStateError>;

    /// Whether the registry watches are healthy (fail closed when not).
    fn healthy(&self) -> Result<bool, AuthorizationStateError>;

    /// Current verification time in Unix seconds.
    fn now_seconds(&self) -> Result<i64, AuthorizationStateError>;

    /// Registry I/O counters observed since startup.
    #[allow(dead_code)] // exercised by local verifier tests and live integration hooks
    fn io_counters(&self) -> AuthorizationValidatorIoCounters;

    /// Resolve an unknown context digest (coalesced; registry I/O).
    fn resolve_context(
        &self,
        digest: &str,
        now: i64,
    ) -> BoxFuture<'_, Result<VerifiedAuthorizationContextV1, AuthorizationStateError>>;

    /// Resolve an unknown context against its retained historical trust evidence.
    fn resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> BoxFuture<
        '_,
        Result<VerifiedAuthorizationContextV1, trellis_rs::service::EventVerificationFailure>,
    >;
}

impl ValidatorContextSource for AuthorizationProviderCache {
    fn verified_context(
        &self,
        digest: &str,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, AuthorizationStateError> {
        self.runtime_verified_context_raw(digest)
            .map_err(provider_error)
    }

    fn revocation_time(&self, digest: &str) -> Result<Option<i64>, AuthorizationStateError> {
        self.runtime_revocation_time(digest).map_err(provider_error)
    }

    fn policy(
        &self,
    ) -> Result<trellis_protocol::AuthorizationVerificationPolicyV1, AuthorizationStateError> {
        self.runtime_policy().map_err(provider_error)
    }

    fn healthy(&self) -> Result<bool, AuthorizationStateError> {
        Ok(self.runtime_healthy())
    }

    fn now_seconds(&self) -> Result<i64, AuthorizationStateError> {
        now_seconds()
    }

    fn io_counters(&self) -> AuthorizationValidatorIoCounters {
        self.runtime_io_counters()
    }

    fn resolve_context(
        &self,
        digest: &str,
        now: i64,
    ) -> BoxFuture<'_, Result<VerifiedAuthorizationContextV1, AuthorizationStateError>> {
        let digest = digest.to_owned();
        Box::pin(async move {
            self.resolve_admission_context(&digest, now)
                .await
                .map_err(provider_error)
        })
    }

    fn resolve_event_context(
        &self,
        digest: &str,
        event_time: i64,
    ) -> BoxFuture<
        '_,
        Result<VerifiedAuthorizationContextV1, trellis_rs::service::EventVerificationFailure>,
    > {
        let digest = digest.to_owned();
        Box::pin(async move {
            self.runtime_resolve_event_context_for_verification(&digest, event_time)
                .await
        })
    }
}

fn provider_error(error: trellis_rs::client::TrellisClientError) -> AuthorizationStateError {
    AuthorizationStateError::Storage(error.to_string())
}

/// Verified request material handed to Auth RPC handlers.
pub(crate) struct VerifiedRequest {
    pub(crate) caller: VerifiedCaller,
    pub(crate) context: VerifiedAuthorizationContextV1,
}

/// Runtime-local verifier shared by the Auth RPC provider and built-in routers.
#[derive(Clone)]
pub(crate) struct RuntimeAuthVerifier {
    source: Arc<dyn ValidatorContextSource>,
    verification: AuthorizationVerificationCore,
}

impl std::fmt::Debug for RuntimeAuthVerifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RuntimeAuthVerifier")
            .finish_non_exhaustive()
    }
}

impl RuntimeAuthVerifier {
    pub(crate) fn new(source: Arc<dyn ValidatorContextSource>) -> Self {
        Self {
            source,
            verification: AuthorizationVerificationCore::new(),
        }
    }

    /// Registry I/O counters observed by the backing context source.
    #[allow(dead_code)] // exercised by local verifier tests and live integration hooks
    pub(crate) fn io_counters(&self) -> AuthorizationValidatorIoCounters {
        self.source.io_counters()
    }

    /// Require one additional exact permission from an already-cached current context.
    pub(crate) fn require_cached_permission(
        &self,
        context_digest: &str,
        permission: &PermissionAtomV1,
    ) -> Result<(), AuthorizationStateError> {
        self.require_healthy()?;
        if self.source.revocation_time(context_digest)?.is_some() {
            return Err(denied("request is not granted by the active authority"));
        }
        let context = self
            .source
            .verified_context(context_digest)?
            .ok_or_else(|| denied("authorization context is not cached"))?;
        if context.allows(permission) {
            Ok(())
        } else {
            Err(denied("request is not granted by the active authority"))
        }
    }

    /// Verify a v2 request proof against the exact routed permission.
    ///
    /// `reply` must be the actual NATS reply inbox (`message.reply`); the
    /// proof is bound to it. Verification happens only after full proof, time,
    /// revocation, exact permission, and capability checks succeed.
    #[allow(clippy::too_many_arguments)] // Mirrors the complete signed request tuple.
    pub(crate) async fn verify_request(
        &self,
        subject: &str,
        payload: &[u8],
        session_key: &str,
        proof: &str,
        authorization_context: &str,
        iat: i64,
        request_id: &str,
        reply: Option<&str>,
        required_permission: &PermissionAtomV1,
        required_capabilities: &[String],
    ) -> Result<VerifiedRequest, AuthorizationStateError> {
        let now = self.source.now_seconds()?;
        if session_key.is_empty() || proof.is_empty() || authorization_context.is_empty() {
            return Err(denied("request proof headers are missing"));
        }
        let reply = reply
            .filter(|reply| !reply.is_empty())
            .ok_or_else(|| denied("request reply is missing"))?;
        self.require_healthy()?;
        if self
            .source
            .revocation_time(authorization_context)?
            .is_some()
        {
            return Err(denied("request is not granted by the active authority"));
        }
        let context = match self.source.verified_context(authorization_context)? {
            Some(context) => context,
            None => {
                self.source
                    .resolve_context(authorization_context, now)
                    .await?
            }
        };
        let mut policy = self.source.policy()?;
        policy.now_unix_seconds = now;
        let verified = self
            .verification
            .verify_request(
                &context,
                session_key,
                authorization_context,
                subject,
                payload,
                iat,
                request_id,
                Some(reply),
                proof,
                &policy,
                std::slice::from_ref(required_permission),
                required_capabilities,
            )
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
    /// Event eligibility is the strict signed window `[notBefore, expiresAt)`
    /// with `eventTime < revokedAt`; the full revocation timestamp is used.
    #[allow(clippy::too_many_arguments)] // Mirrors the complete signed event tuple.
    pub(crate) async fn verify_event(
        &self,
        subject: &str,
        payload: &[u8],
        session_key: &str,
        proof: &str,
        authorization_context: &str,
        event_id: &str,
        event_time: &str,
    ) -> Result<AuthorizationEventPublisherV2, trellis_rs::service::EventVerificationFailure> {
        use trellis_rs::service::EventVerificationFailure;

        let now = self
            .source
            .now_seconds()
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
        if !self
            .source
            .healthy()
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?
        {
            return Err(EventVerificationFailure::Retryable(
                "authorization validator is not ready".into(),
            ));
        }
        let revoked_at = self
            .source
            .revocation_time(authorization_context)
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
        let historical_time =
            time::OffsetDateTime::parse(event_time, &time::format_description::well_known::Rfc3339)
                .map_err(|_| EventVerificationFailure::Rejected("event time is invalid".into()))?
                .unix_timestamp();
        let context = match self
            .source
            .verified_context(authorization_context)
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?
        {
            Some(context) => context,
            None => {
                self.source
                    .resolve_event_context(authorization_context, historical_time)
                    .await?
            }
        };
        let mut policy = self
            .source
            .policy()
            .map_err(|error| EventVerificationFailure::Retryable(error.to_string()))?;
        policy.now_unix_seconds = now;
        let verified_event = self
            .verification
            .verify_event(
                &context,
                session_key,
                authorization_context,
                subject,
                payload,
                event_id,
                event_time,
                proof,
                &policy,
                &[],
                &[],
                revoked_at,
            )
            .map_err(|error| {
                EventVerificationFailure::Rejected(format!(
                    "event is not granted by the active authority: {error}"
                ))
            })?;
        Ok(verified_event.publisher().clone())
    }

    fn require_healthy(&self) -> Result<(), AuthorizationStateError> {
        if self.source.healthy()? {
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
    let trust = super::context::trust::VerifiedValidatorTrustMaterial::load(authorization, now)
        .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let manifest_digest = trust
        .manifest
        .digest()
        .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
    let cache = AuthorizationProviderCache::attach_runtime(
        client,
        &AuthorizationRegistryBinding::from_runtime_parts(
            authorization.trust_bucket.clone(),
            authorization.context_bucket.clone(),
        ),
        RuntimeAuthorizationTrust {
            root: trust.root.clone(),
            policy: trust.policy.clone(),
            minimum_manifest_generation: trust.verified_manifest.generation(),
            minimum_manifest_digest: manifest_digest.clone(),
            manifest: trust.verified_manifest.clone(),
            manifest_digest,
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
            let required_capabilities = context.required_capabilities.clone().unwrap_or_default();
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
                .verify_request(
                    subject,
                    payload,
                    &session_key,
                    &proof,
                    &authorization_context,
                    iat,
                    &request_id,
                    context.reply_to.as_deref(),
                    &required_permission,
                    &required_capabilities,
                )
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

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use std::collections::BTreeMap;
    use std::sync::Arc;
    use trellis_protocol::{
        parse_authorization_context_v1, parse_issuer_manifest_v1, verify_authorization_context_v1,
        verify_issuer_manifest_v1, AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1,
        VerifiedAuthorizationContextV1,
    };

    use super::*;

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ChainFixture {
        root_canonical_json: String,
        manifest_canonical_json: String,
        context_canonical_json: String,
        issuer_seed: String,
        session_seed: String,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct VectorDefaults {
        policy: PolicyFixture,
        request: RequestFixture,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct PolicyFixture {
        now_unix_seconds: i64,
        allowed_clock_skew_seconds: u32,
        maximum_context_lifetime_seconds: u32,
        maximum_context_bytes: usize,
        maximum_permissions: usize,
        maximum_capabilities: usize,
        minimum_manifest_generation: u64,
    }

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct RequestFixture {
        subject: String,
        reply: String,
        payload: String,
        iat: i64,
        request_id: String,
    }

    /// In-memory context source recording every registry resolution.
    #[derive(Default)]
    struct StubSource {
        contexts: std::sync::Mutex<BTreeMap<String, VerifiedAuthorizationContextV1>>,
        registry: std::sync::Mutex<BTreeMap<String, VerifiedAuthorizationContextV1>>,
        revocations: std::sync::Mutex<BTreeMap<String, i64>>,
        policy: std::sync::Mutex<Option<AuthorizationVerificationPolicyV1>>,
        now: std::sync::Mutex<Option<i64>>,
        resolves: std::sync::atomic::AtomicU64,
        healthy: std::sync::atomic::AtomicBool,
    }

    impl StubSource {
        fn install(
            &self,
            context: VerifiedAuthorizationContextV1,
            policy: AuthorizationVerificationPolicyV1,
        ) {
            let digest = context.context_digest().to_owned();
            let now = policy.now_unix_seconds;
            *self.policy.lock().unwrap() = Some(policy);
            *self.now.lock().unwrap() = Some(now);
            self.contexts
                .lock()
                .unwrap()
                .insert(digest.clone(), context.clone());
            self.registry.lock().unwrap().insert(digest, context);
            self.healthy
                .store(true, std::sync::atomic::Ordering::SeqCst);
        }

        fn seed_registry_only(&self, context: VerifiedAuthorizationContextV1) {
            self.registry
                .lock()
                .unwrap()
                .insert(context.context_digest().to_owned(), context);
        }
    }

    impl ValidatorContextSource for StubSource {
        fn verified_context(
            &self,
            digest: &str,
        ) -> Result<Option<VerifiedAuthorizationContextV1>, AuthorizationStateError> {
            Ok(self.contexts.lock().unwrap().get(digest).cloned())
        }

        fn revocation_time(&self, digest: &str) -> Result<Option<i64>, AuthorizationStateError> {
            Ok(self.revocations.lock().unwrap().get(digest).copied())
        }

        fn policy(
            &self,
        ) -> Result<trellis_protocol::AuthorizationVerificationPolicyV1, AuthorizationStateError>
        {
            Ok(self
                .policy
                .lock()
                .unwrap()
                .clone()
                .expect("installed policy"))
        }

        fn healthy(&self) -> Result<bool, AuthorizationStateError> {
            Ok(self.healthy.load(std::sync::atomic::Ordering::SeqCst))
        }

        fn now_seconds(&self) -> Result<i64, AuthorizationStateError> {
            self.now.lock().unwrap().ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("no clock installed".to_owned())
            })
        }

        fn io_counters(&self) -> AuthorizationValidatorIoCounters {
            AuthorizationValidatorIoCounters {
                context_resolves: self.resolves.load(std::sync::atomic::Ordering::Relaxed),
            }
        }

        fn resolve_context(
            &self,
            digest: &str,
            _now: i64,
        ) -> BoxFuture<'_, Result<VerifiedAuthorizationContextV1, AuthorizationStateError>>
        {
            self.resolves
                .fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            let digest = digest.to_owned();
            Box::pin(async move {
                let context = self
                    .registry
                    .lock()
                    .unwrap()
                    .get(&digest)
                    .cloned()
                    .ok_or_else(|| {
                        AuthorizationStateError::InvalidRecord(
                            "authorization context is missing from the registry".to_owned(),
                        )
                    })?;
                // Mirror the real cache: resolved contexts become cache hits.
                self.contexts
                    .lock()
                    .unwrap()
                    .insert(digest, context.clone());
                Ok(context)
            })
        }

        fn resolve_event_context(
            &self,
            digest: &str,
            event_time: i64,
        ) -> BoxFuture<
            '_,
            Result<VerifiedAuthorizationContextV1, trellis_rs::service::EventVerificationFailure>,
        > {
            let digest = digest.to_owned();
            Box::pin(async move {
                self.resolve_context(&digest, event_time)
                    .await
                    .map_err(|error| {
                        trellis_rs::service::EventVerificationFailure::Rejected(error.to_string())
                    })
            })
        }
    }

    fn fixture() -> (ChainFixture, VectorDefaults) {
        let fixture_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/authorization-context/vectors.json");
        let value: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(fixture_path).unwrap()).unwrap();
        (
            serde_json::from_value(value["completeChain"].clone()).unwrap(),
            serde_json::from_value(value["defaults"].clone()).unwrap(),
        )
    }

    /// Re-sign the fixture context with additional exact permissions using the
    /// fixture issuer key, returning the verified context and its digest.
    fn fixture_context_with_grants(
        chain: &ChainFixture,
        defaults: &VectorDefaults,
        grants: Vec<PermissionAtomV1>,
    ) -> (
        VerifiedAuthorizationContextV1,
        AuthorizationVerificationPolicyV1,
        String,
    ) {
        use base64::Engine as _;
        let policy = AuthorizationVerificationPolicyV1::new(
            defaults.policy.now_unix_seconds,
            defaults.policy.allowed_clock_skew_seconds,
            defaults.policy.maximum_context_lifetime_seconds,
            defaults.policy.maximum_context_bytes,
            defaults.policy.maximum_permissions,
            defaults.policy.maximum_capabilities,
            defaults.policy.minimum_manifest_generation,
        )
        .unwrap();
        let root = AuthorizationTrustRootV1::parse(
            &serde_json::from_str(&chain.root_canonical_json).unwrap(),
        )
        .unwrap();
        let manifest = parse_issuer_manifest_v1(
            &serde_json::from_str(&chain.manifest_canonical_json).unwrap(),
        )
        .unwrap();
        let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &policy).unwrap();
        let fixture = parse_authorization_context_v1(
            &serde_json::from_str(&chain.context_canonical_json).unwrap(),
        )
        .unwrap();
        let mut unsigned = fixture.unsigned.clone();
        unsigned.grant_set = trellis_protocol::GrantSetV1::new(grants);
        let seed = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&chain.issuer_seed)
            .expect("issuer seed");
        let issuer_key = ed25519_dalek::SigningKey::from_bytes(
            seed.as_slice().try_into().expect("issuer seed bytes"),
        );
        let signed = trellis_protocol::sign_authorization_context_v1(unsigned, &issuer_key)
            .expect("sign granted context");
        let verified = verify_authorization_context_v1(&root, &verified_manifest, &signed, &policy)
            .expect("verify granted context");
        let digest = signed.digest().expect("granted context digest");
        (verified, policy, digest)
    }

    fn documents_grants() -> (PermissionAtomV1, PermissionAtomV1) {
        let call = PermissionAtomV1::new(
            PermissionTargetV1::api_surface("documents@v1", ApiSurfaceKindV1::Rpc, "Documents.Get")
                .unwrap(),
            PermissionActionV1::Call,
        )
        .unwrap();
        let publish = PermissionAtomV1::new(
            PermissionTargetV1::api_surface(
                "documents@v1",
                ApiSurfaceKindV1::Event,
                "Documents.Changed",
            )
            .unwrap(),
            PermissionActionV1::Publish,
        )
        .unwrap();
        (call, publish)
    }

    fn documents_permission() -> PermissionAtomV1 {
        documents_grants().0
    }

    /// Build a verifier whose context is the fixture context (with the
    /// documents grants) valid at the fixture policy time.
    fn verifier(
        revoked_at: Option<i64>,
    ) -> (RuntimeAuthVerifier, Arc<StubSource>, ChainFixture, String) {
        let (chain, defaults) = fixture();
        let (call, publish) = documents_grants();
        let (verified, mut policy, digest) =
            fixture_context_with_grants(&chain, &defaults, vec![call, publish]);
        let now = defaults.policy.now_unix_seconds;
        policy.now_unix_seconds = now;
        let source = Arc::new(StubSource::default());
        source.install(verified, policy);
        if let Some(revoked_at) = revoked_at {
            source
                .revocations
                .lock()
                .unwrap()
                .insert(digest.clone(), revoked_at);
        }
        (
            RuntimeAuthVerifier::new(source.clone()),
            source,
            chain,
            digest,
        )
    }

    fn signer(chain: &ChainFixture) -> trellis_rs::client::SessionAuth {
        trellis_rs::client::SessionAuth::from_seed_base64url(&chain.session_seed).unwrap()
    }

    #[tokio::test]
    async fn valid_request_and_denied_altered_inputs() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let (verifier, source, _, digest) = verifier(None);
        let proof = auth
            .create_request_proof_v2(
                &digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();

        let verified = verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .expect("valid request");
        assert_eq!(verified.caller.session_key, auth.session_key);
        assert_eq!(verified.caller.context_digest, digest);
        assert_eq!(verified.caller.session_id, "ses_test");

        // Altered reply subject denies: the proof is bound to the exact inbox.
        assert!(verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some("_INBOX.other.reply"),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        // Altered subject denies: the proof is bound to the exact subject.
        assert!(verifier
            .verify_request(
                "rpc.v1.Documents.List",
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        // Altered payload denies: the proof covers the payload digest.
        assert!(verifier
            .verify_request(
                &defaults.request.subject,
                br#"{"tampered":true}"#,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        // Unknown subject denies without any context resolution.
        assert!(verifier
            .verify_request(
                "rpc.v1.Unknown.Surface",
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        assert_eq!(source.io_counters().context_resolves, 0);
    }

    #[tokio::test]
    async fn duplicate_request_and_event_ids_remain_accepted() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let (verifier, _, _, digest) = verifier(None);
        let proof = auth
            .create_request_proof_v2(
                &digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .expect("first use");
        // A repeated signed request remains valid.
        verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .expect("duplicate request id remains accepted");

        // The same event id delivered twice is accepted as a fresh signed proof.
        let event_id = "evt_dup";
        let event_time = "1970-01-01T00:19:10Z";
        let event_subject = "events.v1.Documents.Changed.doc-1";
        let event_proof = auth
            .create_event_proof_v2(
                &digest,
                event_subject,
                br#"{"id":"doc-1"}"#,
                event_id,
                event_time,
            )
            .unwrap();
        verifier
            .verify_event(
                event_subject,
                br#"{"id":"doc-1"}"#,
                &auth.session_key,
                event_proof.as_str(),
                &digest,
                event_id,
                event_time,
            )
            .await
            .expect("first event");
        verifier
            .verify_event(
                event_subject,
                br#"{"id":"doc-1"}"#,
                &auth.session_key,
                event_proof.as_str(),
                &digest,
                event_id,
                event_time,
            )
            .await
            .expect("duplicate event id remains accepted");
    }

    #[tokio::test]
    async fn revoked_context_denies_requests_and_bounds_historical_events() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let (verifier, _, _, digest) = verifier(Some(1150));
        let proof = auth
            .create_request_proof_v2(
                &digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        assert!(verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        let event_subject = "events.v1.Documents.Changed.doc-1";
        // Events before the revocation remain eligible...
        let before_proof = auth
            .create_event_proof_v2(
                &digest,
                event_subject,
                br#"{"id":"doc-1"}"#,
                "evt_before",
                "1970-01-01T00:19:00Z",
            )
            .unwrap();
        verifier
            .verify_event(
                event_subject,
                br#"{"id":"doc-1"}"#,
                &auth.session_key,
                before_proof.as_str(),
                &digest,
                "evt_before",
                "1970-01-01T00:19:00Z",
            )
            .await
            .expect("pre-revocation event remains eligible");
        // ...while events at or after the revocation deny.
        let after_proof = auth
            .create_event_proof_v2(
                &digest,
                event_subject,
                br#"{"id":"doc-1"}"#,
                "evt_after",
                "1970-01-01T00:19:30Z",
            )
            .unwrap();
        assert!(verifier
            .verify_event(
                event_subject,
                br#"{"id":"doc-1"}"#,
                &auth.session_key,
                after_proof.as_str(),
                &digest,
                "evt_after",
                "1970-01-01T00:19:30Z",
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn missing_exact_grant_set_atom_denies_without_resolution() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        // Sign a proof for a production Auth surface the fixture context lacks.
        let subject = "rpc.v1.Auth.Sessions.Me";
        let reply = format!("{}.reply", "_INBOX.test");
        let (call, _) = documents_grants();
        let (verified, mut policy, digest) =
            fixture_context_with_grants(&chain, &defaults, vec![call]);
        policy.now_unix_seconds = defaults.policy.now_unix_seconds;
        let source = Arc::new(StubSource::default());
        source.install(verified, policy);
        let verifier = RuntimeAuthVerifier::new(source.clone());
        let mut routes = trellis_rs::service::Router::new();
        trellis_sdk_auth::api::register_rpc_metadata(&mut routes);
        let required_permission = routes
            .required_permission(subject, payload)
            .unwrap()
            .unwrap()
            .permission_atom()
            .unwrap();
        let proof = auth
            .create_request_proof_v2(&digest, subject, &reply, payload, 1100, "req_admin")
            .unwrap();
        assert!(verifier
            .verify_request(
                subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                1100,
                "req_admin",
                Some(&reply),
                &required_permission,
                &[],
            )
            .await
            .is_err());
        // The denial is local: no registry resolution was performed.
        assert_eq!(source.io_counters().context_resolves, 0);
    }

    #[tokio::test]
    async fn cache_hit_performs_no_registry_io_and_unknown_context_resolves_once() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let now = defaults.policy.now_unix_seconds;
        let subject = &defaults.request.subject;
        let reply = defaults.request.reply.clone();
        let (verifier, source, _, digest) = verifier(None);
        let proof = |request_id: &str| {
            auth.create_request_proof_v2(&digest, subject, &reply, payload, now, request_id)
                .unwrap()
        };
        verifier
            .verify_request(
                subject,
                payload,
                &auth.session_key,
                proof("req_1").as_str(),
                &digest,
                now,
                "req_1",
                Some(&reply),
                &documents_permission(),
                &[],
            )
            .await
            .expect("first cache hit");
        // A second cache-hit request performs no registry I/O.
        verifier
            .verify_request(
                subject,
                payload,
                &auth.session_key,
                proof("req_2").as_str(),
                &digest,
                now,
                "req_2",
                Some(&reply),
                &documents_permission(),
                &[],
            )
            .await
            .expect("second cache hit");
        assert_eq!(source.io_counters().context_resolves, 0);

        // An unknown digest resolves exactly once and is then cached.
        let (chain, defaults) = fixture();
        let (call, _) = documents_grants();
        let (verified, mut policy, unknown_digest) =
            fixture_context_with_grants(&chain, &defaults, vec![call]);
        policy.now_unix_seconds = now;
        let source = Arc::new(StubSource::default());
        *source.policy.lock().unwrap() = Some(policy.clone());
        *source.now.lock().unwrap() = Some(policy.now_unix_seconds);
        source
            .healthy
            .store(true, std::sync::atomic::Ordering::SeqCst);
        source.seed_registry_only(verified);
        let verifier = RuntimeAuthVerifier::new(source.clone());
        for request_id in ["req_unknown_1", "req_unknown_2"] {
            let proof = auth
                .create_request_proof_v2(&unknown_digest, subject, &reply, payload, now, request_id)
                .unwrap();
            verifier
                .verify_request(
                    subject,
                    payload,
                    &auth.session_key,
                    proof.as_str(),
                    &unknown_digest,
                    now,
                    request_id,
                    Some(&reply),
                    &documents_permission(),
                    &[],
                )
                .await
                .expect("unknown context resolves");
        }
        assert_eq!(source.io_counters().context_resolves, 1);
    }

    #[tokio::test]
    async fn unhealthy_source_fails_closed_without_io() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let (verifier, source, _, digest) = verifier(None);
        source
            .healthy
            .store(false, std::sync::atomic::Ordering::SeqCst);
        let proof = auth
            .create_request_proof_v2(
                &digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        assert!(verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &auth.session_key,
                proof.as_str(),
                &digest,
                defaults.request.iat,
                &defaults.request.request_id,
                Some(&defaults.request.reply),
                &documents_permission(),
                &[],
            )
            .await
            .is_err());
        assert_eq!(source.io_counters().context_resolves, 0);
    }
}
