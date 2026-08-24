use trellis_protocol::{
    verify_authorization_event, verify_authorization_request, AuthorizationEventProof,
    AuthorizationEventPublisher, AuthorizationEventVerificationInput, AuthorizationRequestProof,
    AuthorizationRequestVerificationInput, AuthorizationVerificationPolicy, PermissionAtom,
    ProtocolError, VerifiedAuthorizationContext, VerifiedAuthorizationEventProof,
    VerifiedAuthorizationRequestProof,
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
    pub principal: trellis_protocol::AuthorizationPrincipal,
    /// Signed participant identity and artifact evidence.
    pub participant: trellis_protocol::AuthorizationParticipant,
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
    pub fn context(&self) -> &VerifiedAuthorizationContext {
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
    pub fn context(&self) -> &VerifiedAuthorizationContext {
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

/// Borrowed transport and authority inputs for shared request verification.
#[derive(Clone, Copy, Debug)]
pub struct RequestVerificationInput<'a> {
    /// Resolved and cryptographically verified authorization context.
    pub context: &'a VerifiedAuthorizationContext,
    /// Session key presented by the transport.
    pub session_key: &'a str,
    /// Context digest presented by the transport.
    pub context_digest: &'a str,
    /// Exact routed subject.
    pub subject: &'a str,
    /// Exact received payload bytes.
    pub payload: &'a [u8],
    /// Signed proof issue time.
    pub iat: i64,
    /// Signed request identifier.
    pub request_id: &'a str,
    /// Actual reply subject.
    pub reply_subject: Option<&'a str>,
    /// Encoded request proof.
    pub proof: &'a str,
    /// Verification policy.
    pub policy: &'a AuthorizationVerificationPolicy,
    /// Required exact permissions.
    pub required_permissions: &'a [PermissionAtom],
    /// Required platform capabilities.
    pub required_capabilities: &'a [String],
}

/// Borrowed transport and authority inputs for shared event verification.
#[derive(Clone, Copy, Debug)]
pub struct EventVerificationInput<'a> {
    /// Resolved and cryptographically verified authorization context.
    pub context: &'a VerifiedAuthorizationContext,
    /// Session key presented by the transport.
    pub session_key: &'a str,
    /// Context digest presented by the transport.
    pub context_digest: &'a str,
    /// Exact published subject.
    pub subject: &'a str,
    /// Exact received payload bytes.
    pub payload: &'a [u8],
    /// Signed event identifier.
    pub event_id: &'a str,
    /// Signed event time.
    pub event_time: &'a str,
    /// Encoded event proof.
    pub proof: &'a str,
    /// Verification policy.
    pub policy: &'a AuthorizationVerificationPolicy,
    /// Required exact permissions.
    pub required_permissions: &'a [PermissionAtom],
    /// Required platform capabilities.
    pub required_capabilities: &'a [String],
    /// Context revocation time, when present.
    pub revoked_at: Option<i64>,
}

impl AuthorizationVerificationCore {
    /// Create an empty local verification core.
    pub fn new() -> Self {
        Self::default()
    }

    /// Verify one context-bound request proof.
    pub fn verify_request(
        &self,
        input: RequestVerificationInput<'_>,
    ) -> Result<VerifiedAuthorizationRequest, AuthorizationVerificationError> {
        let RequestVerificationInput {
            context,
            session_key,
            context_digest,
            subject,
            payload,
            iat,
            request_id,
            reply_subject,
            proof,
            policy,
            required_permissions,
            required_capabilities,
        } = input;
        self.check_context_binding(context, session_key, context_digest)?;
        let proof = AuthorizationRequestProof::parse(proof.to_owned())?;
        let request = verify_authorization_request(AuthorizationRequestVerificationInput {
            context,
            subject,
            reply_subject,
            raw_payload: payload,
            iat,
            request_id,
            proof: &proof,
            policy,
            required_permissions,
            required_capabilities,
        })?;
        Ok(VerifiedAuthorizationRequest {
            caller: project_caller(session_key, request.context()),
            request,
        })
    }

    /// Verify one context-bound event proof.
    pub fn verify_event(
        &self,
        input: EventVerificationInput<'_>,
    ) -> Result<VerifiedAuthorizationEvent, AuthorizationVerificationError> {
        let EventVerificationInput {
            context,
            session_key,
            context_digest,
            subject,
            payload,
            event_id,
            event_time,
            proof,
            policy,
            required_permissions,
            required_capabilities,
            revoked_at,
        } = input;
        self.check_context_binding(context, session_key, context_digest)?;
        let proof = AuthorizationEventProof::parse(proof.to_owned())?;
        let event = verify_authorization_event(AuthorizationEventVerificationInput {
            context,
            subject,
            raw_payload: payload,
            event_id,
            event_time,
            proof: &proof,
            policy,
            required_permissions,
            required_capabilities,
            revoked_at,
        })?;
        Ok(VerifiedAuthorizationEvent { event })
    }

    fn check_context_binding(
        &self,
        context: &VerifiedAuthorizationContext,
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

fn project_caller(session_key: &str, context: &VerifiedAuthorizationContext) -> VerifiedCaller {
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
