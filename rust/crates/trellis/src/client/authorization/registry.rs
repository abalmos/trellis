use async_nats::jetstream::{
    self, consumer,
    kv::{Operation, Store},
};
use futures_util::StreamExt;

use super::super::TrellisClientError;
use super::types::AuthorizationRegistryBinding;

pub(crate) const REVOCATION_PREFIX: &str = "revocation.";

pub(crate) struct RegistryWatchEntry {
    pub(crate) key: String,
    pub(crate) value: Vec<u8>,
    pub(crate) removed: bool,
}

pub(crate) struct RegistryWatch {
    subscription: consumer::push::Messages,
    prefix: String,
}

impl futures_util::Stream for RegistryWatch {
    type Item = Result<RegistryWatchEntry, TrellisClientError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.subscription.poll_next_unpin(cx) {
            std::task::Poll::Ready(Some(Ok(message))) => {
                if let Err(error) = message.info() {
                    return std::task::Poll::Ready(Some(Err(
                        TrellisClientError::AuthorizationUnavailable(format!(
                            "authorization watch metadata is invalid: {error}"
                        )),
                    )));
                }
                let Some(key) = message.subject.strip_prefix(&self.prefix) else {
                    return std::task::Poll::Ready(Some(Err(
                        TrellisClientError::AuthorizationUnavailable(
                            "authorization watch subject is invalid".into(),
                        ),
                    )));
                };
                let removed = message.headers.as_ref().is_some_and(|headers| {
                    headers
                        .get("KV-Operation")
                        .is_some_and(|value| matches!(value.as_str(), "DEL" | "PURGE"))
                });
                std::task::Poll::Ready(Some(Ok(RegistryWatchEntry {
                    key: key.to_owned(),
                    value: message.payload.to_vec(),
                    removed,
                })))
            }
            std::task::Poll::Ready(Some(Err(error))) => {
                std::task::Poll::Ready(Some(Err(TrellisClientError::AuthorizationUnavailable(
                    format!("authorization watch failed: {error}"),
                ))))
            }
            std::task::Poll::Ready(None) => std::task::Poll::Ready(None),
            std::task::Poll::Pending => std::task::Poll::Pending,
        }
    }
}

/// Exact context/revocation reads and watches on the server-assigned registry.
#[derive(Clone)]
pub(crate) struct AuthorizationRegistryReader {
    nats: async_nats::Client,
    jetstream: jetstream::Context,
    contexts: Store,
    binding: AuthorizationRegistryBinding,
}

impl AuthorizationRegistryReader {
    pub(crate) async fn open(
        nats: async_nats::Client,
        binding: &AuthorizationRegistryBinding,
    ) -> Result<Self, TrellisClientError> {
        if binding.context_bucket.trim().is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization registry bucket is empty".into(),
            ));
        }
        let jetstream = jetstream::new(nats.clone());
        let contexts = jetstream
            .get_key_value(binding.context_bucket.clone())
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot open authorization registry: {error}"
                ))
            })?;
        Ok(Self {
            nats,
            jetstream,
            contexts,
            binding: binding.clone(),
        })
    }

    pub(crate) async fn get_context(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        validate_digest_key(digest)?;
        self.contexts
            .get(digest.to_owned())
            .await
            .map(|value| value.map(|value| value.to_vec()))
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot read authorization context: {error}"
                ))
            })
    }

    pub(crate) async fn get_revocation(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        validate_digest_key(digest)?;
        let entry = self
            .contexts
            .entry(format!("{REVOCATION_PREFIX}{digest}"))
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot read authorization revocation: {error}"
                ))
            })?;
        match entry {
            None => Ok(None),
            Some(entry) if entry.operation == Operation::Put => Ok(Some(entry.value.to_vec())),
            Some(_) => Err(TrellisClientError::AuthorizationUnavailable(
                "authorization revocation was removed".into(),
            )),
        }
    }

    pub(crate) async fn watch_revocation(
        &self,
        digest: &str,
    ) -> Result<RegistryWatch, TrellisClientError> {
        validate_digest_key(digest)?;
        let stream = self
            .jetstream
            .get_stream_no_info(format!("KV_{}", self.binding.context_bucket))
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot open authorization registry stream: {error}"
                ))
            })?;
        let prefix = format!("$KV.{}.", self.binding.context_bucket);
        let consumer = stream
            .create_consumer(consumer::push::Config {
                deliver_subject: self.nats.new_inbox(),
                name: Some(format!("TRELLIS_AUTH_{}", ulid::Ulid::new())),
                description: Some("trellis exact context revocation watch".into()),
                filter_subject: format!("{prefix}{REVOCATION_PREFIX}{digest}"),
                deliver_policy: consumer::DeliverPolicy::LastPerSubject,
                ack_policy: consumer::AckPolicy::None,
                idle_heartbeat: std::time::Duration::from_secs(5),
                inactive_threshold: std::time::Duration::from_secs(10),
                ..Default::default()
            })
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot create authorization revocation watch: {error}"
                ))
            })?;
        let subscription = consumer.messages().await.map_err(|error| {
            TrellisClientError::AuthorizationUnavailable(format!(
                "cannot consume authorization revocation watch: {error}"
            ))
        })?;
        // Establish subscription interest before the caller's exact revocation read.
        self.nats.flush().await.map_err(|error| {
            TrellisClientError::AuthorizationUnavailable(format!(
                "cannot establish authorization revocation watch: {error}"
            ))
        })?;
        Ok(RegistryWatch {
            subscription,
            prefix,
        })
    }
}

pub(crate) fn validate_digest_key(digest: &str) -> Result<(), TrellisClientError> {
    if digest.len() != 43
        || !digest
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization context digest is invalid".into(),
        ));
    }
    Ok(())
}
