#[cfg(feature = "test-support")]
use std::sync::atomic::{AtomicU64, Ordering};
#[cfg(feature = "test-support")]
use std::sync::Arc;

use async_nats::jetstream::{self, consumer, kv::Store};
use futures_util::StreamExt;

use super::super::TrellisClientError;
use super::types::AuthorizationRegistryBinding;

/// Wire format of the `manifest.current` generation pointer.
pub(crate) const MANIFEST_CURRENT_KEY: &str = "manifest.current";
pub(crate) const MANIFEST_PREFIX: &str = "manifest.";
pub(crate) const REVOCATION_PREFIX: &str = "revocation.";

/// Parsed `manifest.current` generation pointer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ManifestPointer {
    pub(crate) generation: u64,
    pub(crate) digest: String,
}

pub(crate) struct RegistryWatchEntry {
    pub(crate) key: String,
    pub(crate) value: Vec<u8>,
    pub(crate) revision: u64,
    pub(crate) removed: bool,
}

pub(crate) struct RegistryWatch {
    subscription: consumer::push::Messages,
    prefix: String,
    seen_current: bool,
    initially_empty: bool,
}

impl RegistryWatch {
    pub(crate) fn initially_empty(&self) -> bool {
        self.initially_empty
    }
}

impl futures_util::Stream for RegistryWatch {
    type Item = Result<RegistryWatchEntry, TrellisClientError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.subscription.poll_next_unpin(cx) {
            std::task::Poll::Ready(Some(Ok(message))) => {
                let info = match message.info() {
                    Ok(info) => info,
                    Err(error) => {
                        return std::task::Poll::Ready(Some(Err(TrellisClientError::Bootstrap(
                            format!("authorization registry watch metadata is invalid: {error}"),
                        ))))
                    }
                };
                let key = match message.subject.strip_prefix(&self.prefix) {
                    Some(key) => key.to_owned(),
                    None => {
                        return std::task::Poll::Ready(Some(Err(TrellisClientError::Bootstrap(
                            "authorization registry watch subject is invalid".into(),
                        ))))
                    }
                };
                if !self.seen_current && info.pending == 0 {
                    self.seen_current = true;
                }
                let removed = message.headers.as_ref().is_some_and(|headers| {
                    headers
                        .get("KV-Operation")
                        .is_some_and(|value| matches!(value.as_str(), "DEL" | "PURGE"))
                });
                std::task::Poll::Ready(Some(Ok(RegistryWatchEntry {
                    key,
                    value: message.payload.to_vec(),
                    revision: info.stream_sequence,
                    removed,
                })))
            }
            std::task::Poll::Ready(Some(Err(error))) => {
                std::task::Poll::Ready(Some(Err(TrellisClientError::Bootstrap(format!(
                    "authorization registry watch failed: {error}"
                )))))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// Registry I/O counters observed since provider-cache start.
#[cfg(feature = "test-support")]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct AuthorizationRegistryIoCounters {
    /// Exact context fetches from the NATS KV context bucket.
    pub(crate) context_gets: u64,
    /// Exact manifest and pointer fetches from the NATS KV trust bucket.
    pub(crate) trust_gets: u64,
    /// Revocation watch initializations.
    pub(crate) revocation_watch_initializations: u64,
}

/// Connected NATS KV reader for authorization evidence.
///
/// The reader performs only exact key reads and ephemeral
/// watch consumers on the two registry buckets named by the bootstrap binding.
/// It never writes, deletes, or purges registry records.
#[derive(Clone)]
pub(crate) struct AuthorizationRegistryReader {
    nats: async_nats::Client,
    jetstream: jetstream::Context,
    trust: Store,
    contexts: Store,
    binding: AuthorizationRegistryBinding,
    #[cfg(feature = "test-support")]
    context_gets: Arc<AtomicU64>,
    #[cfg(feature = "test-support")]
    trust_gets: Arc<AtomicU64>,
    #[cfg(feature = "test-support")]
    revocation_watch_initializations: Arc<AtomicU64>,
}

impl AuthorizationRegistryReader {
    pub(crate) async fn open(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
    ) -> Result<Self, TrellisClientError> {
        if binding.trust_bucket.trim().is_empty() || binding.context_bucket.trim().is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization registry binding has empty buckets".into(),
            ));
        }
        let jetstream = jetstream::new(nats.clone());
        let trust = jetstream
            .get_key_value(binding.trust_bucket.clone())
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!(
                    "cannot open authorization trust registry: {error}"
                ))
            })?;
        let contexts = jetstream
            .get_key_value(binding.context_bucket.clone())
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!(
                    "cannot open authorization context registry: {error}"
                ))
            })?;
        Ok(Self {
            nats,
            jetstream,
            trust,
            contexts,
            binding: binding.clone(),
            #[cfg(feature = "test-support")]
            context_gets: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "test-support")]
            trust_gets: Arc::new(AtomicU64::new(0)),
            #[cfg(feature = "test-support")]
            revocation_watch_initializations: Arc::new(AtomicU64::new(0)),
        })
    }

    #[cfg(feature = "test-support")]
    pub(crate) fn io_counters(&self) -> AuthorizationRegistryIoCounters {
        AuthorizationRegistryIoCounters {
            context_gets: self.context_gets.load(Ordering::Relaxed),
            trust_gets: self.trust_gets.load(Ordering::Relaxed),
            revocation_watch_initializations: self
                .revocation_watch_initializations
                .load(Ordering::Relaxed),
        }
    }

    /// Exact context fetch by digest; `Ok(None)` when absent.
    pub(crate) async fn get_context(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        if !digest_key_is_valid(digest) {
            return Err(TrellisClientError::Bootstrap(
                "authorization context digest is invalid".into(),
            ));
        }
        #[cfg(feature = "test-support")]
        self.context_gets.fetch_add(1, Ordering::Relaxed);
        let value = self
            .contexts
            .get(digest.to_owned())
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!(
                    "cannot read authorization context {digest}: {error}"
                ))
            })?;
        Ok(value.map(|value| value.to_vec()))
    }

    /// Exact revocation fetch by context digest; `Ok(None)` when active.
    pub(crate) async fn get_revocation(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        if !digest_key_is_valid(digest) {
            return Err(TrellisClientError::Bootstrap(
                "authorization context digest is invalid".into(),
            ));
        }
        let key = format!("{REVOCATION_PREFIX}{digest}");
        let value = self.contexts.get(key).await.map_err(|error| {
            TrellisClientError::Bootstrap(format!(
                "cannot read authorization context revocation {digest}: {error}"
            ))
        })?;
        Ok(value.map(|value| value.to_vec()))
    }

    /// Exact issuer-manifest fetch by generation.
    pub(crate) async fn get_manifest(
        &self,
        generation: u64,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        let key = format!("{MANIFEST_PREFIX}{generation}");
        #[cfg(feature = "test-support")]
        self.trust_gets.fetch_add(1, Ordering::Relaxed);
        let value = self.trust.get(key.clone()).await.map_err(|error| {
            TrellisClientError::Bootstrap(format!("cannot read issuer manifest {key}: {error}"))
        })?;
        Ok(value.map(|value| value.to_vec()))
    }

    /// Read the current `manifest.current` generation pointer.
    pub(crate) async fn get_manifest_current(
        &self,
    ) -> Result<Option<(ManifestPointer, u64)>, TrellisClientError> {
        #[cfg(feature = "test-support")]
        self.trust_gets.fetch_add(1, Ordering::Relaxed);
        let entry = self
            .trust
            .entry(MANIFEST_CURRENT_KEY)
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!("cannot read manifest.current: {error}"))
            })?;
        let Some(entry) = entry.filter(|entry| !entry.value.is_empty()) else {
            return Ok(None);
        };
        let pointer = parse_api_authoring_source_pointer(&entry.value)?;
        Ok(Some((pointer, entry.revision)))
    }

    /// Watch `manifest.current` for generation advance.
    pub(crate) async fn watch_manifest_current(&self) -> Result<RegistryWatch, TrellisClientError> {
        self.watch(
            &self.binding.trust_bucket,
            MANIFEST_CURRENT_KEY,
            consumer::DeliverPolicy::LastPerSubject,
        )
        .await
    }

    /// Watch only one context's revocation record with current state first.
    pub(crate) async fn watch_revocation(
        &self,
        context_digest: &str,
    ) -> Result<RegistryWatch, TrellisClientError> {
        self.watch(
            &self.binding.context_bucket,
            &format!("{REVOCATION_PREFIX}{context_digest}"),
            consumer::DeliverPolicy::LastPerSubject,
        )
        .await
    }

    /// Record a revocation watch initialization.
    pub(crate) fn record_revocation_watch_initialization(&self) {
        #[cfg(feature = "test-support")]
        self.revocation_watch_initializations
            .fetch_add(1, Ordering::Relaxed);
    }

    async fn watch(
        &self,
        bucket: &str,
        key: &str,
        deliver_policy: consumer::DeliverPolicy,
    ) -> Result<RegistryWatch, TrellisClientError> {
        let stream = self
            .jetstream
            .get_stream_no_info(format!("KV_{bucket}"))
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!(
                    "cannot open authorization registry stream: {error}"
                ))
            })?;
        let prefix = format!("$KV.{bucket}.");
        let consumer = stream
            .create_consumer(consumer::push::Config {
                deliver_subject: self.nats.new_inbox(),
                name: Some(format!("TRELLIS_AUTH_{}", ulid::Ulid::new())),
                description: Some("trellis authorization registry watch".into()),
                filter_subject: format!("{prefix}{key}"),
                deliver_policy,
                ack_policy: consumer::AckPolicy::None,
                ..Default::default()
            })
            .await
            .map_err(|error| {
                TrellisClientError::Bootstrap(format!(
                    "cannot create authorization registry watch: {error}"
                ))
            })?;
        let initially_empty = consumer.cached_info().num_pending == 0;
        let subscription = consumer.messages().await.map_err(|error| {
            TrellisClientError::Bootstrap(format!(
                "cannot consume authorization registry watch: {error}"
            ))
        })?;
        Ok(RegistryWatch {
            subscription,
            prefix,
            seen_current: initially_empty,
            initially_empty,
        })
    }
}

fn digest_key_is_valid(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 256
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
}

pub(crate) fn parse_api_authoring_source_pointer(
    value: &[u8],
) -> Result<ManifestPointer, TrellisClientError> {
    #[derive(serde::Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Pointer {
        generation: u64,
        digest: String,
    }
    let pointer: Pointer = serde_json::from_slice(value).map_err(|error| {
        TrellisClientError::Bootstrap(format!(
            "current issuer manifest pointer is invalid: {error}"
        ))
    })?;
    if pointer.generation == 0 || !digest_key_is_valid(&pointer.digest) {
        return Err(TrellisClientError::Bootstrap(
            "current issuer manifest pointer is invalid".into(),
        ));
    }
    Ok(ManifestPointer {
        generation: pointer.generation,
        digest: pointer.digest,
    })
}

/// Revocation watch entry with KV delete tombstones mapped to removal.
///
pub(crate) enum RevocationWatchEntry {
    Applied {
        key: String,
        value: Vec<u8>,
        revision: u64,
    },
    Removed {
        key: String,
        revision: u64,
    },
}

/// Map one KV watch entry to a revocation watch event.
pub(crate) fn revocation_entry(entry: RegistryWatchEntry) -> RevocationWatchEntry {
    if entry.removed {
        RevocationWatchEntry::Removed {
            key: entry.key,
            revision: entry.revision,
        }
    } else {
        RevocationWatchEntry::Applied {
            key: entry.key,
            value: entry.value,
            revision: entry.revision,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_pointer_is_additively_tolerant() {
        let pointer = parse_api_authoring_source_pointer(
            br#"{"generation":7,"digest":"manifest-digest","future":true}"#,
        )
        .expect("extended pointer");
        assert_eq!(pointer.generation, 7);
        assert!(parse_api_authoring_source_pointer(
            br#"{"generation":0,"digest":"manifest-digest","future":true}"#
        )
        .is_err());
    }
}
