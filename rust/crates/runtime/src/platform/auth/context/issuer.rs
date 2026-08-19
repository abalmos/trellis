use std::{cmp, sync::Arc};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde_json::{json, Map, Value};
use sha2::{Digest as _, Sha256};
use trellis_protocol::{
    authorization_context_refresh_at_v1, canonicalize_json, sign_authorization_context_v1,
    UnsignedAuthorizationContextV1, AUTHORIZATION_CONTEXT_FORMAT_V1,
};
use trellis_rs::client::{AuthorizationProviderCache, RuntimeAuthorizationTrust};

use super::{
    trust::VerifiedTrustMaterial, AuthorizationContextBundle, AuthorizationContextCommit,
    AuthorizationContextRecord, AuthorizationContextRegistry, AuthorizationContextRepository,
    AuthorizationContextState, AuthorizationRegistryBinding, AuthorizationTrustBundle,
    AuthorizationTrustStateRecord,
};
use crate::{
    config::AuthorizationConfig,
    platform::auth::{
        authority::{
            issuance_snapshot_token, AuthorityRepository, ContextRepository, IssuanceSnapshotToken,
        },
        compile_transport_permissions,
        sqlite::{
            common::{encode_enum, sql_error},
            contexts::decode_resource,
        },
        AuthorizationStateError, IdempotencyResultRecord, IdempotentOutcome,
        SqliteAuthorizationStore, TransportPermissions,
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
    validator_cache: AuthorizationProviderCache,
    trust_bundle: AuthorizationTrustBundle,
    config: AuthorizationConfig,
}

impl AuthorizationContextService {
    /// Clone of the validator cache backing local request/event verification.
    pub(crate) fn validator_cache(&self) -> AuthorizationProviderCache {
        self.validator_cache.clone()
    }

    pub(crate) async fn transport_permissions(
        &self,
        context: &trellis_protocol::VerifiedAuthorizationContextV1,
    ) -> Result<TransportPermissions, AuthorizationStateError> {
        let signed = &context.signed_context().unsigned;
        let durable = self
            .repository
            .get_context_by_digest(context.context_digest())
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "authorization context is missing from durable state".to_owned(),
                )
            })?;
        if durable.state != AuthorizationContextState::Active {
            return Err(AuthorizationStateError::InvalidRecord(
                "authorization context is not active in durable state".to_owned(),
            ));
        }
        let binding = self
            .repository
            .get_participant_binding(&signed.participant.id, &signed.participant.artifact_digest)
            .await?
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "authorization participant binding is missing".to_owned(),
                )
            })?;
        let authority_kind = match signed.authority_ref.kind {
            trellis_protocol::AuthorizationAuthorityKindV1::Identity => {
                crate::platform::auth::AuthorityKind::Identity
            }
            trellis_protocol::AuthorizationAuthorityKindV1::Deployment => {
                crate::platform::auth::AuthorityKind::Deployment
            }
        };
        let authority_id = signed.authority_ref.id.clone();
        let resources = self
            .repository
            .run_read(move |connection| {
                let mut statement = connection
                    .prepare(
                        "SELECT resource.resource_kind, resource.local_name, resource.binding_id,
                                resource.owner_participant_id, resource.provider_identity,
                                resource.state, resource.materialized_at, resource.error
                         FROM auth_materialized_authorities AS authority
                         JOIN auth_materialized_resource_bindings AS resource
                           ON resource.materialization_id = authority.materialization_id
                         WHERE authority.authority_kind = ?1
                           AND authority.authority_id = ?2
                           AND authority.state = 'available'
                         ORDER BY resource.resource_kind, resource.local_name",
                    )
                    .map_err(sql_error)?;
                let resources = statement
                    .query_map(
                        rusqlite::params![encode_enum(authority_kind)?, authority_id],
                        decode_resource,
                    )
                    .map_err(sql_error)?
                    .collect::<rusqlite::Result<Vec<_>>>()
                    .map_err(sql_error)?;
                Ok(resources)
            })
            .await?;
        let permissions = compile_transport_permissions(
            signed,
            &binding,
            &resources,
            &AuthorizationRegistryBinding::from_config(&self.config),
        )?;
        Ok(permissions)
    }

    pub(crate) fn manifest_generation(&self) -> u64 {
        self.trust.verified_manifest.generation()
    }

    pub(crate) fn root_key_id(&self) -> &str {
        self.trust.root.key_id()
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
            .run_runtime({
                let (sender, receiver) = tokio::sync::watch::channel(());
                tokio::spawn(async move {
                    stop.stopped().await;
                    drop(sender);
                });
                receiver
            })
            .await
            .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))
    }

    pub(crate) async fn wait_for_validator_cache(
        &self,
    ) -> Result<(), crate::supervisor::RuntimeError> {
        tokio::time::timeout(
            std::time::Duration::from_secs(30),
            self.validator_cache.wait_until_ready(),
        )
        .await
        .map_err(|_| {
            crate::supervisor::RuntimeError::Platform(
                "authorization validator cache did not become ready".to_owned(),
            )
        })?
        .map_err(|error| crate::supervisor::RuntimeError::Platform(error.to_string()))
    }

    pub(crate) async fn require_current_context(
        &self,
        session_id: &str,
        context_digest: &str,
        now_seconds: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        let manifest = self
            .validator_cache
            .runtime_current_manifest()
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        let context = self
            .repository
            .get_context_by_digest(context_digest)
            .await?
            .ok_or(AuthorizationStateError::AuthorityStale)?;
        let signed = context.signed_context()?;
        if context.session_id != session_id
            || context.state != AuthorizationContextState::Active
            || context.published_at.is_none()
            || signed.unsigned.not_before > now_seconds
            || context.expires_at <= now_seconds
            || !manifest
                .manifest()
                .unsigned
                .issuers
                .iter()
                .any(|issuer| issuer.key_id == context.issuer_key_id)
        {
            return Err(AuthorizationStateError::AuthorityStale);
        }
        Ok(context)
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
        let provider_nats = nats.clone();
        let registry = AuthorizationContextRegistry::ensure(nats, &config).await?;
        let existing = repository.get_trust_state().await?;
        let registry_floor = registry.trust_floor().await?;
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
                !trust
                    .verified_manifest
                    .manifest()
                    .unsigned
                    .issuers
                    .iter()
                    .any(|issuer| issuer.key_id == *issuer_key_id)
            })
            .collect();
        repository
            .accept_trust_state(state, removed_issuer_key_ids, now_seconds)
            .await?;
        registry.advance_trust_pointer(&trust_bundle).await?;
        let registry_binding = AuthorizationRegistryBinding::from_config(&config);
        let provider_binding = trellis_rs::client::AuthorizationRegistryBinding::from_runtime_parts(
            registry_binding.trust_bucket,
            registry_binding.context_bucket,
        );
        let validator_cache = AuthorizationProviderCache::attach_runtime(
            provider_nats,
            &provider_binding,
            RuntimeAuthorizationTrust {
                root: trust.root.clone(),
                policy: trust.policy.clone(),
                minimum_manifest_generation: trust.verified_manifest.generation(),
                minimum_manifest_digest: trust
                    .manifest
                    .digest()
                    .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
                manifest: trust.verified_manifest.clone(),
                manifest_digest: trust
                    .manifest
                    .digest()
                    .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
            },
        )
        .await
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
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
            after = contexts
                .last()
                .map(|context| context.context_digest.clone());
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
            self.registry.publish_revocation(&context).await?;
            self.validator_cache
                .apply_runtime_revocation(
                    digest,
                    context.revoked_at.ok_or_else(|| {
                        AuthorizationStateError::Storage(
                            "revoked context has no revocation time".to_owned(),
                        )
                    })?,
                )
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))
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
        let authorization = super::super::issuance::resolve_snapshot(snapshot.clone(), now_millis)?;
        let expires_at = context_expiry(&authorization, &self.trust, &self.config, now_seconds)?;

        if authorization.grant_set.permissions().len() > self.config.maximum_permissions
            || authorization.capabilities.len() > self.config.maximum_capabilities
        {
            return Err(AuthorizationStateError::InvalidRecord(
                "authorization context exceeds configured bounds".to_owned(),
            ));
        }
        let unsigned = UnsignedAuthorizationContextV1 {
            format: AUTHORIZATION_CONTEXT_FORMAT_V1.to_owned(),
            authority: self.trust.root.authority().to_owned(),
            issuer_key_id: issuer_key_id(&self.trust.issuer_signing_key),
            issuer_manifest_generation: self.trust.verified_manifest.generation(),
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
                self.trust.manifest.unsigned.not_before,
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
            context_digest: context_digest.clone(),
            session_id: authorization.session_id.clone(),
            principal_id: authorization.principal.id.clone(),
            authority_kind: snapshot
                .authority
                .as_ref()
                .ok_or(AuthorizationStateError::AuthorityMissing)?
                .target()
                .kind,
            authority_id: authorization.authority_ref.id,
            deployment_id: authorization.deployment_id.clone(),
            instance_id: authorization.instance_id.clone(),
            issuer_key_id: issuer_key_id(&self.trust.issuer_signing_key),
            issuer_manifest_generation: self.trust.verified_manifest.generation(),
            signed_context_json,
            issuance_snapshot_token: snapshot_token.0.clone(),
            refresh_at,
            expires_at,
            state: AuthorizationContextState::Active,
            published_at: None,
            revoked_at: None,
            revocation_reason: None,
            version: 1,
        };
        let commit = AuthorizationContextCommit {
            expected_snapshot_token: snapshot_token.clone(),
            context,
            idempotency: context_issue_idempotency(
                request,
                now_millis,
                expires_at,
                &context_digest,
            )?,
            now: now_seconds,
            minimum_remaining_seconds: i64::try_from(self.config.minimum_context_lifetime_seconds)
                .map_err(|_| AuthorizationStateError::ContextLifetimeUnavailable)?,
        };
        let context = match self.repository.commit_context(commit).await? {
            IdempotentOutcome::Applied(context) => context,
            IdempotentOutcome::Replayed(result) => {
                let context_digest = result
                    .get("contextDigest")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        AuthorizationStateError::Storage("invalid context replay".to_owned())
                    })?;
                let context = self
                    .repository
                    .get_context_by_digest(context_digest)
                    .await?
                    .ok_or_else(|| {
                        AuthorizationStateError::Storage("context replay is missing".to_owned())
                    })?;
                require_reusable_context(
                    &context,
                    &snapshot_token,
                    self.trust.verified_manifest.generation(),
                    now_seconds,
                )?;
                context
            }
        };
        let context = self.publish(&context, now_seconds).await?;
        tracing::info!(
            context_digest = %context.context_digest,
            session_id = %context.session_id,
            authority_id = %context.authority_id,
            expires_at = context.expires_at,
            "issued authorization context"
        );
        self.bundle(&context)
    }

    async fn publish(
        &self,
        context: &AuthorizationContextRecord,
        published_at: i64,
    ) -> Result<AuthorizationContextRecord, AuthorizationStateError> {
        self.registry.publish_context(context).await?;
        if context.published_at.is_some() {
            return Ok(context.clone());
        }
        self.repository
            .mark_context_published(&context.context_digest, context.version, published_at)
            .await
    }

    fn bundle(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<AuthorizationContextBundle, AuthorizationStateError> {
        Ok(AuthorizationContextBundle {
            context: serde_json::from_str(&context.signed_context_json).map_err(|error| {
                AuthorizationStateError::InvalidRecord(format!(
                    "persisted authorization context is invalid: {error}"
                ))
            })?,
            trust: self.trust_bundle.clone(),
        })
    }
}

fn require_reusable_context(
    context: &AuthorizationContextRecord,
    snapshot_token: &IssuanceSnapshotToken,
    manifest_generation: u64,
    now_seconds: i64,
) -> Result<(), AuthorizationStateError> {
    if context.state != AuthorizationContextState::Active
        || context.published_at.is_none()
        || context.expires_at <= now_seconds
        || context.issuer_manifest_generation != manifest_generation
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
    expires_at = cmp::min(expires_at, trust.manifest.unsigned.expires_at);
    let remaining = expires_at - now_seconds;
    if remaining
        < i64::try_from(config.minimum_context_lifetime_seconds)
            .map_err(|_| AuthorizationStateError::ContextLifetimeUnavailable)?
    {
        return Err(AuthorizationStateError::ContextLifetimeUnavailable);
    }
    Ok(expires_at)
}

fn issuer_key_id(key: &ed25519_dalek::SigningKey) -> String {
    URL_SAFE_NO_PAD.encode(Sha256::digest(key.verifying_key().to_bytes()))
}

fn seconds_to_millis(seconds: i64) -> Result<i64, AuthorizationStateError> {
    seconds
        .checked_mul(1_000)
        .ok_or_else(|| AuthorizationStateError::InvalidRecord("timestamp overflow".to_owned()))
}

fn context_issue_idempotency(
    request: &AuthorizationContextIssueRequest,
    now_millis: i64,
    expires_at: i64,
    context_digest: &str,
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
        result: json!({ "contextDigest": context_digest }),
        created_at: now_millis,
        expires_at: seconds_to_millis(expires_at)?,
    })
}
