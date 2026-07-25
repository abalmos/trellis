use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use async_nats::jetstream::{self, kv};
use bytes::Bytes;
use futures_util::{StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use trellis_protocol::{
    canonicalize_json, parse_authorization_context_v1, parse_issuer_certificate_v1,
    parse_issuer_manifest_v1, verify_authorization_context_v1, verify_issuer_certificate_v1,
    verify_issuer_manifest_v1, AuthorizationIssuerStatusV1, AuthorizationTrustRootV1,
    AuthorizationVerificationPolicyV1, SignedAuthorizationContextV1,
    SignedAuthorizationIssuerCertificateV1, VerifiedAuthorizationContextV1,
    VerifiedAuthorizationIssuerManifestV1,
};

use super::{
    trust::VerifiedTrustMaterial, AuthorizationContextRecord, AuthorizationTrustStateRecord,
};
use crate::{config::AuthorizationConfig, platform::auth::AuthorizationStateError};

const TRUST_VALUE_BYTES: i32 = 65_536;
const REVOCATION_VALUE_BYTES: usize = 4_096;
const MANIFEST_POINTER_FORMAT: &str = "trellis.authorization-issuer-manifest-pointer.v1";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ManifestPointer {
    format: String,
    generation: u64,
    digest: String,
    locator: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthorizationContextRevocationV1 {
    pub(crate) format: String,
    pub(crate) context_id: String,
    pub(crate) context_digest: String,
    pub(crate) session_id: String,
    pub(crate) issuer_key_id: String,
    pub(crate) issuer_manifest_generation: u64,
    pub(crate) revoked_at: i64,
    pub(crate) expires_at: i64,
    pub(crate) reason: super::AuthorizationContextRevocationReason,
    pub(crate) version: u64,
}

/// Token-level validator cache and registry watcher state for M10/M11 consumers.
#[derive(Clone)]
pub(crate) struct AuthorizationValidatorCache {
    registry: AuthorizationContextRegistry,
    root: AuthorizationTrustRootV1,
    policy: AuthorizationVerificationPolicyV1,
    manifest: Arc<RwLock<(VerifiedAuthorizationIssuerManifestV1, String)>>,
    verified_contexts: Arc<RwLock<HashMap<String, VerifiedAuthorizationContextV1>>>,
    certificates: Arc<RwLock<BTreeMap<(String, String), SignedAuthorizationIssuerCertificateV1>>>,
    revocations: Arc<RwLock<HashSet<String>>>,
    health: Arc<RwLock<AuthorizationValidatorCacheHealth>>,
}

/// Observable health for manifest and revocation registry watches.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorizationValidatorCacheHealth {
    pub(crate) manifest_revision: u64,
    pub(crate) revocation_revision: u64,
    pub(crate) last_update_at: i64,
    pub(crate) healthy: bool,
}

impl AuthorizationValidatorCache {
    pub(crate) fn new(
        registry: AuthorizationContextRegistry,
        trust: &VerifiedTrustMaterial,
    ) -> Result<Self, AuthorizationStateError> {
        Ok(Self {
            registry,
            root: trust.root.clone(),
            policy: trust.policy.clone(),
            manifest: Arc::new(RwLock::new((
                trust.verified_manifest.clone(),
                trust
                    .manifest
                    .digest()
                    .map_err(|error| storage(format!("cannot digest issuer manifest: {error}")))?,
            ))),
            verified_contexts: Arc::new(RwLock::new(HashMap::new())),
            certificates: Arc::new(RwLock::new(trust.certificates.clone())),
            revocations: Arc::new(RwLock::new(HashSet::new())),
            health: Arc::new(RwLock::new(AuthorizationValidatorCacheHealth {
                manifest_revision: 0,
                revocation_revision: 0,
                last_update_at: 0,
                healthy: false,
            })),
        })
    }

    pub(crate) fn get_verified(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<Option<VerifiedAuthorizationContextV1>, AuthorizationStateError> {
        if self.is_revoked(digest)? {
            return Err(storage("authorization context is revoked"));
        }
        let context = self
            .verified_contexts
            .read()
            .map_err(|_| storage("validator context cache lock is poisoned"))?
            .get(digest)
            .cloned();
        Ok(context.filter(|context| context.signed_context().unsigned.expires_at > now))
    }

    pub(crate) fn insert_verified(
        &self,
        digest: String,
        context: VerifiedAuthorizationContextV1,
    ) -> Result<(), AuthorizationStateError> {
        self.verified_contexts
            .write()
            .map_err(|_| storage("validator context cache lock is poisoned"))?
            .insert(digest, context);
        Ok(())
    }

    #[allow(dead_code)] // M9 substrate for later digest-only request/event validation.
    pub(crate) fn certificate(
        &self,
        key_id: &str,
        digest: &str,
    ) -> Result<Option<SignedAuthorizationIssuerCertificateV1>, AuthorizationStateError> {
        self.certificates
            .read()
            .map_err(|_| storage("validator certificate cache lock is poisoned"))
            .map(|certificates| {
                certificates
                    .get(&(key_id.to_owned(), digest.to_owned()))
                    .cloned()
            })
    }

    pub(crate) fn current_manifest(
        &self,
    ) -> Result<VerifiedAuthorizationIssuerManifestV1, AuthorizationStateError> {
        self.manifest
            .read()
            .map_err(|_| storage("validator manifest lock is poisoned"))
            .map(|manifest| manifest.0.clone())
    }

    #[allow(dead_code)] // M9 substrate for later digest-only request/event validation.
    pub(crate) async fn resolve_certificate(
        &self,
        key_id: &str,
        digest: &str,
        now: i64,
    ) -> Result<SignedAuthorizationIssuerCertificateV1, AuthorizationStateError> {
        if let Some(certificate) = self.certificate(key_id, digest)? {
            return Ok(certificate);
        }
        let manifest = self.current_manifest()?;
        if manifest.active_certificate_digest(key_id) != Some(digest) {
            return Err(storage(
                "authorization issuer is not active in the current manifest",
            ));
        }
        let key = format!("certificate.{key_id}.{digest}");
        let value =
            self.registry.get_trust(&key).await?.ok_or_else(|| {
                storage(format!("authorization issuer certificate {key} is missing"))
            })?;
        let json: serde_json::Value = serde_json::from_slice(&value).map_err(|error| {
            storage(format!(
                "authorization issuer certificate is invalid: {error}"
            ))
        })?;
        let certificate = parse_issuer_certificate_v1(&json).map_err(|error| {
            storage(format!(
                "authorization issuer certificate is invalid: {error}"
            ))
        })?;
        if certificate.digest().map_err(|error| {
            storage(format!(
                "cannot digest authorization issuer certificate: {error}"
            ))
        })? != digest
        {
            return Err(storage(
                "authorization issuer certificate digest does not match its key",
            ));
        }
        let mut policy = self.policy.clone();
        policy.now_unix_seconds = now;
        policy.minimum_manifest_generation = manifest.generation();
        verify_issuer_certificate_v1(&self.root, &certificate, &policy).map_err(|error| {
            storage(format!(
                "authorization issuer certificate is not trusted: {error}"
            ))
        })?;
        self.certificates
            .write()
            .map_err(|_| storage("validator certificate cache lock is poisoned"))?
            .insert((key_id.to_owned(), digest.to_owned()), certificate.clone());
        Ok(certificate)
    }

    #[allow(dead_code)] // M9 substrate for later digest-only request/event validation.
    pub(crate) async fn resolve_context(
        &self,
        digest: &str,
        now: i64,
    ) -> Result<VerifiedAuthorizationContextV1, AuthorizationStateError> {
        if let Some(context) = self.get_verified(digest, now)? {
            return Ok(context);
        }
        let value = self
            .registry
            .get_context(digest)
            .await?
            .ok_or_else(|| storage("authorization context is missing from the registry"))?;
        let json: serde_json::Value = serde_json::from_slice(&value)
            .map_err(|error| storage(format!("authorization context is invalid: {error}")))?;
        let context = parse_authorization_context_v1(&json)
            .map_err(|error| storage(format!("authorization context is invalid: {error}")))?;
        if context
            .digest()
            .map_err(|error| storage(format!("cannot digest authorization context: {error}")))?
            != digest
        {
            return Err(storage(
                "authorization context digest does not match its registry key",
            ));
        }
        let manifest = self.current_manifest()?;
        let certificate_digest = manifest
            .active_certificate_digest(&context.unsigned.issuer_key_id)
            .ok_or_else(|| storage("authorization context issuer is not active"))?;
        let certificate = self
            .resolve_certificate(&context.unsigned.issuer_key_id, certificate_digest, now)
            .await?;
        let mut policy = self.policy.clone();
        policy.now_unix_seconds = now;
        policy.minimum_manifest_generation = manifest.generation();
        let verified =
            verify_authorization_context_v1(&self.root, &manifest, &certificate, &context, &policy)
                .map_err(|error| {
                    storage(format!("authorization context is not trusted: {error}"))
                })?;
        self.insert_verified(digest.to_owned(), verified.clone())?;
        Ok(verified)
    }

    pub(crate) fn cache_context(
        &self,
        context: SignedAuthorizationContextV1,
        now: i64,
    ) -> Result<VerifiedAuthorizationContextV1, AuthorizationStateError> {
        let digest = context
            .digest()
            .map_err(|error| storage(format!("cannot digest authorization context: {error}")))?;
        if let Some(context) = self.get_verified(&digest, now)? {
            return Ok(context);
        }
        let manifest = self.current_manifest()?;
        let certificate_digest = manifest
            .active_certificate_digest(&context.unsigned.issuer_key_id)
            .ok_or_else(|| storage("authorization context issuer is not active"))?;
        let certificate = self
            .certificates
            .read()
            .map_err(|_| storage("validator certificate cache lock is poisoned"))?
            .get(&(
                context.unsigned.issuer_key_id.clone(),
                certificate_digest.to_owned(),
            ))
            .cloned()
            .ok_or_else(|| storage("authorization issuer certificate is not cached"))?;
        let mut policy = self.policy.clone();
        policy.now_unix_seconds = now;
        policy.minimum_manifest_generation = manifest.generation();
        let verified =
            verify_authorization_context_v1(&self.root, &manifest, &certificate, &context, &policy)
                .map_err(|error| {
                    storage(format!("authorization context is not trusted: {error}"))
                })?;
        self.insert_verified(digest, verified.clone())?;
        Ok(verified)
    }

    pub(crate) fn is_revoked(&self, digest: &str) -> Result<bool, AuthorizationStateError> {
        Ok(self
            .revocations
            .read()
            .map_err(|_| storage("validator revocation cache lock is poisoned"))?
            .contains(digest))
    }

    pub(crate) fn health(
        &self,
    ) -> Result<AuthorizationValidatorCacheHealth, AuthorizationStateError> {
        self.health
            .read()
            .map_err(|_| storage("validator health lock is poisoned"))
            .map(|health| health.clone())
    }

    pub(crate) async fn run(
        &self,
        stop: crate::shutdown::StopHandle,
    ) -> Result<(), AuthorizationStateError> {
        while !stop.is_stopped() {
            self.health
                .write()
                .map_err(|_| storage("validator health lock is poisoned"))?
                .healthy = false;
            match self.watch_once(&stop).await {
                Ok(()) => return Ok(()),
                Err(error) => {
                    tracing::warn!(%error, "authorization validator registry watch restarting");
                    tokio::select! {
                        () = stop.stopped() => return Ok(()),
                        () = tokio::time::sleep(Duration::from_secs(1)) => {}
                    }
                }
            }
        }
        Ok(())
    }

    async fn watch_once(
        &self,
        stop: &crate::shutdown::StopHandle,
    ) -> Result<(), AuthorizationStateError> {
        // Subscribe before snapshotting so updates racing with the snapshot remain queued.
        let mut manifests = self
            .registry
            .trust
            .watch("manifest.current")
            .await
            .map_err(|error| storage(format!("cannot watch manifest.current: {error}")))?;
        let mut revocations = self
            .registry
            .contexts
            .watch("revocation.>")
            .await
            .map_err(|error| storage(format!("cannot watch context revocations: {error}")))?;
        let manifest = self
            .registry
            .trust
            .entry("manifest.current".to_owned())
            .await
            .map_err(|error| storage(format!("cannot snapshot manifest.current: {error}")))?
            .ok_or_else(|| storage("manifest.current is missing"))?;
        self.observe_manifest(&manifest.value, manifest.revision)
            .await?;
        let keys = self
            .registry
            .contexts
            .keys()
            .await
            .map_err(|error| storage(format!("cannot snapshot context revocations: {error}")))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|error| storage(format!("cannot snapshot context revocations: {error}")))?;
        for key in keys
            .into_iter()
            .filter(|key| key.starts_with("revocation."))
        {
            let entry = self
                .registry
                .contexts
                .entry(key)
                .await
                .map_err(|error| storage(format!("cannot read context revocation: {error}")))?
                .ok_or_else(|| storage("context revocation disappeared during snapshot"))?;
            self.observe_revocation(&entry.key, &entry.value, entry.revision)?;
        }
        {
            let mut health = self
                .health
                .write()
                .map_err(|_| storage("validator health lock is poisoned"))?;
            health.last_update_at = unix_seconds()?;
            health.healthy = true;
        }
        loop {
            tokio::select! {
                () = stop.stopped() => return Ok(()),
                entry = manifests.next() => {
                    let entry = entry.ok_or_else(|| storage("manifest.current watch ended"))?
                        .map_err(|error| storage(format!("manifest.current watch failed: {error}")))?;
                    self.observe_manifest(&entry.value, entry.revision).await?;
                }
                entry = revocations.next() => {
                    let entry = entry.ok_or_else(|| storage("revocation watch ended"))?
                        .map_err(|error| storage(format!("revocation watch failed: {error}")))?;
                    self.observe_revocation(&entry.key, &entry.value, entry.revision)?;
                }
            }
        }
    }

    async fn observe_manifest(
        &self,
        value: &[u8],
        revision: u64,
    ) -> Result<(), AuthorizationStateError> {
        let pointer = parse_manifest_pointer(value)?;
        let was_healthy = self
            .health
            .read()
            .map_err(|_| storage("validator health lock is poisoned"))?
            .healthy;
        let (current_generation, current_digest) = {
            let manifest = self
                .manifest
                .read()
                .map_err(|_| storage("validator manifest lock is poisoned"))?;
            (manifest.0.generation(), manifest.1.clone())
        };
        let mut advanced = false;
        if pointer.generation < current_generation {
            return Err(storage("manifest.current rolled back"));
        }
        if pointer.generation == current_generation && pointer.digest != current_digest {
            return Err(storage(
                "manifest.current equivocates at the accepted generation",
            ));
        }
        if pointer.generation > current_generation {
            self.health
                .write()
                .map_err(|_| storage("validator health lock is poisoned"))?
                .healthy = false;
            let key = format!("manifest.{}", pointer.generation);
            let value = self
                .registry
                .get_trust(&key)
                .await?
                .ok_or_else(|| storage(format!("issuer manifest {key} is missing")))?;
            let json: serde_json::Value = serde_json::from_slice(&value)
                .map_err(|error| storage(format!("issuer manifest {key} is invalid: {error}")))?;
            let manifest = parse_issuer_manifest_v1(&json)
                .map_err(|error| storage(format!("issuer manifest {key} is invalid: {error}")))?;
            let digest = manifest.digest().map_err(|error| {
                storage(format!("cannot digest issuer manifest {key}: {error}"))
            })?;
            if digest != pointer.digest || pointer.locator != manifest_locator(pointer.generation) {
                return Err(storage(
                    "manifest.current does not match its immutable manifest",
                ));
            }
            let mut policy = self.policy.clone();
            policy.now_unix_seconds = unix_seconds()?;
            policy.minimum_manifest_generation = current_generation;
            let verified =
                verify_issuer_manifest_v1(&self.root, &manifest, &policy).map_err(|error| {
                    storage(format!("issuer manifest {key} is not trusted: {error}"))
                })?;
            let mut certificates = Vec::new();
            for issuer in manifest
                .unsigned
                .issuers
                .iter()
                .filter(|issuer| issuer.status == AuthorizationIssuerStatusV1::Active)
            {
                let certificate_key = format!(
                    "certificate.{}.{}",
                    issuer.key_id, issuer.certificate_digest
                );
                let certificate_value = self
                    .registry
                    .get_trust(&certificate_key)
                    .await?
                    .ok_or_else(|| {
                        storage(format!(
                            "authorization issuer certificate {certificate_key} is missing"
                        ))
                    })?;
                let certificate_json: serde_json::Value =
                    serde_json::from_slice(&certificate_value).map_err(|error| {
                        storage(format!(
                            "authorization issuer certificate is invalid: {error}"
                        ))
                    })?;
                let certificate =
                    parse_issuer_certificate_v1(&certificate_json).map_err(|error| {
                        storage(format!(
                            "authorization issuer certificate is invalid: {error}"
                        ))
                    })?;
                if certificate.digest().map_err(|error| {
                    storage(format!(
                        "cannot digest authorization issuer certificate: {error}"
                    ))
                })? != issuer.certificate_digest
                {
                    return Err(storage(
                        "authorization issuer certificate digest does not match its key",
                    ));
                }
                verify_issuer_certificate_v1(&self.root, &certificate, &policy).map_err(
                    |error| {
                        storage(format!(
                            "authorization issuer certificate is not trusted: {error}"
                        ))
                    },
                )?;
                certificates.push((
                    (issuer.key_id.clone(), issuer.certificate_digest.clone()),
                    certificate,
                ));
            }
            self.certificates
                .write()
                .map_err(|_| storage("validator certificate cache lock is poisoned"))?
                .extend(certificates);
            *self
                .manifest
                .write()
                .map_err(|_| storage("validator manifest lock is poisoned"))? = (verified, digest);
            self.verified_contexts
                .write()
                .map_err(|_| storage("validator context cache lock is poisoned"))?
                .clear();
            advanced = true;
        }
        let mut health = self
            .health
            .write()
            .map_err(|_| storage("validator health lock is poisoned"))?;
        health.manifest_revision = health.manifest_revision.max(revision);
        health.last_update_at = unix_seconds()?;
        health.healthy = was_healthy && !advanced;
        drop(health);
        if advanced {
            return Err(storage(
                "issuer manifest advanced; restarting complete validator snapshot",
            ));
        }
        Ok(())
    }

    fn observe_revocation(
        &self,
        key: &str,
        value: &[u8],
        revision: u64,
    ) -> Result<(), AuthorizationStateError> {
        let record: AuthorizationContextRevocationV1 = serde_json::from_slice(value)
            .map_err(|error| storage(format!("context revocation is invalid: {error}")))?;
        record.validate()?;
        if key != format!("revocation.{}", record.context_digest) {
            return Err(storage("context revocation key does not match its digest"));
        }
        self.revocations
            .write()
            .map_err(|_| storage("validator revocation cache lock is poisoned"))?
            .insert(record.context_digest);
        let mut health = self
            .health
            .write()
            .map_err(|_| storage("validator health lock is poisoned"))?;
        health.revocation_revision = health.revocation_revision.max(revision);
        health.last_update_at = unix_seconds()?;
        Ok(())
    }
}

impl AuthorizationContextRevocationV1 {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        if self.format != "trellis.authorization-context-revocation.v1"
            || self.context_id.is_empty()
            || self.context_digest.is_empty()
            || self.session_id.is_empty()
            || self.issuer_key_id.is_empty()
            || self.issuer_manifest_generation == 0
            || self.revoked_at <= 0
            || self.expires_at <= 0
            || self.revoked_at > self.expires_at
            || self.version != 1
        {
            return Err(storage("authorization context revocation is invalid"));
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorizationRegistryTrustFloor {
    pub(crate) generation: u64,
    pub(crate) digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorizationTrustBundle {
    pub(crate) root: serde_json::Value,
    pub(crate) issuer_manifest_generation: u64,
    pub(crate) issuer_manifest_digest: String,
    pub(crate) issuer_manifest_locator: String,
    pub(crate) issuer_certificate_locator: String,
    pub(crate) context_registry_locator: String,
    pub(crate) revocation_snapshot_locator: String,
    pub(crate) manifest_watch_subject: String,
    pub(crate) revocation_watch_subject: String,
    pub(crate) policy: AuthorizationTrustPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorizationTrustPolicy {
    pub(crate) allowed_clock_skew_seconds: u32,
    pub(crate) maximum_context_lifetime_seconds: u32,
    /// Maximum canonical signed-context JSON size in UTF-8 bytes.
    pub(crate) maximum_context_bytes: usize,
    pub(crate) maximum_permissions: usize,
    pub(crate) maximum_capabilities: usize,
    pub(crate) refresh_lead_seconds: u32,
    pub(crate) refresh_jitter_seconds: u32,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorizationContextBundle {
    pub(crate) context: String,
    pub(crate) context_digest: String,
    pub(crate) refresh_at: i64,
    pub(crate) trust: AuthorizationTrustBundle,
}

#[derive(Clone)]
pub(crate) struct AuthorizationContextRegistry {
    trust: kv::Store,
    contexts: kv::Store,
}

impl AuthorizationContextRegistry {
    pub(crate) async fn trust_floor(
        &self,
        trust: &VerifiedTrustMaterial,
    ) -> Result<Option<AuthorizationRegistryTrustFloor>, AuthorizationStateError> {
        inspect_trust_floor(&self.trust, trust).await
    }

    pub(crate) async fn check(
        client: async_nats::Client,
        config: &AuthorizationConfig,
        trust: &VerifiedTrustMaterial,
        sqlite_floor: Option<&AuthorizationTrustStateRecord>,
    ) -> Result<(), AuthorizationStateError> {
        let jetstream = jetstream::new(client);
        let trust_store = open_existing(&jetstream, &config.trust_bucket).await?;
        if let Some(trust_store) = &trust_store {
            check_policy(
                trust_store,
                &config.trust_bucket,
                Duration::ZERO,
                TRUST_VALUE_BYTES,
                config.registry_replicas,
            )
            .await?;
        }
        let context_age = config
            .context_lifetime_seconds
            .checked_add(config.cleanup_grace_seconds)
            .ok_or_else(|| storage("context registry retention overflow"))?;
        let context_value_bytes = context_registry_value_bytes(config.maximum_context_bytes)?;
        if let Some(context_store) = open_existing(&jetstream, &config.context_bucket).await? {
            check_policy(
                &context_store,
                &config.context_bucket,
                Duration::from_secs(context_age),
                context_value_bytes,
                config.registry_replicas,
            )
            .await?;
        }

        let trust_store =
            trust_store.ok_or_else(|| storage("authorization trust registry is missing"))?;
        if open_existing(&jetstream, &config.context_bucket)
            .await?
            .is_none()
        {
            return Err(storage("authorization context registry is missing"));
        }

        let registry_floor = inspect_trust_floor(&trust_store, trust).await?;
        reconcile_trust_floors(trust, sqlite_floor, registry_floor.as_ref())?;

        let root_json = serde_json::to_value(&trust.root)
            .map_err(|error| storage(format!("cannot encode trust root: {error}")))?;
        let root = canonicalize_json(&root_json)
            .map_err(|error| storage(format!("cannot encode trust root: {error}")))?;
        confirm_exact(&trust_store, "root", root.as_bytes()).await?;
        let generation = trust.verified_manifest.generation();
        let manifest_value = serde_json::to_value(&trust.manifest)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let manifest = canonicalize_json(&manifest_value)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        confirm_exact(
            &trust_store,
            &format!("manifest.{generation}"),
            manifest.as_bytes(),
        )
        .await?;
        let manifest_digest = trust
            .manifest
            .digest()
            .map_err(|error| storage(format!("cannot digest issuer manifest: {error}")))?;
        let locator = format!("/.well-known/trellis/authorization/trust/manifest.{generation}");
        let pointer = manifest_pointer(generation, manifest_digest.clone(), locator)?;
        confirm_exact(&trust_store, "manifest.current", pointer.as_bytes()).await?;
        let certificate_digest = trust.active_certificate.digest().map_err(|error| {
            storage(format!("cannot digest active issuer certificate: {error}"))
        })?;
        let certificate_value = serde_json::to_value(&trust.active_certificate)
            .map_err(|error| storage(format!("cannot encode issuer certificate: {error}")))?;
        let certificate = canonicalize_json(&certificate_value)
            .map_err(|error| storage(format!("cannot encode issuer certificate: {error}")))?;
        confirm_exact(
            &trust_store,
            &format!(
                "certificate.{}.{}",
                trust.active_certificate.unsigned.key_id, certificate_digest
            ),
            certificate.as_bytes(),
        )
        .await
    }

    pub(crate) async fn ensure(
        client: async_nats::Client,
        config: &AuthorizationConfig,
    ) -> Result<Self, AuthorizationStateError> {
        let jetstream = jetstream::new(client);
        let trust = open_or_create(
            &jetstream,
            &config.trust_bucket,
            Duration::ZERO,
            TRUST_VALUE_BYTES,
            config.registry_replicas,
        )
        .await?;
        let context_age = config
            .context_lifetime_seconds
            .checked_add(config.cleanup_grace_seconds)
            .ok_or_else(|| storage("context registry retention overflow"))?;
        let context_value_bytes = context_registry_value_bytes(config.maximum_context_bytes)?;
        let contexts = open_or_create(
            &jetstream,
            &config.context_bucket,
            Duration::from_secs(context_age),
            context_value_bytes,
            config.registry_replicas,
        )
        .await?;
        Ok(Self { trust, contexts })
    }

    pub(crate) async fn publish_trust_immutables(
        &self,
        trust: &VerifiedTrustMaterial,
        config: &AuthorizationConfig,
    ) -> Result<AuthorizationTrustBundle, AuthorizationStateError> {
        let root_json = serde_json::to_value(&trust.root)
            .map_err(|error| storage(format!("cannot encode trust root: {error}")))?;
        let root = canonicalize_json(&root_json)
            .map_err(|error| storage(format!("cannot encode trust root: {error}")))?;
        publish_immutable(&self.trust, "root", root.as_bytes()).await?;
        let mut issuer_certificate_locator = None;
        let active_certificate_digest = trust.active_certificate.digest().map_err(|error| {
            storage(format!("cannot digest active issuer certificate: {error}"))
        })?;
        for ((key_id, certificate_digest), certificate) in &trust.certificates {
            let value = serde_json::to_value(certificate)
                .map_err(|error| storage(format!("cannot encode issuer certificate: {error}")))?;
            let canonical = canonicalize_json(&value)
                .map_err(|error| storage(format!("cannot encode issuer certificate: {error}")))?;
            let key = format!("certificate.{key_id}.{certificate_digest}");
            publish_immutable(&self.trust, &key, canonical.as_bytes()).await?;
            if key_id == &trust.active_certificate.unsigned.key_id
                && certificate_digest == &active_certificate_digest
            {
                issuer_certificate_locator =
                    Some(format!("/.well-known/trellis/authorization/trust/{key}"));
            }
        }
        let manifest_value = serde_json::to_value(&trust.manifest)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let manifest = canonicalize_json(&manifest_value)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let generation = trust.verified_manifest.generation();
        let manifest_digest = trust
            .manifest
            .digest()
            .map_err(|error| storage(format!("cannot digest issuer manifest: {error}")))?;
        let manifest_key = format!("manifest.{generation}");
        publish_immutable(&self.trust, &manifest_key, manifest.as_bytes()).await?;
        let manifest_locator = format!("/.well-known/trellis/authorization/trust/{manifest_key}");
        Ok(AuthorizationTrustBundle {
            root: root_json,
            issuer_manifest_generation: generation,
            issuer_manifest_digest: manifest_digest,
            issuer_manifest_locator: manifest_locator,
            issuer_certificate_locator: issuer_certificate_locator
                .ok_or_else(|| storage("active issuer certificate was not published"))?,
            context_registry_locator: "/.well-known/trellis/authorization/contexts/".to_owned(),
            revocation_snapshot_locator: "/.well-known/trellis/authorization/revocations"
                .to_owned(),
            manifest_watch_subject: format!("$KV.{}.manifest.current", config.trust_bucket),
            revocation_watch_subject: format!("$KV.{}.revocation.>", config.context_bucket),
            policy: AuthorizationTrustPolicy {
                allowed_clock_skew_seconds: trust.policy.allowed_clock_skew_seconds,
                maximum_context_lifetime_seconds: trust.policy.maximum_context_lifetime_seconds,
                maximum_context_bytes: trust.policy.maximum_context_bytes,
                maximum_permissions: trust.policy.maximum_permissions,
                maximum_capabilities: trust.policy.maximum_capabilities,
                refresh_lead_seconds: u32::try_from(config.refresh_lead_seconds)
                    .map_err(|_| storage("refresh lead exceeds protocol bounds"))?,
                refresh_jitter_seconds: u32::try_from(config.refresh_jitter_seconds)
                    .map_err(|_| storage("refresh jitter exceeds protocol bounds"))?,
            },
        })
    }

    pub(crate) async fn advance_trust_pointer(
        &self,
        trust: &AuthorizationTrustBundle,
    ) -> Result<(), AuthorizationStateError> {
        let pointer = manifest_pointer(
            trust.issuer_manifest_generation,
            trust.issuer_manifest_digest.clone(),
            trust.issuer_manifest_locator.clone(),
        )?;
        advance_manifest_pointer(&self.trust, &pointer).await
    }

    pub(crate) async fn publish_context(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<(), AuthorizationStateError> {
        publish_immutable(
            &self.contexts,
            &context.context_digest,
            context.signed_context_json.as_bytes(),
        )
        .await
    }

    pub(crate) async fn get_trust(
        &self,
        key: &str,
    ) -> Result<Option<Bytes>, AuthorizationStateError> {
        self.trust
            .get(key)
            .await
            .map_err(|error| storage(format!("cannot read trust registry key {key}: {error}")))
    }

    pub(crate) async fn get_context(
        &self,
        digest: &str,
    ) -> Result<Option<Bytes>, AuthorizationStateError> {
        self.contexts.get(digest).await.map_err(|error| {
            storage(format!(
                "cannot read context registry key {digest}: {error}"
            ))
        })
    }

    pub(crate) async fn list_revocations(
        &self,
    ) -> Result<Vec<AuthorizationContextRevocationV1>, AuthorizationStateError> {
        let keys = self
            .contexts
            .keys()
            .await
            .map_err(|error| storage(format!("cannot list context revocations: {error}")))?
            .try_collect::<Vec<_>>()
            .await
            .map_err(|error| storage(format!("cannot list context revocations: {error}")))?;
        let mut records = Vec::new();
        for key in keys
            .into_iter()
            .filter(|key| key.starts_with("revocation."))
        {
            let value = self
                .contexts
                .get(&key)
                .await
                .map_err(|error| storage(format!("cannot read context revocation {key}: {error}")))?
                .ok_or_else(|| storage(format!("context revocation {key} disappeared")))?;
            let record: AuthorizationContextRevocationV1 =
                serde_json::from_slice(&value).map_err(|error| {
                    storage(format!("context revocation {key} is invalid: {error}"))
                })?;
            record.validate()?;
            if key != format!("revocation.{}", record.context_digest) {
                return Err(storage("context revocation key does not match its digest"));
            }
            records.push(record);
        }
        records.sort_by(|left, right| left.context_digest.cmp(&right.context_digest));
        Ok(records)
    }

    pub(crate) async fn publish_revocation(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<(), AuthorizationStateError> {
        let signed = parse_authorization_context_v1(
            &serde_json::from_str(&context.signed_context_json)
                .map_err(|error| storage(format!("cannot parse revoked context: {error}")))?,
        )
        .map_err(|error| storage(format!("cannot parse revoked context: {error}")))?;
        if signed.unsigned.context_id != context.context_id
            || signed.unsigned.session_id != context.session_id
            || signed.unsigned.issuer_key_id != context.issuer_key_id
        {
            return Err(storage(
                "revoked context record does not match its signed context",
            ));
        }
        let record = AuthorizationContextRevocationV1 {
            format: "trellis.authorization-context-revocation.v1".to_owned(),
            context_id: context.context_id.clone(),
            context_digest: context.context_digest.clone(),
            session_id: context.session_id.clone(),
            issuer_key_id: context.issuer_key_id.clone(),
            issuer_manifest_generation: context.trust_generation,
            revoked_at: context
                .revoked_at
                .ok_or_else(|| storage("revoked context has no revocation time"))?,
            expires_at: context.expires_at,
            reason: context
                .revocation_reason
                .ok_or_else(|| storage("revoked context has no revocation reason"))?,
            version: 1,
        };
        record.validate()?;
        let payload = canonicalize_json(
            &serde_json::to_value(record)
                .map_err(|error| storage(format!("cannot encode context revocation: {error}")))?,
        )
        .map_err(|error| storage(format!("cannot encode context revocation: {error}")))?;
        publish_immutable(
            &self.contexts,
            &format!("revocation.{}", context.context_digest),
            payload.as_bytes(),
        )
        .await
    }
}

pub(crate) fn reconcile_trust_floors(
    trust: &VerifiedTrustMaterial,
    sqlite_floor: Option<&AuthorizationTrustStateRecord>,
    registry_floor: Option<&AuthorizationRegistryTrustFloor>,
) -> Result<(), AuthorizationStateError> {
    let configured_generation = trust.verified_manifest.generation();
    let configured_digest = trust
        .manifest
        .digest()
        .map_err(|error| storage(format!("cannot digest configured issuer manifest: {error}")))?;
    for (name, generation, digest) in sqlite_floor
        .map(|floor| {
            (
                "SQLite",
                floor.manifest_generation,
                floor.manifest_digest.as_str(),
            )
        })
        .into_iter()
        .chain(registry_floor.map(|floor| ("NATS", floor.generation, floor.digest.as_str())))
    {
        if generation > configured_generation {
            return Err(storage(format!(
                "configured issuer manifest generation {configured_generation} rolls back the {name} floor {generation}"
            )));
        }
        if generation == configured_generation && digest != configured_digest {
            return Err(storage(format!(
                "configured issuer manifest equivocates with the {name} floor at generation {generation}"
            )));
        }
    }
    Ok(())
}

async fn inspect_trust_floor(
    store: &kv::Store,
    trust: &VerifiedTrustMaterial,
) -> Result<Option<AuthorizationRegistryTrustFloor>, AuthorizationStateError> {
    let keys = store
        .keys()
        .await
        .map_err(|error| storage(format!("cannot list authorization trust registry: {error}")))?
        .try_collect::<Vec<_>>()
        .await
        .map_err(|error| storage(format!("cannot list authorization trust registry: {error}")))?;
    let mut manifests = BTreeMap::new();
    for key in keys {
        let Some(generation) = key
            .strip_prefix("manifest.")
            .and_then(|value| value.parse::<u64>().ok())
        else {
            continue;
        };
        let value = store
            .get(&key)
            .await
            .map_err(|error| storage(format!("cannot read issuer manifest {key}: {error}")))?
            .ok_or_else(|| storage(format!("issuer manifest {key} disappeared")))?;
        let json: serde_json::Value = serde_json::from_slice(&value)
            .map_err(|error| storage(format!("issuer manifest {key} is invalid JSON: {error}")))?;
        let canonical = canonicalize_json(&json)
            .map_err(|error| storage(format!("issuer manifest {key} is invalid: {error}")))?;
        if canonical.as_bytes() != value.as_ref() {
            return Err(storage(format!(
                "issuer manifest {key} is not canonical JSON"
            )));
        }
        let manifest = parse_issuer_manifest_v1(&json)
            .map_err(|error| storage(format!("issuer manifest {key} is invalid: {error}")))?;
        let historical_policy = AuthorizationVerificationPolicyV1::new(
            manifest.unsigned.not_before,
            trust.policy.allowed_clock_skew_seconds,
            trust.policy.maximum_context_lifetime_seconds,
            trust.policy.maximum_context_bytes,
            trust.policy.maximum_permissions,
            trust.policy.maximum_capabilities,
            1,
        )
        .map_err(|error| storage(format!("cannot validate issuer manifest {key}: {error}")))?;
        let verified = verify_issuer_manifest_v1(&trust.root, &manifest, &historical_policy)
            .map_err(|error| storage(format!("issuer manifest {key} is not trusted: {error}")))?;
        if verified.generation() != generation {
            return Err(storage(format!(
                "issuer manifest {key} generation does not match its registry key"
            )));
        }
        let digest = manifest
            .digest()
            .map_err(|error| storage(format!("cannot digest issuer manifest {key}: {error}")))?;
        manifests.insert(generation, digest);
    }
    let highest =
        manifests
            .last_key_value()
            .map(|(generation, digest)| AuthorizationRegistryTrustFloor {
                generation: *generation,
                digest: digest.clone(),
            });
    let pointer = store
        .entry("manifest.current".to_owned())
        .await
        .map_err(|error| {
            storage(format!(
                "cannot read current issuer manifest pointer: {error}"
            ))
        })?;
    if let Some(entry) = pointer {
        let pointer = parse_manifest_pointer(&entry.value)?;
        let history_digest = manifests.get(&pointer.generation).ok_or_else(|| {
            storage("current issuer manifest pointer has no immutable manifest history")
        })?;
        if history_digest != &pointer.digest
            || pointer.locator != manifest_locator(pointer.generation)
        {
            return Err(storage(
                "current issuer manifest pointer does not match immutable history",
            ));
        }
    }
    Ok(highest)
}

async fn advance_manifest_pointer(
    store: &kv::Store,
    canonical_pointer: &str,
) -> Result<(), AuthorizationStateError> {
    let configured = parse_manifest_pointer(canonical_pointer.as_bytes())?;
    let existing = store
        .entry("manifest.current".to_owned())
        .await
        .map_err(|error| {
            storage(format!(
                "cannot read current issuer manifest pointer: {error}"
            ))
        })?;
    match existing {
        None => {
            store
                .create(
                    "manifest.current",
                    Bytes::copy_from_slice(canonical_pointer.as_bytes()),
                )
                .await
                .map_err(|error| {
                    storage(format!(
                        "cannot create current issuer manifest pointer: {error}"
                    ))
                })?;
        }
        Some(entry) => {
            let current = parse_manifest_pointer(&entry.value)?;
            if current.generation > configured.generation {
                return Err(storage("current issuer manifest pointer would roll back"));
            }
            if current.generation == configured.generation {
                if current != configured {
                    return Err(storage("current issuer manifest pointer equivocates"));
                }
                confirm_exact(store, "manifest.current", canonical_pointer.as_bytes()).await?;
                return Ok(());
            }
            store
                .update(
                    "manifest.current",
                    Bytes::copy_from_slice(canonical_pointer.as_bytes()),
                    entry.revision,
                )
                .await
                .map_err(|error| {
                    storage(format!(
                        "cannot CAS-advance current issuer manifest pointer: {error}"
                    ))
                })?;
        }
    }
    confirm_exact(store, "manifest.current", canonical_pointer.as_bytes()).await
}

fn parse_manifest_pointer(value: &[u8]) -> Result<ManifestPointer, AuthorizationStateError> {
    let pointer: ManifestPointer = serde_json::from_slice(value).map_err(|error| {
        storage(format!(
            "current issuer manifest pointer is invalid: {error}"
        ))
    })?;
    if pointer.format != MANIFEST_POINTER_FORMAT
        || pointer.generation == 0
        || pointer.digest.is_empty()
        || pointer.locator != manifest_locator(pointer.generation)
    {
        return Err(storage("current issuer manifest pointer is invalid"));
    }
    let canonical = canonicalize_json(
        &serde_json::to_value(&pointer)
            .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))?,
    )
    .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))?;
    if canonical.as_bytes() != value {
        return Err(storage(
            "current issuer manifest pointer is not canonical JSON",
        ));
    }
    Ok(pointer)
}

fn manifest_pointer(
    generation: u64,
    digest: String,
    locator: String,
) -> Result<String, AuthorizationStateError> {
    canonicalize_json(
        &serde_json::to_value(ManifestPointer {
            format: MANIFEST_POINTER_FORMAT.to_owned(),
            generation,
            digest,
            locator,
        })
        .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))?,
    )
    .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))
}

fn manifest_locator(generation: u64) -> String {
    format!("/.well-known/trellis/authorization/trust/manifest.{generation}")
}

fn context_registry_value_bytes(
    maximum_context_bytes: usize,
) -> Result<i32, AuthorizationStateError> {
    i32::try_from(maximum_context_bytes.max(REVOCATION_VALUE_BYTES))
        .map_err(|_| storage("maximum context bytes exceed NATS KV bounds"))
}

async fn open_existing(
    jetstream: &jetstream::Context,
    bucket: &str,
) -> Result<Option<jetstream::kv::Store>, AuthorizationStateError> {
    use async_nats::jetstream::{context::GetStreamErrorKind, ErrorCode};

    match jetstream.get_stream(format!("KV_{bucket}")).await {
        Ok(_) => jetstream
            .get_key_value(bucket)
            .await
            .map(Some)
            .map_err(|error| storage(format!("cannot open {bucket}: {error}"))),
        Err(error)
            if matches!(
                error.kind(),
                GetStreamErrorKind::JetStream(error)
                    if error.error_code() == ErrorCode::STREAM_NOT_FOUND
            ) =>
        {
            Ok(None)
        }
        Err(error) => Err(storage(format!("cannot inspect {bucket}: {error}"))),
    }
}

async fn open_or_create(
    jetstream: &jetstream::Context,
    bucket: &str,
    max_age: Duration,
    max_value_size: i32,
    replicas: usize,
) -> Result<kv::Store, AuthorizationStateError> {
    let config = kv::Config {
        bucket: bucket.to_owned(),
        history: 1,
        max_age,
        max_value_size,
        num_replicas: replicas,
        ..Default::default()
    };
    let store = match jetstream.get_key_value(bucket).await {
        Ok(store) => store,
        Err(open_error) => match jetstream.create_key_value(config).await {
            Ok(store) => store,
            Err(create_error) => jetstream.get_key_value(bucket).await.map_err(|error| {
                storage(format!(
                    "cannot open {bucket} ({open_error}), create it ({create_error}), or reopen it ({error})"
                ))
            })?,
        },
    };
    check_policy(&store, bucket, max_age, max_value_size, replicas).await?;
    Ok(store)
}

async fn check_policy(
    store: &kv::Store,
    bucket: &str,
    max_age: Duration,
    max_value_size: i32,
    replicas: usize,
) -> Result<(), AuthorizationStateError> {
    let status = store
        .status()
        .await
        .map_err(|error| storage(format!("cannot inspect {bucket}: {error}")))?;
    if status.max_age() != max_age
        || status.info.config.max_message_size != max_value_size
        || status.history() != 1
        || status.info.config.num_replicas != replicas
    {
        return Err(storage(format!(
            "{bucket} policy does not match configuration"
        )));
    }
    Ok(())
}

async fn publish_immutable(
    store: &kv::Store,
    key: &str,
    bytes: &[u8],
) -> Result<(), AuthorizationStateError> {
    match store.create(key, Bytes::copy_from_slice(bytes)).await {
        Ok(_) => confirm_exact(store, key, bytes).await,
        Err(create_error) => {
            match store.get(key).await.map_err(|error| {
                storage(format!("cannot read {key} after create failure: {error}"))
            })? {
                Some(existing) if existing.as_ref() == bytes => {
                    confirm_exact(store, key, bytes).await
                }
                Some(_) => Err(storage(format!("immutable registry key {key} changed"))),
                None => Err(storage(format!(
                    "cannot create registry key {key}: {create_error}"
                ))),
            }
        }
    }
}

async fn confirm_exact(
    store: &kv::Store,
    key: &str,
    expected: &[u8],
) -> Result<(), AuthorizationStateError> {
    for attempt in 0..3 {
        match store
            .get(key)
            .await
            .map_err(|error| storage(format!("cannot confirm registry key {key}: {error}")))?
        {
            Some(actual) if actual.as_ref() == expected => return Ok(()),
            Some(_) => return Err(storage(format!("registry key {key} readback changed"))),
            None if attempt < 2 => {
                tokio::time::sleep(Duration::from_millis(50 * (attempt + 1))).await;
            }
            None => break,
        }
    }
    Err(storage(format!(
        "registry key {key} was not visible after publication"
    )))
}

fn storage(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::Storage(message.into())
}

fn unix_seconds() -> Result<i64, AuthorizationStateError> {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|error| storage(format!("system time is invalid: {error}")))?
            .as_secs(),
    )
    .map_err(|_| storage("system time exceeds i64 seconds"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_registry_limit_uses_canonical_json_bytes() {
        assert_eq!(context_registry_value_bytes(16_384).unwrap(), 16_384);
    }
}
