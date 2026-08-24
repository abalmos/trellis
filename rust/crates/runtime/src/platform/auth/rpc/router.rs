use super::*;

pub(super) async fn dispatch(
    processor: &AuthRpcProcessor,
    subject: &str,
    message: &async_nats::Message,
) -> Result<Value, AuthorizationStateError> {
    let headers = message.headers.as_ref().ok_or_else(|| {
        AuthorizationStateError::InvalidRecord("request headers missing".to_owned())
    })?;
    let header = |name: &str| -> Result<String, AuthorizationStateError> {
        headers
            .get(name)
            .map(|value| value.as_str().to_owned())
            .filter(|value| !value.is_empty())
            .ok_or_else(|| AuthorizationStateError::InvalidRecord(format!("{name} header missing")))
    };
    let session_key = header("session-key")?;
    let proof = header("proof")?;
    let authorization_context = header("authorization-context")?;
    let request_id = header("request-id")?;
    let iat = header("iat")?
        .parse()
        .map_err(|_| AuthorizationStateError::InvalidRecord("invalid iat header".to_owned()))?;
    let route = processor
        .routes
        .required_permission(subject, &message.payload)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(
                "request subject has no generated route metadata".to_owned(),
            )
        })?;
    let required_permission = route
        .permission_atom()
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let verified = processor
        .verifier
        .verify_request(
            super::super::verifier::RuntimeAuthorizationRequestVerificationInput {
                subject,
                payload: &message.payload,
                session_key: &session_key,
                proof: &proof,
                authorization_context: &authorization_context,
                iat,
                request_id: &request_id,
                reply: message.reply.as_deref(),
                required_permission: &required_permission,
                required_capabilities: &[],
            },
        )
        .await?;
    let validated = ValidatedRequest {
        principal_id: verified.caller.principal.id.clone(),
        principal_kind: match verified.caller.principal.kind {
            AuthorizationPrincipalKind::User => PrincipalKind::User,
            AuthorizationPrincipalKind::Service => PrincipalKind::Service,
            AuthorizationPrincipalKind::Device => PrincipalKind::Device,
        },
        session_id: verified.caller.session_id.clone(),
        session_public_key: session_key,
        capabilities: verified.context.capabilities().to_vec(),
    };
    workflows::dispatch(processor, subject, &message.payload, validated).await
}
