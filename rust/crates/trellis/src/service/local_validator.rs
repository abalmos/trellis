//! Local context-bound request and event verification for connected services.
//!
//! Connected services verify v2 request/event proofs entirely against
//! in-process state: the digest-keyed [`AuthorizationProviderCache`] (verified
//! caller contexts resolved through the connected NATS authorization registry,
//! live revocation state, and verification policy) plus the exact permission
//! surface recorded at route/event registration time. Verification performs no
//! SQLite, HTTP, Auth RPC, or NATS registry I/O on cache hits: cache hits read
//! only memory. Unknown context digests are resolved from the registry at most
//! once per digest, and revocation watch updates are applied immediately.

use bytes::Bytes;
use futures_util::future::BoxFuture;
use trellis_protocol::{
    ApiSurfaceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1, ProtocolError,
};

use crate::client::{AuthorizationProviderCache, AuthorizationVerificationCore};
use crate::service::{
    RequestContext, RequestValidation, RequestValidator, RoutePermission, ServerError,
};

pub use crate::client::VerifiedCaller;

/// Local request/event verifier for one connected service runtime.
///
/// A verifier without a provider cache (constructed before the connected
/// authorization registry is available) denies every request/event: local
/// verification fails closed until provider evidence is ready.
#[derive(Clone)]
pub struct LocalAuthVerifier {
    provider: Option<AuthorizationProviderCache>,
    /// Versioned API/contract identity of the connected service, used to build
    /// exact permission atoms for its own routed surfaces.
    contract_id: String,
    verification: AuthorizationVerificationCore,
}

impl LocalAuthVerifier {
    /// Build a verifier over the client's provider cache and the connected
    /// service's versioned contract/API identity.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(provider: Option<AuthorizationProviderCache>, api_id: impl Into<String>) -> Self {
        Self {
            provider,
            contract_id: api_id.into(),
            verification: AuthorizationVerificationCore::new(),
        }
    }

    pub(crate) fn api_id(&self) -> &str {
        &self.contract_id
    }

    #[cfg(test)]
    fn resolver_count_for_test(&self) -> u64 {
        self.provider.as_ref().map_or(0, |provider| {
            let counters = provider.io_counters();
            counters.context_resolves
        })
    }

    /// Verify a v1 request proof against the digest-keyed context and exact route.
    async fn verify_request(
        &self,
        subject: &str,
        payload: &[u8],
        context: &RequestContext,
    ) -> Result<RequestValidation, ServerError> {
        self.verify_request_inner(subject, payload, context, true)
            .await
    }

    async fn verify_request_inner(
        &self,
        subject: &str,
        payload: &[u8],
        context: &RequestContext,
        require_exact_permission: bool,
    ) -> Result<RequestValidation, ServerError> {
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
        let reply = context
            .reply_to
            .clone()
            .filter(|value| !value.is_empty())
            .ok_or_else(|| ServerError::MissingReply {
                subject: subject.to_string(),
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
        let route_permission = context.required_permission.as_ref();
        if require_exact_permission && route_permission.is_none() {
            tracing::warn!(
                subject,
                "authenticated request has no exact route permission"
            );
            return Ok(RequestValidation::denied());
        }

        // Resolve the presented context digest through the provider cache;
        // cache hits are memory-only, unknown digests resolve once through the
        // connected NATS authorization registry.
        let Some(provider) = self.provider.as_ref() else {
            tracing::warn!(subject, "local authorization provider unavailable");
            return Ok(RequestValidation::denied());
        };
        if !provider
            .health()
            .map(|health| health.healthy)
            .unwrap_or(false)
        {
            tracing::warn!(subject, "local authorization provider is not ready");
            return Ok(RequestValidation::denied());
        }
        let policy = match provider.policy() {
            Ok(policy) => policy,
            Err(error) => {
                tracing::warn!(subject, %error, "local authorization policy unavailable");
                return Ok(RequestValidation::denied());
            }
        };
        let verified = match provider.verified_context_raw(&authorization_context) {
            Ok(Some(verified)) => verified,
            _ => match provider
                .resolve_context(&authorization_context, policy.now_unix_seconds)
                .await
            {
                Ok(verified) => verified,
                Err(error) => {
                    tracing::warn!(subject, %error, "local authorization context unavailable");
                    return Ok(RequestValidation::denied());
                }
            },
        };
        // Memory-only revocation read: the provider watch applies revocations
        // immediately and never un-revokes a digest. A revoked context denies.
        match provider.revocation_time(&authorization_context) {
            Ok(Some(_)) => {
                return Ok(RequestValidation::denied());
            }
            Ok(None) => {}
            Err(error) => {
                tracing::warn!(subject, %error, "local revocation evidence unavailable");
                return Ok(RequestValidation::denied());
            }
        }
        let permission = match route_permission
            .map(RoutePermission::permission_atom)
            .transpose()
        {
            Ok(permission) => permission,
            Err(error) => {
                tracing::debug!(subject, %error, "invalid route permission");
                return Ok(RequestValidation::denied());
            }
        };
        let required_permissions = permission.as_slice();
        let verified = match self.verification.verify_request(
            &verified,
            &session_key,
            &authorization_context,
            subject,
            payload,
            iat,
            &request_id,
            Some(&reply),
            &proof,
            &policy,
            required_permissions,
            &required_capabilities,
        ) {
            Ok(verified) => verified,
            Err(error) => {
                tracing::debug!(subject, %error, "local request proof rejected");
                return Ok(RequestValidation::denied());
            }
        };
        let caller = verified.caller().clone();
        Ok(RequestValidation {
            allowed: true,
            caller: Some(caller),
            inbox_prefix: Some(verified.caller().inbox_prefix.clone()),
        })
    }

    /// Verify a v1 event proof against the digest-keyed context, historical
    /// policy, and the publisher's exact `Publish` atom for `event_name`.
    ///
    /// `event_name` is the API-local event name derived from the registered
    /// event descriptor, never parsed from the subject on the hot path.
    pub(crate) async fn verify_event(
        &self,
        subject: &str,
        payload: &[u8],
        headers: Option<&async_nats::header::HeaderMap>,
        event_api_id: &str,
        event_name: &str,
        required_capabilities: &[String],
    ) -> Result<super::runtime_facade::ServiceEventPublisherContext, EventVerificationFailure> {
        let Some(headers) = headers else {
            return Err(EventVerificationFailure::rejected(
                "missing event proof headers",
            ));
        };
        let session_key = required_event_header(headers, "session-key")?;
        let proof = required_event_header(headers, "proof")?;
        let authorization_context = required_event_header(headers, "authorization-context")?;
        let event_id = required_event_header(headers, "Nats-Msg-Id")?;
        let event_time = required_event_header(headers, "Trellis-Event-Time")?;

        // Historical resolution retains signed contexts after expiry; the
        // strict eventTime window is enforced below by the protocol verifier.
        let Some(provider) = self.provider.as_ref() else {
            tracing::warn!(subject, "local authorization provider unavailable");
            return Err(EventVerificationFailure::retryable(format!(
                "local authorization context unavailable for {subject}"
            )));
        };
        if !provider
            .health()
            .map(|health| health.healthy)
            .unwrap_or(false)
        {
            tracing::warn!(subject, "local authorization provider is not ready");
            return Err(EventVerificationFailure::retryable(format!(
                "local authorization context unavailable for {subject}"
            )));
        }
        let policy = match provider.policy() {
            Ok(policy) => policy,
            Err(error) => {
                tracing::warn!(subject, %error, "local event policy unavailable");
                return Err(EventVerificationFailure::retryable(format!(
                    "local event policy unavailable for {subject}"
                )));
            }
        };
        let context = match provider.verified_context_raw(&authorization_context) {
            Ok(Some(context)) => context,
            _ => match provider
                .resolve_event_context_for_verification(
                    &authorization_context,
                    time::OffsetDateTime::parse(
                        &event_time,
                        &time::format_description::well_known::Rfc3339,
                    )
                    .map(|value| value.unix_timestamp())
                    .unwrap_or(policy.now_unix_seconds),
                )
                .await
            {
                Ok(context) => context,
                Err(error) => {
                    tracing::warn!(subject, %error, "local event context resolution failed");
                    return Err(error);
                }
            },
        };
        // Memory-only revocation read; the provider watch applies revocations
        // immediately. A revoked context denies.
        let revoked_at = match provider.revocation_time(&authorization_context) {
            Ok(Some(revoked_at)) => Some(revoked_at),
            Ok(None) => None,
            Err(error) => {
                tracing::warn!(subject, %error, "local revocation evidence unavailable");
                return Err(EventVerificationFailure::retryable(format!(
                    "local revocation evidence unavailable for {subject}"
                )));
            }
        };
        // The caller supplies the API identity and event name from its
        // precompiled descriptor; the verifier never infers either from the
        // transport subject or the connected service's own participant.
        let permission = match select_publish_permission(event_api_id, event_name) {
            Ok(permission) => permission,
            Err(error) => {
                tracing::debug!(subject, %error, "local event permission rejected");
                return Err(EventVerificationFailure::rejected(
                    "invalid event proof signature",
                ));
            }
        };
        let verified_event = self
            .verification
            .verify_event(
                &context,
                &session_key,
                &authorization_context,
                subject,
                payload,
                &event_id,
                &event_time,
                &proof,
                &policy,
                &[permission],
                required_capabilities,
                revoked_at,
            )
            .map_err(|error| {
                tracing::debug!(subject, %error, "local event proof rejected");
                EventVerificationFailure::rejected(format!("invalid event proof: {error}"))
            })?;
        let publisher = verified_event.publisher();
        Ok(super::runtime_facade::ServiceEventPublisherContext {
            kind: publisher.kind.clone(),
            deployment_id: publisher.deployment_id.clone(),
            instance_id: publisher.instance_id.clone(),
            contract_id: Some(publisher.participant_id.clone()),
            contract_digest: Some(publisher.participant_digest.clone()),
            session_status: "active".to_owned(),
        })
    }
}

impl RequestValidator for LocalAuthVerifier {
    fn validate<'a>(
        &'a self,
        subject: &'a str,
        payload: &'a Bytes,
        context: &'a RequestContext,
    ) -> BoxFuture<'a, Result<RequestValidation, ServerError>> {
        Box::pin(async move { self.verify_request(subject, payload, context).await })
    }

    fn validate_possession<'a>(
        &'a self,
        subject: &'a str,
        payload: &'a Bytes,
        context: &'a RequestContext,
    ) -> BoxFuture<'a, Result<RequestValidation, ServerError>> {
        Box::pin(async move {
            self.verify_request_inner(subject, payload, context, false)
                .await
        })
    }
}

/// Two-way durable event verification outcome used to choose redelivery.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum EventVerificationFailure {
    /// Authorization infrastructure is temporarily unavailable; redelivery is required.
    Retryable(String),
    /// The signed event is permanently invalid or unauthorized.
    Rejected(String),
}

impl EventVerificationFailure {
    pub(crate) fn retryable(message: impl Into<String>) -> Self {
        Self::Retryable(message.into())
    }

    pub(crate) fn rejected(message: impl Into<String>) -> Self {
        Self::Rejected(message.into())
    }

    /// Return the diagnostic message without changing its retry classification.
    #[must_use]
    pub fn message(&self) -> &str {
        match self {
            Self::Retryable(message) | Self::Rejected(message) => message,
        }
    }
}

impl std::fmt::Display for EventVerificationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.message())
    }
}

fn required_event_header(
    headers: &async_nats::header::HeaderMap,
    name: &str,
) -> Result<String, EventVerificationFailure> {
    headers
        .get(name)
        .map(|value| value.as_str().to_string())
        .filter(|value| !value.is_empty())
        .ok_or_else(|| EventVerificationFailure::rejected(format!("missing event {name} header")))
}

/// Canonical base64url SHA-256 payload digest used by request/event proofs.
#[doc = concat!("Trellis API operation `", stringify!(payload_hash_base64url), "`.")]
pub fn payload_hash_base64url(payload: &[u8]) -> String {
    use base64::Engine as _;
    use sha2::Digest as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(sha2::Sha256::digest(payload))
}

fn select_publish_permission(
    api_id: &str,
    event_name: &str,
) -> Result<PermissionAtomV1, ProtocolError> {
    PermissionAtomV1::new(
        PermissionTargetV1::api_surface(api_id, ApiSurfaceKindV1::Event, event_name)?,
        PermissionActionV1::Publish,
    )
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    use async_nats::header::HeaderMap;
    use ed25519_dalek::SigningKey;
    use serde::Deserialize;
    use trellis_protocol::{
        canonicalize_json, parse_authorization_context_v1, parse_issuer_manifest_v1,
        sign_authorization_context_v1, verify_authorization_context_v1, verify_issuer_manifest_v1,
        ApiSurfaceKindV1, AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1, GrantSetV1,
        PermissionActionV1, PermissionAtomV1, PermissionTargetV1, SignedAuthorizationContextV1,
        VerifiedAuthorizationContextV1,
    };

    use super::*;
    use crate::client::{
        AuthorizationContextBundle, AuthorizationContextCache, AuthorizationTrustBundle,
        SessionAuth,
    };

    #[derive(Debug, Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct ChainFixture {
        root_canonical_json: String,
        manifest_canonical_json: String,
        context_canonical_json: String,
        context_digest: String,
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

    fn fixture() -> (ChainFixture, VectorDefaults) {
        let fixture_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/authorization-context/vectors.json");
        let value: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(fixture_path).unwrap()).unwrap();
        (
            serde_json::from_value(value["completeChain"].clone()).unwrap(),
            serde_json::from_value(value["defaults"].clone()).unwrap(),
        )
    }

    fn context(chain: &ChainFixture) -> SignedAuthorizationContextV1 {
        parse_authorization_context_v1(
            &serde_json::from_str(&chain.context_canonical_json).unwrap(),
        )
        .unwrap()
    }

    fn update_context(chain: &mut ChainFixture, context: &SignedAuthorizationContextV1) {
        chain.context_canonical_json =
            canonicalize_json(&serde_json::to_value(context).unwrap()).unwrap();
        chain.context_digest = context.digest().unwrap();
    }

    fn fixture_verification(
        chain: &ChainFixture,
        defaults: &VectorDefaults,
    ) -> (
        VerifiedAuthorizationContextV1,
        AuthorizationVerificationPolicyV1,
        AuthorizationContextBundle,
    ) {
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
        let context = context(chain);
        let verified =
            verify_authorization_context_v1(&root, &verified_manifest, &context, &policy).unwrap();
        assert_eq!(verified.context_digest(), chain.context_digest);
        let bundle = AuthorizationContextBundle {
            context: serde_json::from_str(&chain.context_canonical_json).unwrap(),
            trust: AuthorizationTrustBundle {
                root: serde_json::from_str(&chain.root_canonical_json).unwrap(),
                manifest: serde_json::from_str(&chain.manifest_canonical_json).unwrap(),
                authorization_registry: crate::client::AuthorizationRegistryBinding {
                    trust_bucket: "trust".to_owned(),
                    context_bucket: "contexts".to_owned(),
                },
                policy: crate::client::AuthorizationTrustPolicy {
                    allowed_clock_skew_seconds: defaults.policy.allowed_clock_skew_seconds,
                    maximum_context_lifetime_seconds: defaults
                        .policy
                        .maximum_context_lifetime_seconds,
                    maximum_context_bytes: defaults.policy.maximum_context_bytes,
                    maximum_permissions: defaults.policy.maximum_permissions,
                    maximum_capabilities: defaults.policy.maximum_capabilities,
                    refresh_lead_seconds: 1,
                    refresh_jitter_seconds: 0,
                },
            },
        };
        (verified, policy, bundle)
    }

    fn real_now_seconds() -> i64 {
        i64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        )
        .unwrap()
    }

    fn verifier_at(
        chain: &ChainFixture,
        defaults: &VectorDefaults,
        revoked_at: Option<i64>,
        now: i64,
    ) -> LocalAuthVerifier {
        use crate::client::{inject_own_verified_for_test, AuthorizationProviderCache};
        let (verified, policy, bundle) = fixture_verification(chain, defaults);
        let cache =
            AuthorizationContextCache::ephemeral("http://trellis.test", "test-binding").unwrap();
        let offset_ms = now
            .saturating_mul(1_000)
            .saturating_sub(real_now_seconds().saturating_mul(1_000));
        cache.set_server_clock_offset_ms(offset_ms);
        inject_own_verified_for_test(&cache, bundle, verified.clone(), policy.clone()).unwrap();
        let input = cache.provider_trust_input().unwrap();
        let root = AuthorizationTrustRootV1::parse(
            &serde_json::from_str(&chain.root_canonical_json).unwrap(),
        )
        .unwrap();
        let manifest = parse_issuer_manifest_v1(
            &serde_json::from_str(&chain.manifest_canonical_json).unwrap(),
        )
        .unwrap();
        let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &policy).unwrap();
        let provider = AuthorizationProviderCache::new_for_test(
            std::sync::Arc::new(cache),
            input,
            root,
            verified_manifest,
            manifest.digest().unwrap(),
        )
        .unwrap();
        provider
            .inject_verified_for_test(&chain.context_digest, verified, revoked_at)
            .unwrap();
        LocalAuthVerifier::new(Some(provider), "documents@v1")
    }

    fn event_fixture() -> (ChainFixture, VectorDefaults) {
        let (mut chain, defaults) = fixture();
        let mut context = context(&chain);
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
        let mut permissions = context.unsigned.grant_set.permissions().to_vec();
        permissions.push(publish);
        context.unsigned.grant_set = GrantSetV1::new(permissions);
        let context =
            sign_authorization_context_v1(context.unsigned, &SigningKey::from_bytes(&[2; 32]))
                .unwrap();
        update_context(&mut chain, &context);
        (chain, defaults)
    }

    fn signer(chain: &ChainFixture) -> SessionAuth {
        SessionAuth::from_seed_base64url(&chain.session_seed).unwrap()
    }

    #[allow(clippy::too_many_arguments)] // Keeps proof-vector inputs explicit in tests.
    fn request_context(
        subject: &str,
        reply: &str,
        _payload: &[u8],
        proof: &str,
        session_key: &str,
        context_digest: &str,
        iat: i64,
        request_id: &str,
        required_capabilities: Vec<String>,
    ) -> RequestContext {
        RequestContext {
            subject: subject.to_string(),
            session_key: Some(session_key.to_string()),
            proof: Some(proof.to_string()),
            authorization_context: Some(context_digest.to_string()),
            iat: Some(iat),
            request_id: Some(request_id.to_string()),
            required_capabilities: Some(required_capabilities),
            required_permission: Some(RoutePermission {
                api: "documents@v1".to_owned(),
                surface: ApiSurfaceKindV1::Rpc,
                name: "Documents.Get".to_owned(),
                action: PermissionActionV1::Call,
                signal: None,
            }),
            reply_to: Some(reply.to_string()),
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn cross_context_request_is_verified_locally() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let proof = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);

        let validation = verifier
            .verify_request(
                &defaults.request.subject,
                payload,
                &request_context(
                    &defaults.request.subject,
                    &defaults.request.reply,
                    payload,
                    proof.as_str(),
                    &auth.session_key,
                    &chain.context_digest,
                    defaults.request.iat,
                    &defaults.request.request_id,
                    Vec::new(),
                ),
            )
            .await
            .unwrap();
        assert!(validation.allowed);
        let caller = validation.caller.expect("verified caller");
        assert_eq!(caller.session_key, auth.session_key);
        assert_eq!(caller.inbox_prefix, "_INBOX.test");
        assert_eq!(caller.context_digest, chain.context_digest);
        assert_eq!(caller.session_id, "ses_test");
        assert_eq!(caller.participant.id, "documents-web");
        assert_eq!(validation.inbox_prefix.as_deref(), Some("_INBOX.test"));

        let altered = request_context(
            &defaults.request.subject,
            "_INBOX.other.reply",
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(
            !verifier
                .verify_request(&defaults.request.subject, payload, &altered)
                .await
                .unwrap()
                .allowed
        );
        // A proof bound to one subject is denied when presented on another.
        let unknown_subject = request_context(
            "rpc.v1.Documents.Delete",
            &defaults.request.reply,
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(
            !verifier
                .verify_request("rpc.v1.Documents.Delete", payload, &unknown_subject)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn missing_exact_permission_is_denied() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let proof = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let mut context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        context.required_permission = Some(RoutePermission {
            api: "documents@v1".to_owned(),
            surface: ApiSurfaceKindV1::Rpc,
            name: "Documents.Delete".to_owned(),
            action: PermissionActionV1::Call,
            signal: None,
        });
        assert!(
            !verifier
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn duplicate_request_id_remains_accepted() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let proof = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(
            verifier
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
        // A repeated signed request remains valid and reaches the handler again.
        assert!(
            verifier
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn forged_proof_does_not_deny_valid_use() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let forged = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                "different-request-id",
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let forged_context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            forged.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(
            !verifier
                .verify_request(&defaults.request.subject, payload, &forged_context)
                .await
                .unwrap()
                .allowed
        );

        let valid = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let valid_context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            valid.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(
            verifier
                .verify_request(&defaults.request.subject, payload, &valid_context)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn expired_and_revoked_contexts_deny() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let proof = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        // The context window is [1100, 1300); outside it the cache fails closed.
        let expired = verifier_at(&chain, &defaults, None, 1_301);
        assert!(
            !expired
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
        // Revocation denies requests at or after the installed revocation time.
        let revoked = verifier_at(&chain, &defaults, Some(1_150), 1_200);
        assert!(
            !revoked
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn unknown_capability_is_denied() {
        let (chain, defaults) = fixture();
        let auth = signer(&chain);
        let payload = defaults.request.payload.as_bytes();
        let proof = auth
            .create_request_proof(
                &chain.context_digest,
                &defaults.request.subject,
                &defaults.request.reply,
                payload,
                defaults.request.iat,
                &defaults.request.request_id,
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let context = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            payload,
            proof.as_str(),
            &auth.session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            vec!["missing.capability".to_owned()],
        );
        assert!(
            !verifier
                .verify_request(&defaults.request.subject, payload, &context)
                .await
                .unwrap()
                .allowed
        );
    }

    #[tokio::test]
    async fn missing_headers_fail_closed() {
        let (chain, defaults) = fixture();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let missing_proof = request_context(
            &defaults.request.subject,
            &defaults.request.reply,
            defaults.request.payload.as_bytes(),
            "",
            &signer(&chain).session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(matches!(
            verifier
                .verify_request(
                    &defaults.request.subject,
                    defaults.request.payload.as_bytes(),
                    &missing_proof
                )
                .await,
            Err(ServerError::MissingProof { .. })
        ));
        let missing_reply = request_context(
            &defaults.request.subject,
            "",
            defaults.request.payload.as_bytes(),
            "proof",
            &signer(&chain).session_key,
            &chain.context_digest,
            defaults.request.iat,
            &defaults.request.request_id,
            Vec::new(),
        );
        assert!(matches!(
            verifier
                .verify_request(
                    &defaults.request.subject,
                    defaults.request.payload.as_bytes(),
                    &missing_reply
                )
                .await,
            Err(ServerError::MissingReply { .. })
        ));
    }

    #[tokio::test]
    async fn valid_event_and_historical_revocation_window() {
        let (chain, defaults) = event_fixture();
        let auth = signer(&chain);
        let payload = br#"{"id":"doc-1"}"#;
        let subject = "events.v1.Documents.Changed.doc-1";
        let event_id = "evt_doc_1";
        let event_time = "1970-01-01T00:19:10Z";
        let proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                event_id,
                event_time,
            )
            .unwrap();

        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let publisher = verifier
            .verify_event(
                subject,
                payload,
                Some(&event_headers(
                    &auth,
                    &chain,
                    proof.as_str(),
                    event_id,
                    event_time,
                )),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .expect("verified publisher");
        assert_eq!(publisher.kind, "user");
        assert_eq!(publisher.contract_id.as_deref(), Some("documents-web"));
        assert_eq!(publisher.session_status, "active");
        assert_eq!(verifier.resolver_count_for_test(), 0);

        // Trust verification is historical: delivery after context expiry is
        // allowed when the signed event time is still inside the context window.
        let historical = verifier_at(&chain, &defaults, None, 1_400);
        let historical_time = "1970-01-01T00:19:10Z";
        let historical_proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                "evt_expired_delivery",
                historical_time,
            )
            .unwrap();
        historical
            .verify_event(
                subject,
                payload,
                Some(&event_headers(
                    &auth,
                    &chain,
                    historical_proof.as_str(),
                    "evt_expired_delivery",
                    historical_time,
                )),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .expect("historical event remains eligible");

        // Revocation invalidates all event proofs from the context, including replays.
        let revoked = verifier_at(&chain, &defaults, Some(1_150), 1_100);
        let before_proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                "evt_before",
                "1970-01-01T00:19:00Z",
            )
            .unwrap();
        assert!(revoked
            .verify_event(
                subject,
                payload,
                Some(&event_headers(
                    &auth,
                    &chain,
                    before_proof.as_str(),
                    "evt_before",
                    "1970-01-01T00:19:00Z",
                )),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .is_err());
        let boundary_time = "1970-01-01T00:19:10Z";
        let boundary_proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                "evt_revoked_boundary",
                boundary_time,
            )
            .unwrap();
        assert!(revoked
            .verify_event(
                subject,
                payload,
                Some(&event_headers(
                    &auth,
                    &chain,
                    boundary_proof.as_str(),
                    "evt_revoked_boundary",
                    boundary_time,
                )),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .is_err());
        let after_proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                "evt_after",
                "1970-01-01T00:19:30Z",
            )
            .unwrap();
        assert!(revoked
            .verify_event(
                subject,
                payload,
                Some(&event_headers(
                    &auth,
                    &chain,
                    after_proof.as_str(),
                    "evt_after",
                    "1970-01-01T00:19:30Z",
                )),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .is_err());
    }

    #[tokio::test]
    async fn duplicate_event_id_remains_accepted() {
        let (chain, defaults) = event_fixture();
        let auth = signer(&chain);
        let payload = br#"{"id":"doc-1"}"#;
        let subject = "events.v1.Documents.Changed.doc-1";
        let event_id = "evt_doc_1";
        let event_time = "1970-01-01T00:19:10Z";
        let proof = auth
            .create_event_proof(
                &chain.context_digest,
                subject,
                payload,
                event_id,
                event_time,
            )
            .unwrap();
        let verifier = verifier_at(&chain, &defaults, None, 1_100);
        let headers = event_headers(&auth, &chain, proof.as_str(), event_id, event_time);
        verifier
            .verify_event(
                subject,
                payload,
                Some(&headers),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .expect("first delivery");
        // A redelivered signed event remains valid and reaches the handler again.
        verifier
            .verify_event(
                subject,
                payload,
                Some(&headers),
                "documents@v1",
                "Documents.Changed",
                &[],
            )
            .await
            .expect("duplicate event id remains accepted");
    }

    fn event_headers(
        auth: &SessionAuth,
        chain: &ChainFixture,
        proof: &str,
        event_id: &str,
        event_time: &str,
    ) -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("session-key", auth.session_key.as_str());
        headers.insert("proof", proof);
        headers.insert("authorization-context", chain.context_digest.as_str());
        headers.insert("Nats-Msg-Id", event_id);
        headers.insert("Trellis-Event-Time", event_time);
        headers
    }
}
