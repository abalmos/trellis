use std::time::Duration;

use async_nats::jetstream::{self, kv};
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use trellis_protocol::{canonicalize_json, parse_authorization_context_v1};

use super::{
    trust::VerifiedTrustMaterial, AuthorizationContextRecord, AuthorizationTrustStateRecord,
};
use crate::{config::AuthorizationConfig, platform::auth::AuthorizationStateError};

const TRUST_VALUE_BYTES: i32 = 65_536;
const REVOCATION_VALUE_BYTES: usize = 4_096;
pub(super) const MANIFEST_CURRENT_KEY: &str = "manifest.current";
pub(super) const MANIFEST_PREFIX: &str = "manifest.";
pub(super) const REVOCATION_PREFIX: &str = "revocation.";

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ManifestPointer {
    generation: u64,
    digest: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthorizationContextRevocationV1 {
    pub(crate) revoked_at: i64,
}

impl AuthorizationContextRevocationV1 {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        if self.revoked_at <= 0 {
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

/// NATS-backed authorization registry buckets distributed with the pinned trust root.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthorizationRegistryBinding {
    /// KV bucket holding the current manifest pointer and immutable manifest history.
    pub(crate) trust_bucket: String,
    /// KV bucket holding canonical signed contexts and revocations.
    pub(crate) context_bucket: String,
}

impl AuthorizationRegistryBinding {
    /// Test binding over the canonical key layout.
    #[cfg(test)]
    pub(crate) fn test_binding() -> Self {
        Self {
            trust_bucket: "trellis_authorization_trust".to_owned(),
            context_bucket: "trellis_authorization_contexts".to_owned(),
        }
    }

    pub(crate) fn from_config(config: &AuthorizationConfig) -> Self {
        Self {
            trust_bucket: config.trust_bucket.clone(),
            context_bucket: config.context_bucket.clone(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthorizationTrustBundle {
    pub(crate) root: serde_json::Value,
    /// Complete canonical issuer manifest embedded in the trust bundle.
    pub(crate) manifest: serde_json::Value,
    pub(crate) authorization_registry: AuthorizationRegistryBinding,
    pub(crate) policy: AuthorizationTrustPolicy,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
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
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthorizationContextBundle {
    pub(crate) context: serde_json::Value,
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
    ) -> Result<Option<AuthorizationRegistryTrustFloor>, AuthorizationStateError> {
        inspect_trust_floor(&self.trust).await
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
        let context_age = context_registry_retention_seconds(config)?;
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

        let registry_floor = inspect_trust_floor(&trust_store).await?;
        reconcile_trust_floors(trust, sqlite_floor, registry_floor.as_ref())?;

        let generation = trust.verified_manifest.generation();
        let manifest_value = serde_json::to_value(&trust.manifest)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let manifest = canonicalize_json(&manifest_value)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        confirm_exact(
            &trust_store,
            &format!("{MANIFEST_PREFIX}{generation}"),
            manifest.as_bytes(),
        )
        .await?;
        let manifest_digest = trust
            .manifest
            .digest()
            .map_err(|error| storage(format!("cannot digest issuer manifest: {error}")))?;
        let pointer = manifest_pointer(generation, manifest_digest.clone())?;
        confirm_exact(&trust_store, MANIFEST_CURRENT_KEY, pointer.as_bytes()).await?;
        Ok(())
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
        let context_age = context_registry_retention_seconds(config)?;
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
        let manifest_value = serde_json::to_value(&trust.manifest)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let manifest = canonicalize_json(&manifest_value)
            .map_err(|error| storage(format!("cannot encode issuer manifest: {error}")))?;
        let generation = trust.verified_manifest.generation();
        let manifest_key = format!("{MANIFEST_PREFIX}{generation}");
        publish_immutable(&self.trust, &manifest_key, manifest.as_bytes()).await?;
        Ok(AuthorizationTrustBundle {
            root: root_json,
            manifest: serde_json::from_str(&manifest)
                .map_err(|error| storage(format!("cannot decode issuer manifest: {error}")))?,
            authorization_registry: AuthorizationRegistryBinding::from_config(config),
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
        let manifest = trellis_protocol::parse_issuer_manifest_v1(&trust.manifest)
            .map_err(|error| storage(format!("cannot parse issuer manifest: {error}")))?;
        let pointer = manifest_pointer(
            manifest.unsigned.generation,
            manifest
                .digest()
                .map_err(|error| storage(format!("cannot digest issuer manifest: {error}")))?,
        )?;
        advance_manifest_pointer(&self.trust, &pointer).await
    }

    pub(crate) async fn publish_context(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<(), AuthorizationStateError> {
        let value: serde_json::Value = serde_json::from_str(&context.signed_context_json)
            .map_err(|error| storage(format!("cannot parse authorization context: {error}")))?;
        let signed = parse_authorization_context_v1(&value)
            .map_err(|error| storage(format!("cannot parse authorization context: {error}")))?;
        let canonical = canonicalize_json(&value).map_err(|error| {
            storage(format!(
                "cannot canonicalize authorization context: {error}"
            ))
        })?;
        let digest = signed
            .digest()
            .map_err(|error| storage(format!("cannot digest authorization context: {error}")))?;
        if canonical != context.signed_context_json || digest != context.context_digest {
            return Err(storage(
                "authorization context key or canonical signed JSON does not match",
            ));
        }
        publish_immutable(&self.contexts, &digest, canonical.as_bytes()).await
    }

    pub(crate) async fn publish_revocation(
        &self,
        context: &AuthorizationContextRecord,
    ) -> Result<(), AuthorizationStateError> {
        let record = AuthorizationContextRevocationV1 {
            revoked_at: context
                .revoked_at
                .ok_or_else(|| storage("revoked context has no revocation time"))?,
        };
        record.validate()?;
        let payload = canonicalize_json(
            &serde_json::to_value(record)
                .map_err(|error| storage(format!("cannot encode context revocation: {error}")))?,
        )
        .map_err(|error| storage(format!("cannot encode context revocation: {error}")))?;
        publish_immutable(
            &self.contexts,
            &format!("{REVOCATION_PREFIX}{}", context.context_digest),
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
    reconcile_trust_floor_values(
        configured_generation,
        &configured_digest,
        sqlite_floor,
        registry_floor,
    )
}

fn reconcile_trust_floor_values(
    configured_generation: u64,
    configured_digest: &str,
    sqlite_floor: Option<&AuthorizationTrustStateRecord>,
    registry_floor: Option<&AuthorizationRegistryTrustFloor>,
) -> Result<(), AuthorizationStateError> {
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
) -> Result<Option<AuthorizationRegistryTrustFloor>, AuthorizationStateError> {
    let pointer = store
        .entry(MANIFEST_CURRENT_KEY.to_owned())
        .await
        .map_err(|error| {
            storage(format!(
                "cannot read current issuer manifest pointer: {error}"
            ))
        })?;
    pointer
        .map(|entry| {
            let pointer = parse_api_authoring_source_pointer(&entry.value)?;
            Ok(AuthorizationRegistryTrustFloor {
                generation: pointer.generation,
                digest: pointer.digest,
            })
        })
        .transpose()
}

async fn advance_manifest_pointer(
    store: &kv::Store,
    canonical_pointer: &str,
) -> Result<(), AuthorizationStateError> {
    let configured = parse_api_authoring_source_pointer(canonical_pointer.as_bytes())?;
    let existing = store
        .entry(MANIFEST_CURRENT_KEY.to_owned())
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
                    MANIFEST_CURRENT_KEY,
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
            let current = parse_api_authoring_source_pointer(&entry.value)?;
            if current.generation > configured.generation {
                return Err(storage("current issuer manifest pointer would roll back"));
            }
            if current.generation == configured.generation {
                if current != configured {
                    return Err(storage("current issuer manifest pointer equivocates"));
                }
                confirm_exact(store, MANIFEST_CURRENT_KEY, canonical_pointer.as_bytes()).await?;
                return Ok(());
            }
            store
                .update(
                    MANIFEST_CURRENT_KEY,
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
    confirm_exact(store, MANIFEST_CURRENT_KEY, canonical_pointer.as_bytes()).await
}

fn parse_api_authoring_source_pointer(
    value: &[u8],
) -> Result<ManifestPointer, AuthorizationStateError> {
    let pointer: ManifestPointer = serde_json::from_slice(value).map_err(|error| {
        storage(format!(
            "current issuer manifest pointer is invalid: {error}"
        ))
    })?;
    if pointer.generation == 0 || pointer.digest.is_empty() {
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

fn manifest_pointer(generation: u64, digest: String) -> Result<String, AuthorizationStateError> {
    canonicalize_json(
        &serde_json::to_value(ManifestPointer { generation, digest })
            .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))?,
    )
    .map_err(|error| storage(format!("cannot encode manifest pointer: {error}")))
}

fn context_registry_retention_seconds(
    config: &AuthorizationConfig,
) -> Result<u64, AuthorizationStateError> {
    let cleanup_retention = config
        .context_lifetime_seconds
        .checked_add(config.cleanup_grace_seconds)
        .ok_or_else(|| storage("context registry retention overflow"))?;
    let event_retention = crate::resources::EVENT_STREAM_RETENTION
        .as_secs()
        .checked_add(config.context_lifetime_seconds)
        .and_then(|value| value.checked_add(config.allowed_clock_skew_seconds))
        .ok_or_else(|| storage("event authorization evidence retention overflow"))?;
    Ok(cleanup_retention.max(event_retention))
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

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn context_registry_limit_uses_canonical_json_bytes() {
        assert_eq!(context_registry_value_bytes(16_384).unwrap(), 16_384);
    }

    #[test]
    fn registry_binding_matches_published_key_layout() {
        let binding = AuthorizationRegistryBinding::test_binding();
        assert_eq!(
            serde_json::to_value(binding).unwrap(),
            json!({
                "trustBucket": "trellis_authorization_trust",
                "contextBucket": "trellis_authorization_contexts"
            })
        );
    }

    #[test]
    fn manifest_current_rollback_below_durable_floor_rejects() {
        let sqlite_floor = AuthorizationTrustStateRecord {
            authority: "trellis-test".to_owned(),
            root_key_id: "root-key".to_owned(),
            root_digest: "root-digest".to_owned(),
            manifest_generation: 7,
            manifest_digest: "manifest-7".to_owned(),
            updated_at: 1,
            version: 1,
        };
        let registry_floor = AuthorizationRegistryTrustFloor {
            generation: 6,
            digest: "manifest-6".to_owned(),
        };
        let error = reconcile_trust_floor_values(
            6,
            "manifest-6",
            Some(&sqlite_floor),
            Some(&registry_floor),
        )
        .expect_err("a configured manifest below the durable floor must fail closed");
        assert!(error.to_string().contains("rolls back the SQLite floor"));

        let equivocated_registry_floor = AuthorizationRegistryTrustFloor {
            generation: 7,
            digest: "equivocated".to_owned(),
        };
        let error = reconcile_trust_floor_values(
            7,
            "manifest-7",
            Some(&sqlite_floor),
            Some(&equivocated_registry_floor),
        )
        .expect_err("same-generation pointer equivocation must fail closed");
        assert!(error.to_string().contains("NATS floor"));
    }

    #[test]
    fn manifest_pointer_and_revocation_are_additively_tolerant() {
        assert_eq!(
            manifest_pointer(7, "digest".to_owned()).unwrap(),
            r#"{"digest":"digest","generation":7}"#
        );
        assert_eq!(
            canonicalize_json(
                &serde_json::to_value(AuthorizationContextRevocationV1 { revoked_at: 42 }).unwrap()
            )
            .unwrap(),
            r#"{"revokedAt":42}"#
        );
        assert!(serde_json::from_value::<ManifestPointer>(json!({
            "generation": 7,
            "digest": "digest",
            "extra": true
        }))
        .is_ok());
        assert!(
            serde_json::from_value::<AuthorizationContextRevocationV1>(json!({
                "revokedAt": 42,
                "extra": true
            }))
            .is_ok()
        );
    }
}
