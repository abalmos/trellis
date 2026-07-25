use std::{cmp, sync::Arc};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::{json, Map, Value};
use sha2::{Digest as _, Sha256};
use trellis_protocol::{
    authorization_context_refresh_at_v1, canonicalize_json, encode_authorization_context_token_v1,
    parse_authorization_context_v1, sign_authorization_context_v1, UnsignedAuthorizationContextV1,
    AUTHORIZATION_CONTEXT_FORMAT_V1,
};
use ulid::Ulid;

use super::{
    trust::VerifiedTrustMaterial, AuthorizationContextBundle, AuthorizationContextCommit,
    AuthorizationContextRecord, AuthorizationContextRegistry, AuthorizationContextRepository,
    AuthorizationContextState, AuthorizationTrustBundle, AuthorizationTrustStateRecord,
    AuthorizationValidatorCache,
};
use crate::{
    config::AuthorizationConfig,
    platform::auth::{
        repository::{
            issuance_snapshot_token, AuthorizationMaterializationRepository, IssuanceSnapshotToken,
        },
        AuthorizationStateError, IdempotencyResultRecord, IdempotentOutcome,
        SqliteAuthorizationStore,
    },
};

const SNAPSHOT_ATTEMPTS: usize = 3;

#[derive(Clone, Debug)]
pub(crate) struct AuthorizationContextIssueRequest {
    pub(crate) session_id: String,
    pub(crate) request_id: String,
    pub(crate) request_digest: String,
}

#[derive(Clone)]
pub(crate) struct AuthorizationContextService {
    repository: Arc<SqliteAuthorizationStore>,
    trust: Arc<VerifiedTrustMaterial>,
    registry: AuthorizationContextRegistry,
    validator_cache: AuthorizationValidatorCache,
    trust_bundle: AuthorizationTrustBundle,
    config: AuthorizationConfig,
}

impl AuthorizationContextService {
    pub(crate) fn manifest_generation(&self) -> u64 {
        self.trust.verified_manifest.generation()
    }

    pub(crate) fn root_key_id(&self) -> &str {
        self.trust.root.key_id()
    }

    pub(crate) async fn read_trust_registry(
        &self,
        key: &str,
    ) -> Result<Option<bytes::Bytes>, AuthorizationStateError> {
        self.registry.get_trust(key).await
    }

    pub(crate) async fn read_context_registry(
        &self,
        digest: &str,
    ) -> Result<Option<bytes::Bytes>, AuthorizationStateError> {
        self.registry.get_context(digest).await
    }

    pub(crate) async fn read_revocation_snapshot(
        &self,
    ) -> Result<Vec<super::AuthorizationContextRevocationV1>, AuthorizationStateError> {
        self.registry.list_revocations().await
    }

    pub(crate) async fn run_janitor(
        self,
        stop: crate::shutdown::StopHandle,
    ) -> Result<(), crate::supervisor::RuntimeError> {
        const BATCH: usize = 256;
        loop {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?
                .as_secs();
            let now = i64::try_from(now).map_err(|_| {
                crate::supervisor::RuntimeError::Platform(
                    "context janitor time overflow".to_owned(),
                )
            })?;
            let expired = self
                .repository
                .expire_contexts(now)
                .await
                .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
            let cleanup_grace = i64::try_from(self.config.cleanup_grace_seconds).map_err(|_| {
                crate::supervisor::RuntimeError::Platform(
                    "context cleanup grace overflow".to_owned(),
                )
            })?;
            let before = now.saturating_sub(cleanup_grace);
            let deleted = self
                .repository
                .delete_terminal_contexts(before, BATCH)
                .await
                .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))?;
            if !expired.is_empty() || deleted > 0 {
                tracing::debug!(
                    expired = expired.len(),
                    deleted,
                    "authorization context janitor completed"
                );
            }
            tokio::select! {
                () = stop.stopped() => return Ok(()),
                () = tokio::time::sleep(std::time::Duration::from_secs(30)) => {}
            }
        }
    }

    pub(crate) async fn run_validator_cache(
        self,
        stop: crate::shutdown::StopHandle,
    ) -> Result<(), crate::supervisor::RuntimeError> {
        self.validator_cache
            .run(stop)
            .await
            .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))
    }

    pub(crate) async fn wait_for_validator_cache(
        &self,
    ) -> Result<(), crate::supervisor::RuntimeError> {
        tokio::time::timeout(std::time::Duration::from_secs(30), async {
            loop {
                if self.validator_cache.health()?.healthy {
                    return Ok(());
                }
                tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            }
        })
        .await
        .map_err(|_| {
            crate::supervisor::RuntimeError::Platform(
                "authorization validator cache did not become ready".to_owned(),
            )
        })?
        .map_err(|error: AuthorizationStateError| {
            crate::supervisor::RuntimeError::Platform(error.to_string())
        })
    }

    pub(crate) async fn require_current_context(
        &self,
        session_id: &str,
        context_digest: &str,
        now_seconds: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        let manifest = self.validator_cache.current_manifest()?;
        let context = self
            .repository
            .get_context_by_digest(context_digest)
            .await?
            .filter(|context| {
                context.session_id == session_id
                    && context.state == AuthorizationContextState::Active
                    && context.published_at.is_some()
                    && context.not_before <= now_seconds
                    && context.expires_at > now_seconds
                    && manifest
                        .active_certificate_digest(&context.issuer_key_id)
                        .is_some()
            })
            .ok_or(AuthorizationStateError::AuthorityStale)?;
        Ok(context)
    }

    pub(crate) async fn require_issuable_context(
        &self,
        authorization: &crate::platform::auth::IssuableAuthorizationState,
        context_digest: &str,
        now_seconds: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        if !self.validator_cache.health()?.healthy {
            return Err(AuthorizationStateError::AuthorityStale);
        }
        let record = self
            .require_current_context(&authorization.session_id, context_digest, now_seconds)
            .await?;
        let signed_context_json = serde_json::from_str(&record.signed_context_json)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        let signed_context = parse_authorization_context_v1(&signed_context_json)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        let verified = self
            .validator_cache
            .cache_context(signed_context, now_seconds)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        let context = &verified.signed_context().unsigned;
        if verified.context_digest() != context_digest
            || context.session_id != authorization.session_id
            || context.session_key != authorization.session_public_key
            || context.principal != authorization.principal
            || context.participant != authorization.participant
            || context.authority_ref != authorization.authority_ref
            || context.deployment_id != authorization.deployment_id
            || context.instance_id != authorization.instance_id
            || context.inbox_prefix != authorization.inbox_prefix
            || context.grant_set != authorization.grant_set
            || context.capabilities != authorization.capabilities
        {
            return Err(AuthorizationStateError::AuthorityStale);
        }
        Ok(record)
    }

    pub(crate) async fn start(
        repository: Arc<SqliteAuthorizationStore>,
        nats: async_nats::Client,
        config: AuthorizationConfig,
        now_seconds: i64,
    ) -> Result<Self, AuthorizationStateError> {
        let trust = Arc::new(
            VerifiedTrustMaterial::load(&config, now_seconds)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
        );
        let manifest_digest = trust
            .manifest
            .digest()
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        let root_digest = trust
            .root
            .digest()
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        let registry = AuthorizationContextRegistry::ensure(nats, &config).await?;
        let existing = repository.get_trust_state().await?;
        let registry_floor = registry.trust_floor(&trust).await?;
        super::registry::reconcile_trust_floors(
            &trust,
            existing.as_ref(),
            registry_floor.as_ref(),
        )?;
        // Immutable evidence may be safely published before either monotonic floor advances.
        let trust_bundle = registry.publish_trust_immutables(&trust, &config).await?;
        let state = AuthorizationTrustStateRecord {
            authority: trust.root.authority().to_owned(),
            root_key_id: trust.root.key_id().to_owned(),
            root_digest,
            manifest_generation: trust.verified_manifest.generation(),
            manifest_digest,
            active_issuer_key_id: trust.active_certificate.unsigned.key_id.clone(),
            updated_at: seconds_to_millis(now_seconds)?,
            version: existing.as_ref().map_or(1, |record| {
                if record.manifest_generation == trust.verified_manifest.generation()
                    && record.manifest_digest
                        == trust.manifest.digest().unwrap_or_else(|_| String::new())
                {
                    record.version
                } else {
                    record.version.saturating_add(1)
                }
            }),
        };
        let removed_issuer_key_ids = repository
            .list_active_issuer_key_ids()
            .await?
            .into_iter()
            .filter(|issuer_key_id| {
                trust
                    .verified_manifest
                    .active_certificate_digest(issuer_key_id)
                    .is_none()
            })
            .collect();
        repository
            .accept_trust_state(state, removed_issuer_key_ids, now_seconds)
            .await?;
        registry.advance_trust_pointer(&trust_bundle).await?;
        let validator_cache = AuthorizationValidatorCache::new(registry.clone(), &trust)?;
        let service = Self {
            repository,
            trust,
            registry,
            validator_cache,
            trust_bundle,
            config,
        };
        service.repair_unpublished(now_seconds).await?;
        let mut after = None;
        loop {
            let contexts = service
                .repository
                .list_revoked_contexts(after.as_deref(), 256)
                .await?;
            if contexts.is_empty() {
                break;
            }
            for context in &contexts {
                service.publish_revocation(context).await?;
            }
            after = contexts.last().map(|context| context.context_id.clone());
        }
        Ok(service)
    }

    pub(crate) async fn issue(
        &self,
        request: AuthorizationContextIssueRequest,
        now_seconds: i64,
    ) -> Result<AuthorizationContextBundle, AuthorizationStateError> {
        for _ in 0..SNAPSHOT_ATTEMPTS {
            match self.issue_once(&request, now_seconds).await {
                Err(AuthorizationStateError::StorageConflict) => continue,
                result => return result,
            }
        }
        Err(AuthorizationStateError::ContextSnapshotChanged)
    }

    pub(crate) async fn repair_unpublished(
        &self,
        now_seconds: i64,
    ) -> Result<(), AuthorizationStateError> {
        self.repository.expire_contexts(now_seconds).await?;
        loop {
            let contexts = self.repository.list_unpublished_contexts(256).await?;
            if contexts.is_empty() {
                break;
            }
            for context in contexts {
                self.publish(&context, now_seconds).await?;
            }
        }
        Ok(())
    }

    pub(crate) async fn publish_revocation(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<(), AuthorizationStateError> {
        self.registry.publish_revocation(context).await
    }

    pub(crate) async fn dispatch_registry_action(
        &self,
        digest: &str,
        revocation: bool,
        now_seconds: i64,
    ) -> Result<(), AuthorizationStateError> {
        let context = self
            .repository
            .get_context_by_digest(digest)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::Storage("context action is missing".to_owned())
            })?;
        if revocation {
            self.registry.publish_revocation(&context).await
        } else if context.expires_at <= now_seconds {
            Ok(())
        } else {
            self.publish(&context, now_seconds).await.map(|_| ())
        }
    }

    async fn issue_once(
        &self,
        request: &AuthorizationContextIssueRequest,
        now_seconds: i64,
    ) -> Result<AuthorizationContextBundle, AuthorizationStateError> {
        let now_millis = seconds_to_millis(now_seconds)?;
        let snapshot = self
            .repository
            .load_issuance_snapshot(&request.session_id)
            .await?;
        let snapshot_token = issuance_snapshot_token(&snapshot)?;
        let authorization = super::super::service::resolve_snapshot(snapshot.clone(), now_millis)?;
        let expires_at = context_expiry(&authorization, &self.trust, &self.config, now_seconds)?;

        if authorization.grant_set.permissions().len() > self.config.maximum_permissions
            || authorization.capabilities.len() > self.config.maximum_capabilities
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "authorization context exceeds configured bounds".to_owned(),
            ));
        }
        let context_id = format!("ctx_{}", Ulid::new().to_string().to_ascii_lowercase());
        let unsigned = UnsignedAuthorizationContextV1 {
            format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
            authority: self.trust.root.authority().to_owned(),
            context_id: context_id.clone(),
            issuer_key_id: self.trust.active_certificate.unsigned.key_id.clone(),
            session_id: authorization.session_id.clone(),
            session_key: authorization.session_public_key.clone(),
            principal: authorization.principal.clone(),
            participant: authorization.participant.clone(),
            authority_ref: authorization.authority_ref.clone(),
            deployment_id: authorization.deployment_id.clone(),
            instance_id: authorization.instance_id.clone(),
            inbox_prefix: authorization.inbox_prefix.clone(),
            issued_at: now_seconds,
            not_before: cmp::max(
                now_seconds
                    - i64::try_from(self.config.allowed_clock_skew_seconds).map_err(|_| {
                        AuthorizationStateError::InvalidRecord("clock skew is too large".to_owned())
                    })?,
                cmp::max(
                    self.trust.active_certificate.unsigned.not_before,
                    self.trust.manifest.unsigned.not_before,
                ),
            ),
            expires_at,
            grant_set: authorization.grant_set.clone(),
            capabilities: authorization.capabilities.clone(),
            extensions: Map::new(),
            critical: Vec::new(),
        };
        let signed = sign_authorization_context_v1(unsigned, &self.trust.issuer_signing_key)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let signed_context_json = canonicalize_json(
            &serde_json::to_value(&signed)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        if signed_context_json.len() > self.config.maximum_context_bytes {
            return Err(AuthorizationStateError::InvalidRecord(
                "authorization context canonical JSON is too large".to_owned(),
            ));
        }
        let context_token = encode_authorization_context_token_v1(&signed)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let context_digest = signed
            .digest()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let refresh_at = authorization_context_refresh_at_v1(
            &context_digest,
            now_seconds,
            signed.unsigned.not_before,
            expires_at,
            u32::try_from(self.config.refresh_lead_seconds).map_err(|_| {
                AuthorizationStateError::InvalidRecord("refresh lead is too large".to_owned())
            })?,
            u32::try_from(self.config.refresh_jitter_seconds).map_err(|_| {
                AuthorizationStateError::InvalidRecord("refresh jitter is too large".to_owned())
            })?,
        )
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let context = AuthorizationContextRecord {
            context_id,
            context_digest,
            session_id: authorization.session_id.clone(),
            principal_id: authorization.principal.id.clone(),
            principal_kind: snapshot
                .principal
                .as_ref()
                .ok_or(AuthorizationStateError::PrincipalMissing)?
                .kind,
            participant_id: authorization.participant.id.clone(),
            participant_artifact_digest: authorization.participant.artifact_digest,
            participant_needs_digest: authorization.participant.needs_digest,
            authority_kind: snapshot
                .authority
                .as_ref()
                .ok_or(AuthorizationStateError::AuthorityMissing)?
                .target()
                .kind,
            authority_id: authorization.authority_ref.id,
            authority_version: authorization.authority_ref.version,
            materialization_version: authorization.materialization_version,
            deployment_id: authorization.deployment_id,
            instance_id: authorization.instance_id,
            issuer_key_id: self.trust.active_certificate.unsigned.key_id.clone(),
            signed_context_json,
            context_token,
            issuance_snapshot_token: snapshot_token.0.clone(),
            trust_generation: self.trust.verified_manifest.generation(),
            issued_at: now_seconds,
            not_before: signed.unsigned.not_before,
            expires_at,
            refresh_at,
            state: AuthorizationContextState::Active,
            published_at: None,
            revoked_at: None,
            revocation_reason: None,
            version: 1,
        };
        let commit = AuthorizationContextCommit {
            expected_snapshot_token: snapshot_token.clone(),
            context,
            idempotency: context_idempotency(request, now_millis, expires_at, "pending")?,
            now: now_seconds,
            minimum_remaining_seconds: i64::try_from(self.config.minimum_context_lifetime_seconds)
                .map_err(|_| AuthorizationStateError::ContextLifetimeUnavailable)?,
        };
        let context = match self.repository.commit_context(commit).await? {
            IdempotentOutcome::Applied(context) => context,
            IdempotentOutcome::Replayed(result) => {
                let context_id =
                    result
                        .get("contextId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            AuthorizationStateError::Storage("invalid context replay".to_owned())
                        })?;
                let context = self
                    .repository
                    .get_context_by_id(context_id)
                    .await?
                    .ok_or_else(|| {
                        AuthorizationStateError::Storage("context replay is missing".to_owned())
                    })?;
                require_reusable_context(&context, &snapshot_token, now_seconds)?;
                context
            }
        };
        let context = self.publish(&context, now_seconds).await?;
        tracing::info!(
            context_id = %context.context_id,
            session_id = %context.session_id,
            authority_id = %context.authority_id,
            expires_at = context.expires_at,
            "issued authorization context"
        );
        Ok(self.bundle(&context))
    }

    async fn publish(
        &self,
        context: &AuthorizationContextRecord,
        published_at: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        self.registry.publish_context(context).await?;
        let signed_context_json = serde_json::from_str(&context.signed_context_json)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        let signed_context = parse_authorization_context_v1(&signed_context_json)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        self.validator_cache
            .cache_context(signed_context, published_at)
            .map_err(|_| AuthorizationStateError::AuthorityStale)?;
        if context.published_at.is_some() {
            return Ok(context.clone());
        }
        self.repository
            .mark_context_published(&context.context_id, context.version, published_at)
            .await
    }

    fn bundle(&self, context: &AuthorizationContextRecord) -> AuthorizationContextBundle {
        AuthorizationContextBundle {
            context: context.context_token.clone(),
            context_digest: context.context_digest.clone(),
            refresh_at: context.refresh_at,
            trust: self.trust_bundle.clone(),
        }
    }
}

fn require_reusable_context(
    context: &AuthorizationContextRecord,
    snapshot_token: &IssuanceSnapshotToken,
    now_seconds: i64,
) -> Result<(), AuthorizationStateError> {
    if context.state != AuthorizationContextState::Active
        || context.published_at.is_none()
        || context.expires_at <= now_seconds
        || context.issuance_snapshot_token != snapshot_token.0
    {
        return Err(AuthorizationStateError::ContextSnapshotChanged);
    }
    Ok(())
}

fn context_expiry(
    authorization: &crate::platform::auth::IssuableAuthorizationState,
    trust: &VerifiedTrustMaterial,
    config: &AuthorizationConfig,
    now_seconds: i64,
) -> Result<i64, AuthorizationStateError> {
    let signed_lifetime = config
        .context_lifetime_seconds
        .checked_sub(config.allowed_clock_skew_seconds)
        .ok_or(AuthorizationStateError::ContextLifetimeUnavailable)?;
    let mut expires_at = now_seconds
        .checked_add(i64::try_from(signed_lifetime).map_err(|_| {
            AuthorizationStateError::InvalidRecord("context lifetime is too large".to_owned())
        })?)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("context expiry overflow".to_owned())
        })?;
    for bound in [
        authorization.session_expires_at,
        authorization.effective_authority_expires_at,
        authorization.delegation_expires_at,
    ]
    .into_iter()
    .flatten()
    {
        expires_at = cmp::min(expires_at, bound.div_euclid(1_000));
    }
    expires_at = cmp::min(
        expires_at,
        cmp::min(
            trust.active_certificate.unsigned.expires_at,
            trust.manifest.unsigned.expires_at,
        ),
    );
    let remaining = expires_at - now_seconds;
    if remaining
        < i64::try_from(config.minimum_context_lifetime_seconds)
            .map_err(|_| AuthorizationStateError::ContextLifetimeUnavailable)?
    {
        return Err(AuthorizationStateError::ContextLifetimeUnavailable);
    }
    Ok(expires_at)
}

fn seconds_to_millis(seconds: i64) -> Result<i64, AuthorizationStateError> {
    seconds
        .checked_mul(1_000)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("timestamp overflow".to_owned()))
}

fn context_idempotency(
    request: &AuthorizationContextIssueRequest,
    now_millis: i64,
    expires_at: i64,
    context_id: &str,
) -> Result<IdempotencyResultRecord, AuthorizationStateError> {
    Ok(IdempotencyResultRecord {
        scope_key: URL_SAFE_NO_PAD.encode(Sha256::digest(
            format!(
                "authorization-context:{}:{}",
                request.session_id, request.request_id
            )
            .as_bytes(),
        )),
        purpose: "authorizationContextIssue".to_owned(),
        signer_id: request.session_id.clone(),
        request_id: request.request_id.clone(),
        request_digest: request.request_digest.clone(),
        result: json!({ "contextId": context_id }),
        created_at: now_millis,
        expires_at: seconds_to_millis(expires_at)?,
    })
}
