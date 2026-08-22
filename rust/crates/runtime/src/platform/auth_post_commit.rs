use async_nats::jetstream;
use bytes::Bytes;
use futures_util::{stream, StreamExt};
use serde_json::Value;
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use trellis_rs::client::SessionAuth;

use super::auth::{
    validate_connection_kick_response, AuthConnectionPresence, AuthEphemeralRepository,
    AuthorityEvidenceRepository, AuthorizationContextService, AuthorizationStateError,
    NatsAuthEphemeralRepository, OutboxRepository, PostCommitActionKind, PostCommitActionRecord,
    SessionRepository, SqliteAuthorizationStore,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const CLAIM_DURATION_MS: i64 = 30_000;
const IDLE_POLL_MS: u64 = 250;
const BATCH_SIZE: usize = 32;

pub(crate) struct AuthPostCommitRuntime {
    repository: SqliteAuthorizationStore,
    ephemeral: NatsAuthEphemeralRepository,
    auth_client: async_nats::Client,
    system_client: async_nats::Client,
    event_session: SessionAuth,
    event_context_digest: String,
    contexts: AuthorizationContextService,
}

impl AuthPostCommitRuntime {
    pub(crate) fn new(
        repository: SqliteAuthorizationStore,
        ephemeral: NatsAuthEphemeralRepository,
        auth_client: async_nats::Client,
        system_client: async_nats::Client,
        event_session: SessionAuth,
        event_context_digest: String,
        contexts: AuthorizationContextService,
    ) -> Self {
        Self {
            repository,
            ephemeral,
            auth_client,
            system_client,
            event_session,
            event_context_digest,
            contexts,
        }
    }

    pub(crate) async fn run(self, stop: StopHandle) -> Result<(), RuntimeError> {
        loop {
            let dispatched = tokio::select! {
                () = stop.stopped() => return Ok(()),
                result = self.dispatch_ready() => result
                    .map_err(|error| RuntimeError::Platform(error.to_string()))?,
            };
            if dispatched == 0 {
                tokio::select! {
                    () = stop.stopped() => return Ok(()),
                    () = tokio::time::sleep(std::time::Duration::from_millis(IDLE_POLL_MS)) => {}
                }
            }
        }
    }

    async fn dispatch_ready(&self) -> Result<usize, AuthorizationStateError> {
        let now = now_millis()?;
        let actions = self
            .repository
            .list_ready_post_commit_actions(now, BATCH_SIZE)
            .await?;
        let action_count = actions.len();
        let mut dispatches = stream::iter(actions)
            .map(|action| self.dispatch_action(action, now))
            .buffer_unordered(16);
        let mut first_error = None;
        while let Some(result) = dispatches.next().await {
            if let Err(error) = result {
                first_error.get_or_insert(error);
            }
        }
        if let Some(error) = first_error {
            return Err(error);
        }
        Ok(action_count)
    }

    async fn dispatch_action(
        &self,
        action: PostCommitActionRecord,
        now: i64,
    ) -> Result<(), AuthorizationStateError> {
        let claimed_until = now.saturating_add(CLAIM_DURATION_MS);
        let Some(action) = self
            .repository
            .claim_post_commit_action(&action.action_id, now, claimed_until)
            .await?
        else {
            return Ok(());
        };
        match self.dispatch(&action).await {
            Ok(()) => {
                self.repository
                    .acknowledge_post_commit_action(&action.action_id, claimed_until)
                    .await
            }
            Err(error) => {
                let delay = retry_delay_ms(action.attempts);
                self.repository
                    .fail_post_commit_action(
                        &action.action_id,
                        claimed_until,
                        now.saturating_add(delay),
                        error.to_string(),
                    )
                    .await
                    .map(|_| ())
            }
        }
    }

    async fn dispatch(
        &self,
        action: &PostCommitActionRecord,
    ) -> Result<(), AuthorizationStateError> {
        match action.kind {
            PostCommitActionKind::Event => self.publish_event(&action.payload).await,
            PostCommitActionKind::Kick => self.kick(&action.payload).await,
            PostCommitActionKind::ContextPublish => {
                self.dispatch_context(&action.payload, false).await
            }
            PostCommitActionKind::ContextRevoke => {
                self.dispatch_context(&action.payload, true).await
            }
        }
    }

    async fn dispatch_context(
        &self,
        payload: &Value,
        revocation: bool,
    ) -> Result<(), AuthorizationStateError> {
        let digest = payload
            .get("contextDigest")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "context post-commit digest is required".to_owned(),
                )
            })?;
        self.contexts
            .dispatch_registry_action(digest, revocation, now_millis()? / 1_000)
            .await?;
        if revocation {
            for connection in self
                .ephemeral
                .list_connection_presence_by_context(digest)
                .await?
            {
                self.kick_connection(&connection).await?;
            }
        }
        tracing::debug!(
            context_digest = digest,
            revocation,
            "published authorization context registry action"
        );
        Ok(())
    }

    async fn publish_event(&self, payload: &Value) -> Result<(), AuthorizationStateError> {
        let event_type = payload
            .get("eventType")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "post-commit eventType is required".to_owned(),
                )
            })?;
        let event_subject = payload
            .get("eventSubject")
            .and_then(Value::as_str)
            .map(str::to_owned)
            .unwrap_or_else(|| format!("events.v1.{event_type}"));
        let mut payload = payload.clone();
        let payload = payload.as_object_mut().ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "post-commit event payload must be an object".to_owned(),
            )
        })?;
        payload.remove("eventType");
        payload.remove("eventSubject");
        let event_id = payload
            .get("eventId")
            .and_then(Value::as_str)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("post-commit eventId is required".to_owned())
            })?;
        let occurred_at = payload
            .get("occurredAt")
            .and_then(Value::as_i64)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(
                    "post-commit occurredAt is required".to_owned(),
                )
            })?;
        let event_time =
            OffsetDateTime::from_unix_timestamp_nanos(i128::from(occurred_at) * 1_000_000)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
                .format(&Rfc3339)
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let payload = Bytes::from(
            serde_json::to_vec(&payload)
                .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
        );
        let mut headers = async_nats::HeaderMap::new();
        headers.insert("Nats-Msg-Id", event_id);
        headers.insert("Trellis-Event-Time", event_time.as_str());
        headers.insert("session-key", self.event_session.session_key.as_str());
        headers.insert("authorization-context", self.event_context_digest.as_str());
        headers.insert(
            "proof",
            self.event_session
                .create_event_proof(
                    &self.event_context_digest,
                    &event_subject,
                    &payload,
                    event_id,
                    &event_time,
                )
                .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
                .as_str(),
        );
        let ack = jetstream::new(self.auth_client.clone())
            .publish_with_headers(event_subject, headers, payload)
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        ack.await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        Ok(())
    }

    async fn kick(&self, payload: &Value) -> Result<(), AuthorizationStateError> {
        let connections = if let Some(session_id) = payload.get("sessionId").and_then(Value::as_str)
        {
            self.ephemeral
                .list_connection_presence(Some(session_id))
                .await?
        } else if let Some(connection_id) = payload.get("connectionId").and_then(Value::as_str) {
            self.ephemeral
                .list_connection_presence(None)
                .await?
                .into_iter()
                .filter(|connection| connection.connection_id == connection_id)
                .collect()
        } else if let Some(principal_id) = payload.get("principalId").and_then(Value::as_str) {
            let mut connections = Vec::new();
            for session in self
                .repository
                .list_sessions()
                .await?
                .into_iter()
                .filter(|session| {
                    session.principal_id == principal_id
                        && payload
                            .get("exceptSessionId")
                            .and_then(Value::as_str)
                            .is_none_or(|except| session.session_id != except)
                })
            {
                connections.extend(
                    self.ephemeral
                        .list_connection_presence(Some(&session.session_id))
                        .await?,
                );
            }
            connections
        } else if let Some(deployment_id) = payload.get("deploymentId").and_then(Value::as_str) {
            let mut connections = Vec::new();
            for session in self.repository.list_sessions().await? {
                if self
                    .repository
                    .get_session_runtime_binding(&session.session_id)
                    .await?
                    .is_some_and(|binding| binding.deployment_id == deployment_id)
                {
                    connections.extend(
                        self.ephemeral
                            .list_connection_presence(Some(&session.session_id))
                            .await?,
                    );
                }
            }
            connections
        } else {
            return Err(AuthorizationStateError::InvalidRecord(
                "post-commit kick target is required".to_owned(),
            ));
        };
        for connection in connections {
            self.kick_connection(&connection).await?;
        }
        Ok(())
    }

    async fn kick_connection(
        &self,
        connection: &AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError> {
        let client_id = connection
            .client_id
            .parse::<u64>()
            .map_err(|_| AuthorizationStateError::InvalidRecord("invalid client id".to_owned()))?;
        let response = self
            .system_client
            .request(
                format!("$SYS.REQ.SERVER.{}.KICK", connection.server_id),
                Bytes::from(
                    serde_json::to_vec(&serde_json::json!({ "cid": client_id }))
                        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?,
                ),
            )
            .await
            .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
        validate_connection_kick_response(&response.payload)?;
        self.ephemeral
            .delete_connection_presence(&connection.connection_id)
            .await
    }
}

fn retry_delay_ms(attempts: u32) -> i64 {
    1_000_i64.saturating_mul(1_i64 << attempts.min(6))
}

fn now_millis() -> Result<i64, AuthorizationStateError> {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| AuthorizationStateError::Storage("current time overflow".to_owned()))
}

#[cfg(test)]
mod tests {
    use super::retry_delay_ms;

    #[test]
    fn post_commit_retry_is_bounded() {
        assert_eq!(retry_delay_ms(0), 1_000);
        assert_eq!(retry_delay_ms(6), 64_000);
        assert_eq!(retry_delay_ms(u32::MAX), 64_000);
    }
}
