use async_nats::jetstream::{self, stream};
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
    subscription: async_nats::Subscriber,
    subject: String,
    key: String,
}

impl futures_util::Stream for RegistryWatch {
    type Item = Result<RegistryWatchEntry, TrellisClientError>;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        match self.subscription.poll_next_unpin(cx) {
            std::task::Poll::Ready(Some(message)) => {
                if message.subject.as_str() != self.subject {
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
                    key: self.key.clone(),
                    value: message.payload.to_vec(),
                    removed,
                })))
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
    contexts: stream::Stream<()>,
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
            .get_stream_no_info(format!("KV_{}", binding.context_bucket))
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot open authorization registry: {error}"
                ))
            })?;
        Ok(Self {
            nats,
            contexts,
            binding: binding.clone(),
        })
    }

    pub(crate) async fn get_context(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        validate_digest_key(digest)?;
        let subject = format!("$KV.{}.{digest}", self.binding.context_bucket);
        match self.contexts.direct_get_last_for_subject(&subject).await {
            Ok(message)
                if message
                    .headers
                    .get("KV-Operation")
                    .is_none_or(|value| value.as_str() == "PUT") =>
            {
                Ok(Some(message.payload.to_vec()))
            }
            Ok(_) => Err(TrellisClientError::AuthorizationUnavailable(
                "authorization context was removed".into(),
            )),
            Err(error) if matches!(error.kind(), stream::DirectGetErrorKind::NotFound) => Ok(None),
            Err(error) => Err(TrellisClientError::AuthorizationUnavailable(format!(
                "cannot read authorization context: {error}"
            ))),
        }
    }

    pub(crate) async fn get_revocation(
        &self,
        digest: &str,
    ) -> Result<Option<Vec<u8>>, TrellisClientError> {
        validate_digest_key(digest)?;
        let subject = format!(
            "$KV.{}.{REVOCATION_PREFIX}{digest}",
            self.binding.context_bucket
        );
        match self.contexts.direct_get_last_for_subject(&subject).await {
            Ok(message)
                if message
                    .headers
                    .get("KV-Operation")
                    .is_none_or(|value| value.as_str() == "PUT") =>
            {
                Ok(Some(message.payload.to_vec()))
            }
            Ok(_) => Err(TrellisClientError::AuthorizationUnavailable(
                "authorization revocation was removed".into(),
            )),
            Err(error) if matches!(error.kind(), stream::DirectGetErrorKind::NotFound) => Ok(None),
            Err(error) => Err(TrellisClientError::AuthorizationUnavailable(format!(
                "cannot read authorization revocation: {error}"
            ))),
        }
    }

    pub(crate) async fn watch_revocation(
        &self,
        digest: &str,
    ) -> Result<RegistryWatch, TrellisClientError> {
        validate_digest_key(digest)?;
        let subject = format!(
            "$KV.{}.{REVOCATION_PREFIX}{digest}",
            self.binding.context_bucket
        );
        let key = format!("{REVOCATION_PREFIX}{digest}");
        let subscription = self
            .nats
            .subscribe(subject.clone())
            .await
            .map_err(|error| {
                TrellisClientError::AuthorizationUnavailable(format!(
                    "cannot subscribe to authorization revocation watch: {error}"
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
            subject,
            key,
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
