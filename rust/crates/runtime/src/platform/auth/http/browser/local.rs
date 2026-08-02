use super::super::*;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowserFlowResponse {
    pub(crate) flow_id: String,
    pub(crate) state: AuthBrowserFlowState,
    pub(crate) expires_at: i64,
    pub(crate) providers: Vec<String>,
    pub(crate) registration_enabled: bool,
    pub(crate) federated_registration_enabled: bool,
    pub(crate) consent_view: Value,
    pub(crate) consent_view_digest: String,
    pub(crate) user: Option<BrowserFlowUser>,
    pub(crate) redirect_target: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowserFlowUser {
    origin: &'static str,
    id: String,
    name: Option<String>,
    email: Option<String>,
    image: Option<String>,
}

pub(crate) async fn get_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let flow = load_flow(&state.ephemeral, &flow_id).await?;
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    let providers = portal
        .provider_ids
        .iter()
        .filter(|provider| {
            (provider.as_str() == "local" && settings.local_login_enabled)
                || state.oidc_providers.contains_key(provider.as_str())
        })
        .cloned()
        .collect();
    let user = if let Some(principal_id) = &flow.principal_id {
        state
            .service
            .repository()
            .get_user_profile(principal_id)
            .await?
            .map(|profile| BrowserFlowUser {
                origin: "trellis",
                id: profile.principal_id,
                name: profile.display_name,
                email: profile.email,
                image: profile.image_url,
            })
    } else {
        None
    };
    let mut response = flow_response(flow);
    response.flow_id = flow_id;
    response.providers = providers;
    response.registration_enabled =
        portal.local_registration_enabled && settings.local_login_enabled;
    response.federated_registration_enabled = settings.federated_registration_enabled;
    response.user = user;
    Ok(Json(response))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalLoginRequest {
    flow_id: String,
    username: String,
    password: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AccountFlowResponse {
    status: &'static str,
    kind: Option<super::super::super::AccountFlowKind>,
    expires_at: Option<i64>,
}

pub(crate) async fn get_account_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_token): Path<String>,
) -> Result<Json<AccountFlowResponse>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let Some((_, flow)) =
        load_account_flow_by_token(state.service.repository(), &flow_token).await?
    else {
        return Ok(Json(AccountFlowResponse {
            status: "expired",
            kind: None,
            expires_at: None,
        }));
    };
    let status = if flow.state == AccountFlowState::Consumed {
        "consumed"
    } else if flow.state != AccountFlowState::Pending || flow.expires_at < now_ms()? {
        "expired"
    } else {
        "pending"
    };
    Ok(Json(AccountFlowResponse {
        status,
        kind: Some(flow.kind),
        expires_at: Some(flow.expires_at),
    }))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FirstAdminRequest {
    username: Option<String>,
    password: String,
    name: Option<String>,
    email: Option<String>,
}

pub(crate) async fn complete_first_admin<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_token): Path<String>,
    headers: HeaderMap,
    Json(request): Json<FirstAdminRequest>,
) -> Result<Json<Value>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone,
{
    require_portal_origin(&headers, &state.public_origin)?;
    let (token_hash, flow) = load_account_flow_by_token(state.service.repository(), &flow_token)
        .await?
        .ok_or_else(|| HttpError::gone("account_flow_expired"))?;
    if flow.state != AccountFlowState::Pending || flow.expires_at < now_ms()? {
        return Err(HttpError::conflict("account_flow_consumed"));
    }
    let now = now_ms()?;
    let request_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&request)
            .map_err(|_| HttpError::bad_request("invalid_account_flow"))?,
    )
    .map_err(|_| HttpError::bad_request("invalid_account_flow"))?;
    if flow.kind == super::super::super::AccountFlowKind::PasswordReset {
        let outcome = state
            .service
            .complete_password_reset(
                flow_token,
                flow.version,
                &request.password,
                now,
                idempotency(
                    &token_hash,
                    "account.password.reset",
                    flow.target_principal_id
                        .as_deref()
                        .unwrap_or("account-flow"),
                    &token_hash,
                    &request_digest,
                    now,
                )?,
                session_revocation_actions(
                    flow.target_principal_id
                        .as_deref()
                        .unwrap_or("account-flow"),
                    &token_hash,
                    now,
                    json!({
                        "principalId": flow.target_principal_id,
                        "reason": "password_reset",
                    }),
                ),
            )
            .await?;
        return match outcome {
            IdempotentOutcome::Applied(flow) => Ok(Json(json!({
                "status": "updated",
                "userId": flow.target_principal_id,
            }))),
            IdempotentOutcome::Replayed(_) => Err(HttpError::conflict("account_flow_consumed")),
        };
    }
    if flow.kind != super::super::super::AccountFlowKind::FirstAdmin {
        return Err(HttpError::bad_request("account_flow_provider_required"));
    }
    let username = request
        .username
        .clone()
        .ok_or_else(|| HttpError::bad_request("username_required"))?;
    let participant_id = flow.payload["participantId"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let participant_artifact_digest = flow.payload["participantArtifactDigest"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let participant_needs_digest = flow.payload["participantNeedsDigest"]
        .as_str()
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let binding = state
        .service
        .repository()
        .get_participant_binding(participant_id, participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::internal("first_admin_target_missing"))?;
    let grant_set = binding.resolve()?.proposal().required().grant_set().clone();
    let outcome = state
        .service
        .complete_first_admin(FirstAdminRegistration {
            token: flow_token,
            expected_flow_version: flow.version,
            username: username.clone(),
            password: request.password,
            display_name: request.name.unwrap_or(username),
            email: request.email,
            image_url: None,
            participant_id: participant_id.to_owned(),
            participant_artifact_digest: participant_artifact_digest.to_owned(),
            participant_needs_digest: participant_needs_digest.to_owned(),
            grant_set,
            authority_expires_at: None,
            completed_at: now,
            idempotency: idempotency(
                &token_hash,
                "first_admin.complete",
                "system:first-admin",
                &token_hash,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    match outcome {
        IdempotentOutcome::Applied(account) => Ok(Json(json!({
            "status": "created",
            "userId": account.principal.principal_id,
        }))),
        IdempotentOutcome::Replayed(_) => Err(HttpError::conflict("account_flow_consumed")),
    }
}

pub(crate) async fn local_login<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    headers: HeaderMap,
    Json(request): Json<LocalLoginRequest>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &request.flow_id).await?;
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if portal.disabled
        || portal.removed
        || !settings.local_login_enabled
        || !portal
            .provider_ids
            .iter()
            .any(|provider| provider == "local")
    {
        return Err(HttpError::forbidden("local_login_disabled"));
    }
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let now = now_ms()?;
    let principal = match state
        .service
        .authenticate_local(&request.username, &request.password, now)
        .await?
    {
        LocalAuthentication::Authenticated { principal, .. } => principal,
        LocalAuthentication::Denied => return Err(HttpError::unauthorized("invalid_credentials")),
    };
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Authenticated;
    flow.principal_id = Some(principal.principal_id);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::ApprovalRequired;
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let mut response = flow_response(flow);
    response.providers = vec!["local".to_owned()];
    Ok(Json(response))
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct LocalRegistrationRequest {
    username: String,
    password: String,
    name: Option<String>,
    email: Option<String>,
    idempotency_key: String,
}

pub(crate) async fn register_local<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<LocalRegistrationRequest>,
) -> Result<Json<BrowserFlowResponse>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + AuthorityRepository
        + ContextRepository
        + DeploymentRepository
        + OutboxRepository
        + PortalRepository
        + ProvisioningRepository
        + SessionRepository
        + Clone
        + Send
        + Sync
        + 'static,
    E: AuthEphemeralRepository + Clone,
{
    let mut flow = load_flow(&state.ephemeral, &flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if portal.disabled
        || portal.removed
        || !portal.local_registration_enabled
        || !settings.local_login_enabled
        || !portal
            .provider_ids
            .iter()
            .any(|provider| provider == "local")
    {
        return Err(HttpError::forbidden("local_registration_disabled"));
    }
    let now = now_ms()?;
    let request_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&request)
            .map_err(|_| HttpError::bad_request("invalid_registration"))?,
    )
    .map_err(|_| HttpError::bad_request("invalid_registration"))?;
    let account = state
        .service
        .create_local_user(CreateLocalUserInput {
            username: request.username,
            password: request.password,
            name: request.name,
            email: request.email,
            created_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.local.register",
                &flow.session_public_key,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let principal_id = match account {
        IdempotentOutcome::Applied(account) => account.principal.principal_id,
        IdempotentOutcome::Replayed(value) => value
            .get("principalId")
            .and_then(Value::as_str)
            .ok_or_else(|| HttpError::internal("invalid_registration_replay"))?
            .to_owned(),
    };
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Authenticated;
    flow.principal_id = Some(principal_id);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::ApprovalRequired;
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    Ok(Json(flow_response(flow)))
}
