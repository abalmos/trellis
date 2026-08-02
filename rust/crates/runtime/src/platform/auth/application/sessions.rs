use serde_json::json;
use trellis_protocol::ParticipantKindV1;
use ulid::Ulid;

use super::super::*;

/// Input shared by user, service, and device session creation.
#[derive(Clone, Debug)]
pub struct CreateSessionInput {
    /// Stable authenticated principal ID.
    pub principal_id: String,
    /// Authenticated principal class.
    pub principal_kind: PrincipalKind,
    /// Exact participant ID.
    pub participant_id: String,
    /// Exact participant class.
    pub participant_kind: ParticipantKindV1,
    /// Exact participant artifact digest.
    pub participant_artifact_digest: String,
    /// Exact participant needs digest.
    pub participant_needs_digest: String,
    /// Canonical Ed25519 session public key.
    pub session_public_key: String,
    /// Optional identity authority accepted during this user bind.
    pub desired_authority: Option<DesiredAuthorityRecord>,
    /// Required deployment ID for service and device sessions.
    pub deployment_id: Option<String>,
    /// Required runtime instance ID for service and device sessions.
    pub instance_id: Option<String>,
    /// Creation time in Unix milliseconds.
    pub created_at: i64,
    /// Durable proof claim; its result is replaced with the committed session ID.
    pub idempotency: IdempotencyResultRecord,
    /// Deterministic post-commit actions.
    pub actions: Vec<PostCommitActionRecord>,
}

impl<R> AuthService<R>
where
    R: SessionRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + Clone,
{
    /// Create any principal session through the single aggregate path.
    ///
    /// This generates the session ID and inbox prefix, commits exact authority
    /// or runtime evidence, and reconciles the applicable authority. Replays
    /// retry reconciliation against the previously committed session.
    ///
    /// # Errors
    ///
    /// Returns an invalid-record error for mismatched principal, participant,
    /// deployment, or instance inputs; repository conflicts remain fail-closed.
    pub(crate) async fn create_session(
        &self,
        mut input: CreateSessionInput,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        super::validation::validate_idempotency_and_actions(&input.idempotency, &input.actions)?;
        super::super::domain::require_protocol_timestamp("createdAt", input.created_at)?;
        let expires_at = u64::try_from(input.created_at)
            .ok()
            .and_then(|created| created.checked_add(self.config.session_ttl_ms))
            .filter(|expires| *expires <= super::super::MAX_PROTOCOL_INTEGER)
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord("session expiry overflow".to_owned())
            })? as i64;
        let session_id = format!("ses_{}", Ulid::new());
        let session = SessionRecord::from_new(NewSession {
            session_id: session_id.clone(),
            principal_id: input.principal_id,
            principal_kind: input.principal_kind,
            participant_id: input.participant_id,
            participant_kind: input.participant_kind,
            participant_artifact_digest: input.participant_artifact_digest,
            participant_needs_digest: input.participant_needs_digest,
            session_public_key: input.session_public_key,
            inbox_prefix: format!("_INBOX.{session_id}"),
            created_at: input.created_at,
            expires_at: Some(expires_at),
        })?;
        super::super::authority::validate_session(&session)?;
        let runtime_binding = match (input.deployment_id, input.instance_id) {
            (None, None) => None,
            (Some(deployment_id), Some(instance_id)) => Some(SessionRuntimeBinding {
                session_id: session_id.clone(),
                deployment_id,
                instance_id,
            }),
            _ => {
                return Err(AuthorizationStateError::InvalidRecord(
                    "deploymentId and instanceId must be supplied together".to_owned(),
                ));
            }
        };
        if let Some(desired) = &input.desired_authority {
            super::validation::validate_session_desired_authority(&session, desired)?;
        }
        if let Some(binding) = &runtime_binding {
            super::super::authority::validate_session_runtime_binding(binding)?;
        }
        input.idempotency.result = json!({ "sessionId": session_id });
        let outcome = self
            .repository
            .create_session(SessionCreation {
                session: session.clone(),
                desired_authority: input.desired_authority,
                runtime_binding,
                idempotency: input.idempotency,
                actions: input.actions,
            })
            .await?;
        let committed = match &outcome {
            IdempotentOutcome::Applied(session) => session.clone(),
            IdempotentOutcome::Replayed(value) => {
                let session_id = value
                    .get("sessionId")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| {
                        AuthorizationStateError::Storage(
                            "session replay result has no sessionId".to_owned(),
                        )
                    })?;
                self.repository
                    .get_session(session_id)
                    .await?
                    .ok_or(AuthorizationStateError::SessionMissing)?
            }
        };
        let target = match committed.principal_kind {
            PrincipalKind::User => self
                .repository
                .get_identity_authority(&committed.principal_id, &committed.participant_id)
                .await?
                .map(|authority| AuthorityTarget {
                    kind: AuthorityKind::Identity,
                    authority_id: authority.authority_id,
                }),
            PrincipalKind::Service | PrincipalKind::Device => {
                let binding = self
                    .repository
                    .get_session_runtime_binding(&committed.session_id)
                    .await?
                    .ok_or(AuthorizationStateError::AuthorityMissing)?;
                self.repository
                    .get_deployment_authority(&binding.deployment_id, &committed.participant_id)
                    .await?
                    .map(|authority| AuthorityTarget {
                        kind: AuthorityKind::Deployment,
                        authority_id: authority.authority_id,
                    })
            }
        }
        .ok_or(AuthorizationStateError::AuthorityMissing)?;
        self.authorization
            .reconcile_authority(&target, input.created_at)
            .await?;
        Ok(outcome)
    }

    /// Revoke a session and durably enqueue its event and kick intents.
    ///
    /// # Errors
    ///
    /// Returns a conflict when the expected active version changed, or an
    /// invalid-record error unless both event and kick actions are supplied.
    pub(crate) async fn revoke_session(
        &self,
        session_id: String,
        expected_version: u64,
        revoked_at: i64,
        mut idempotency: IdempotencyResultRecord,
        actions: Vec<PostCommitActionRecord>,
    ) -> Result<IdempotentOutcome<SessionRecord>, AuthorizationStateError> {
        super::validation::validate_idempotency_and_actions(&idempotency, &actions)?;
        super::super::domain::require_protocol_timestamp("revokedAt", revoked_at)?;
        super::validation::validate_session_revocation_actions(&actions)?;
        idempotency.result = json!({ "sessionId": session_id, "state": "revoked" });
        self.repository
            .revoke_session(SessionRevocation {
                session_id,
                expected_version,
                revoked_at,
                idempotency,
                actions,
            })
            .await
    }
}
