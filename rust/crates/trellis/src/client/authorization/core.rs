use trellis_protocol::{
    verify_authorization_event, verify_authorization_request, AuthorizationEventProof,
    AuthorizationEventPublisher, AuthorizationRequestProof, AuthorizationVerificationPolicyV1,
    PermissionAtomV1, ProtocolError, VerifiedAuthorizationContextV1,
    VerifiedAuthorizationEventProof, VerifiedAuthorizationRequestProof,
};

/// Typed caller projection produced after a local authorization proof verifies.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedCaller {
    /// Session public key presented with the request.
    pub session_key: String,
    /// Signed reply-inbox prefix bound to the session.
    pub inbox_prefix: String,
    /// Digest of the complete signed authorization context.
    pub context_digest: String,
    /// Signed stable session id.
    pub session_id: String,
    /// Signed principal identity.
    pub principal: trellis_protocol::AuthorizationPrincipalV1,
    /// Signed participant identity and artifact evidence.
    pub participant: trellis_protocol::AuthorizationParticipantV1,
    /// Signed deployment identity, when present.
    pub deployment_id: Option<String>,
    /// Signed runtime instance identity, when present.
    pub instance_id: Option<String>,
}

/// Failure returned by the shared local authorization verification core.
#[derive(Debug, thiserror::Error)]
pub enum AuthorizationVerificationError {
    /// The presented digest does not identify the verified context.
    #[error("authorization context digest does not match verified context")]
    ContextDigestMismatch,
    /// The transport session key is not the key bound into the context.
    #[error("authorization session key does not match verified context")]
    SessionKeyMismatch,
    /// The protocol proof, permission, capability, or validity check failed.
    #[error("authorization proof rejected: {0}")]
    Protocol(Box<ProtocolError>),
}

impl From<ProtocolError> for AuthorizationVerificationError {
    fn from(error: ProtocolError) -> Self {
        Self::Protocol(Box::new(error))
    }
}

/// Verified request proof and its local caller projection.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationRequest {
    request: VerifiedAuthorizationRequestProof,
    caller: VerifiedCaller,
}

impl VerifiedAuthorizationRequest {
    /// Return the verified caller projection.
    pub fn caller(&self) -> &VerifiedCaller {
        &self.caller
    }

    /// Return the verified signed authorization context.
    pub fn context(&self) -> &VerifiedAuthorizationContextV1 {
        self.request.context()
    }
}

/// Verified event proof and its protocol publisher projection.
#[derive(Clone, Debug)]
pub struct VerifiedAuthorizationEvent {
    event: VerifiedAuthorizationEventProof,
}

impl VerifiedAuthorizationEvent {
    /// Return the verified publisher projection.
    pub fn publisher(&self) -> &AuthorizationEventPublisher {
        self.event.publisher()
    }

    /// Return the verified signed authorization context.
    pub fn context(&self) -> &VerifiedAuthorizationContextV1 {
        self.event.context()
    }
}

/// Shared local verifier used by both the platform and connected Rust SDKs.
///
/// Resolution of trust/context records stays with each provider cache. Once a
/// verified context is available, this core owns proof parsing, session-key
/// matching, exact permission and capability checks, and caller projection.
#[derive(Clone, Debug, Default)]
pub struct AuthorizationVerificationCore {}

impl AuthorizationVerificationCore {
    /// Create an empty local verification core.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify one context-bound request proof.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_request(
        &self,
        context: &VerifiedAuthorizationContextV1,
        session_key: &str,
        context_digest: &str,
        subject: &str,
        payload: &[u8],
        iat: i64,
        request_id: &str,
        reply_subject: Option<&str>,
        proof: &str,
        policy: &AuthorizationVerificationPolicyV1,
        required_permissions: &[PermissionAtomV1],
        required_capabilities: &[String],
    ) -> Result<VerifiedAuthorizationRequest, AuthorizationVerificationError> {
        self.check_context_binding(context, session_key, context_digest)?;
        let proof = AuthorizationRequestProof::parse(proof.to_owned())?;
        let request = verify_authorization_request(
            context,
            subject,
            reply_subject,
            payload,
            iat,
            request_id,
            &proof,
            policy,
            required_permissions,
            required_capabilities,
        )?;
        Ok(VerifiedAuthorizationRequest {
            caller: project_caller(session_key, request.context()),
            request,
        })
    }

    /// Verify one context-bound event proof.
    #[allow(clippy::too_many_arguments)]
    pub fn verify_event(
        &self,
        context: &VerifiedAuthorizationContextV1,
        session_key: &str,
        context_digest: &str,
        subject: &str,
        payload: &[u8],
        event_id: &str,
        event_time: &str,
        proof: &str,
        policy: &AuthorizationVerificationPolicyV1,
        required_permissions: &[PermissionAtomV1],
        required_capabilities: &[String],
        revoked_at: Option<i64>,
    ) -> Result<VerifiedAuthorizationEvent, AuthorizationVerificationError> {
        self.check_context_binding(context, session_key, context_digest)?;
        let proof = AuthorizationEventProof::parse(proof.to_owned())?;
        let event = verify_authorization_event(
            context,
            subject,
            payload,
            event_id,
            event_time,
            &proof,
            policy,
            required_permissions,
            required_capabilities,
            revoked_at,
        )?;
        Ok(VerifiedAuthorizationEvent { event })
    }

    fn check_context_binding(
        &self,
        context: &VerifiedAuthorizationContextV1,
        session_key: &str,
        context_digest: &str,
    ) -> Result<(), AuthorizationVerificationError> {
        if context.context_digest() != context_digest {
            return Err(AuthorizationVerificationError::ContextDigestMismatch);
        }
        if !session_key_matches(session_key, context.session_key()) {
            return Err(AuthorizationVerificationError::SessionKeyMismatch);
        }
        Ok(())
    }
}

fn project_caller(session_key: &str, context: &VerifiedAuthorizationContextV1) -> VerifiedCaller {
    VerifiedCaller {
        session_key: session_key.to_owned(),
        inbox_prefix: context.inbox_prefix().to_owned(),
        context_digest: context.context_digest().to_owned(),
        session_id: context.session_id().to_owned(),
        principal: context.principal().clone(),
        participant: context.participant().clone(),
        deployment_id: context.deployment_id().map(ToOwned::to_owned),
        instance_id: context.instance_id().map(ToOwned::to_owned),
    }
}

fn session_key_matches(encoded: &str, verifying_key: &ed25519_dalek::VerifyingKey) -> bool {
    use base64::Engine as _;

    let Ok(bytes) = base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(encoded) else {
        return false;
    };
    bytes.len() == 32 && verifying_key.to_bytes().as_slice() == bytes.as_slice()
}
