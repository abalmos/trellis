use super::super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct OidcStartQuery {
    flow_id: String,
}

pub(crate) async fn start_oidc<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(provider_id): Path<String>,
    Query(query): Query<OidcStartQuery>,
) -> Result<Response, HttpError>
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
    let flow = load_flow(&state.ephemeral, &query.flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let (portal, settings) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    if portal.disabled || portal.removed || !portal.provider_ids.contains(&provider_id) {
        return Err(HttpError::forbidden("provider_not_allowed"));
    }
    begin_oidc(
        &state,
        provider_id,
        flow.flow_id,
        AuthOAuthKind::Browser,
        Some((&portal, &settings)),
    )
    .await
}

pub(crate) async fn start_account_flow_oidc<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path((flow_token, provider_id)): Path<(String, String)>,
) -> Result<Response, HttpError>
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
        return Err(HttpError::gone("account_flow_expired"));
    };
    if !matches!(
        flow.kind,
        super::super::super::AccountFlowKind::IdentityLink
            | super::super::super::AccountFlowKind::FirstAdmin
    ) || flow.state != AccountFlowState::Pending
        || flow.expires_at < now_ms()?
        || flow
            .target_provider_id
            .as_ref()
            .is_some_and(|target| target != &provider_id)
    {
        return Err(HttpError::conflict("account_flow_not_eligible"));
    }
    let portal = if flow.kind == super::super::super::AccountFlowKind::FirstAdmin {
        let portal = state
            .service
            .repository()
            .get_login_portal("builtin")
            .await?
            .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
        if portal.0.disabled || portal.0.removed || !portal.0.provider_ids.contains(&provider_id) {
            return Err(HttpError::forbidden("provider_not_allowed"));
        }
        Some(portal)
    } else {
        None
    };
    begin_oidc(
        &state,
        provider_id,
        flow_token,
        AuthOAuthKind::AccountFlow,
        portal.as_ref().map(|(portal, settings)| (portal, settings)),
    )
    .await
}

async fn begin_oidc<R, E>(
    state: &AuthHttpState<R, E>,
    provider_id: String,
    flow_id: String,
    kind: AuthOAuthKind,
    portal_policy: Option<(&LoginPortalRecord, &LoginSettingsRecord)>,
) -> Result<Response, HttpError>
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
    let provider = state
        .oidc_providers
        .get(&provider_id)
        .ok_or_else(|| HttpError::not_found("provider_not_found"))?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        provider.client_id.clone(),
        provider.client_secret.clone(),
    )
    .set_redirect_uri(provider.redirect_uri.clone());
    let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();
    let mut authorization = client
        .authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        )
        .set_pkce_challenge(pkce_challenge);
    for scope in &provider.scopes {
        authorization = authorization.add_scope(scope.clone());
    }
    let (authorization_url, csrf, nonce) = authorization.url();
    let now = now_ms()?;
    let mut browser_binding = [0_u8; 32];
    getrandom::fill(&mut browser_binding)
        .map_err(|_| HttpError::internal("oauth_browser_binding_generation"))?;
    let browser_binding = URL_SAFE_NO_PAD.encode(browser_binding);
    let browser_binding_digest = URL_SAFE_NO_PAD.encode(Sha256::digest(&browser_binding));
    let (portal_id, portal_policy_digest) = match portal_policy {
        Some((portal, settings)) => (
            Some(portal.portal_id.clone()),
            Some(oidc_portal_policy_digest(portal, settings)?),
        ),
        None => (None, None),
    };
    state
        .ephemeral
        .create_oauth_state(AuthOAuthState {
            format: "trellis.auth-oauth-state.v1".to_owned(),
            state_id: csrf.secret().clone(),
            provider_id: provider_id.clone(),
            kind,
            flow_id,
            status: AuthOAuthStatus::Pending,
            pkce_verifier: pkce_verifier.secret().clone(),
            nonce: nonce.secret().clone(),
            redirect_uri: provider.redirect_uri.as_str().to_owned(),
            browser_binding_digest,
            portal_id,
            portal_policy_digest,
            claim_owner: None,
            result_digest: None,
            created_at: now,
            expires_at: checked_add(now, 15 * 60_000)?,
            version: 1,
        })
        .await?;
    let mut response = Redirect::temporary(authorization_url.as_str()).into_response();
    response.headers_mut().append(
        SET_COOKIE,
        oauth_cookie_header(
            &oauth_cookie_name(csrf.secret()),
            &browser_binding,
            &provider_id,
            state.public_origin.starts_with("https://"),
            15 * 60,
        )?,
    );
    Ok(response)
}

#[derive(Deserialize)]
pub(crate) struct OidcCallbackQuery {
    code: Option<String>,
    state: String,
    error: Option<String>,
}

pub(crate) async fn oidc_callback<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(provider_id): Path<String>,
    Query(query): Query<OidcCallbackQuery>,
    headers: HeaderMap,
) -> Response
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
    let cookie_name = oauth_cookie_name(&query.state);
    let mut response = match oidc_callback_inner(&state, &provider_id, query, &headers).await {
        Ok(response) => response,
        Err(error) => error.into_response(),
    };
    if let Ok(cookie) = oauth_cookie_header(
        &cookie_name,
        "",
        &provider_id,
        state.public_origin.starts_with("https://"),
        0,
    ) {
        response.headers_mut().append(SET_COOKIE, cookie);
    }
    response
}

async fn oidc_callback_inner<R, E>(
    state: &AuthHttpState<R, E>,
    provider_id: &str,
    query: OidcCallbackQuery,
    headers: &HeaderMap,
) -> Result<Response, HttpError>
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
    let pending = state
        .ephemeral
        .get_oauth_state(&query.state)
        .await?
        .ok_or_else(|| HttpError::bad_request("oauth_state_invalid"))?;
    if pending.status != AuthOAuthStatus::Pending
        || pending.provider_id != provider_id
        || pending.expires_at < now_ms()?
    {
        return Err(HttpError::bad_request("oauth_state_invalid"));
    }
    require_oauth_browser_binding(&query.state, &pending, headers)?;
    let provider = state
        .oidc_providers
        .get(provider_id)
        .ok_or_else(|| HttpError::not_found("provider_not_found"))?;
    if pending.redirect_uri != provider.redirect_uri.as_str() {
        return Err(HttpError::bad_request("oauth_state_invalid"));
    }
    let federated_registration_enabled = if let Some(portal_id) = pending.portal_id.as_deref() {
        let (portal, settings) = state
            .service
            .repository()
            .get_login_portal(portal_id)
            .await?
            .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
        if portal.disabled
            || portal.removed
            || !portal.provider_ids.iter().any(|value| value == provider_id)
            || pending.portal_policy_digest.as_deref()
                != Some(oidc_portal_policy_digest(&portal, &settings)?.as_str())
        {
            return Err(HttpError::conflict("oauth_policy_changed"));
        }
        settings.federated_registration_enabled
    } else {
        false
    };
    match pending.kind {
        AuthOAuthKind::Browser => {
            let flow = load_flow(&state.ephemeral, &pending.flow_id).await?;
            if flow.state != AuthBrowserFlowState::ChooseProvider
                || pending.portal_id.as_deref() != Some(flow.portal_id.as_str())
            {
                return Err(HttpError::conflict("oauth_flow_changed"));
            }
        }
        AuthOAuthKind::AccountFlow => {
            let Some((_, flow)) =
                load_account_flow_by_token(state.service.repository(), &pending.flow_id).await?
            else {
                return Err(HttpError::gone("account_flow_expired"));
            };
            if !matches!(
                flow.kind,
                super::super::super::AccountFlowKind::IdentityLink
                    | super::super::super::AccountFlowKind::FirstAdmin
            ) || flow.state != AccountFlowState::Pending
                || flow.expires_at < now_ms()?
                || flow
                    .target_provider_id
                    .as_deref()
                    .is_some_and(|target| target != provider_id)
                || (flow.kind == super::super::super::AccountFlowKind::FirstAdmin
                    && pending.portal_id.as_deref() != Some("builtin"))
                || (flow.kind == super::super::super::AccountFlowKind::IdentityLink
                    && pending.portal_id.is_some())
            {
                return Err(HttpError::conflict("oauth_flow_changed"));
            }
        }
    }
    let claim_owner = format!("callback_{}", URL_SAFE_NO_PAD.encode(getrandom_bytes()?));
    let mut oauth = claim_oauth_state(&state.ephemeral, &query.state, &claim_owner).await?;
    if query.error.is_some() {
        let expected = oauth.version;
        oauth.status = AuthOAuthStatus::Expired;
        oauth.version += 1;
        state.ephemeral.replace_oauth_state(expected, oauth).await?;
        return Err(HttpError::bad_request("oauth_denied"));
    }
    let code = query
        .code
        .ok_or_else(|| HttpError::bad_request("oauth_code_missing"))?;
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::ExchangeStarted;
    oauth.version += 1;
    state
        .ephemeral
        .replace_oauth_state(expected, oauth.clone())
        .await?;
    let client = CoreClient::from_provider_metadata(
        provider.metadata.clone(),
        provider.client_id.clone(),
        provider.client_secret.clone(),
    )
    .set_redirect_uri(provider.redirect_uri.clone());
    let http_client = openidconnect::reqwest::ClientBuilder::new()
        .redirect(openidconnect::reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| HttpError::internal("oauth_http_client"))?;
    let token = match client
        .exchange_code(AuthorizationCode::new(code))
        .map_err(|_| HttpError::bad_request("oauth_exchange_invalid"))?
        .set_pkce_verifier(PkceCodeVerifier::new(oauth.pkce_verifier.clone()))
        .request_async(&http_client)
        .await
    {
        Ok(token) => token,
        Err(_) => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::bad_gateway("oauth_exchange_failed"));
        }
    };
    let id_token = match token.id_token() {
        Some(id_token) => id_token,
        None => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::bad_gateway("oauth_id_token_missing"));
        }
    };
    let verifier = client.id_token_verifier();
    let claims = match id_token.claims(&verifier, &Nonce::new(oauth.nonce.clone())) {
        Ok(claims) => claims,
        Err(_) => {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("oauth_id_token_invalid"));
        }
    };
    if let Some(expected) = claims.access_token_hash() {
        let actual = AccessTokenHash::from_token(
            token.access_token(),
            id_token
                .signing_alg()
                .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?,
            id_token
                .signing_key(&verifier)
                .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?,
        )
        .map_err(|_| HttpError::unauthorized("oauth_id_token_invalid"))?;
        if actual != *expected {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("oauth_access_token_mismatch"));
        }
    }
    let subject = claims.subject().as_str().to_owned();
    let now = now_ms()?;
    if oauth.kind == AuthOAuthKind::AccountFlow {
        let Some((token_hash, flow)) =
            load_account_flow_by_token(state.service.repository(), &oauth.flow_id).await?
        else {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::gone("account_flow_expired"));
        };
        if !matches!(
            flow.kind,
            super::super::super::AccountFlowKind::IdentityLink
                | super::super::super::AccountFlowKind::FirstAdmin
        ) || flow.state != AccountFlowState::Pending
            || flow.expires_at < now
            || flow
                .target_provider_id
                .as_ref()
                .is_some_and(|target| target != provider_id)
        {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::conflict("account_flow_not_eligible"));
        }
        if flow.kind == super::super::super::AccountFlowKind::FirstAdmin {
            if state
                .service
                .repository()
                .get_provider_identity(provider_id, &subject)
                .await?
                .is_some()
            {
                mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
                return Err(HttpError::unauthorized("federated_login_denied"));
            }
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
            if binding.needs_digest != participant_needs_digest {
                return Err(HttpError::conflict("first_admin_target_changed"));
            }
            let grant_set = binding.resolve()?.proposal().required().grant_set().clone();
            let digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
            let outcome = state
                .service
                .complete_first_admin_federated(FirstAdminFederatedRegistration {
                    token: oauth.flow_id.clone(),
                    expected_flow_version: flow.version,
                    provider: provider_id.to_owned(),
                    provider_subject: subject,
                    display_name: None,
                    email: claims.email().map(|email| email.as_str().to_owned()),
                    image_url: None,
                    participant_id: participant_id.to_owned(),
                    participant_artifact_digest: participant_artifact_digest.to_owned(),
                    participant_needs_digest: participant_needs_digest.to_owned(),
                    grant_set,
                    authority_expires_at: None,
                    completed_at: now,
                    idempotency: idempotency(
                        &token_hash,
                        "first_admin.federated.complete",
                        "system:first-admin",
                        &oauth.state_id,
                        &digest,
                        now,
                    )?,
                    actions: Vec::new(),
                })
                .await?;
            let principal_id = match outcome {
                IdempotentOutcome::Applied(account) => account.principal.principal_id,
                IdempotentOutcome::Replayed(_) => {
                    return Err(HttpError::conflict("account_flow_consumed"));
                }
            };
            let expected = oauth.version;
            oauth.status = AuthOAuthStatus::Completed;
            oauth.result_digest = Some(digest_parts(&[&principal_id]));
            oauth.version += 1;
            state.ephemeral.replace_oauth_state(expected, oauth).await?;
            return Ok(Redirect::temporary(&format!(
                "{}/_trellis/portal/account/complete",
                state.public_origin.trim_end_matches('/')
            ))
            .into_response());
        }
        let principal_id = flow
            .target_principal_id
            .clone()
            .ok_or_else(|| HttpError::internal("identity_link_target_missing"))?;
        if state
            .service
            .repository()
            .get_provider_identity(provider_id, &subject)
            .await?
            .is_some()
        {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::conflict("provider_identity_already_linked"));
        }
        let digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
        let outcome = state
            .service
            .complete_identity_link(CompleteIdentityLinkInput {
                token: oauth.flow_id.clone(),
                expected_flow_version: flow.version,
                identity: super::super::super::ProviderIdentityLink {
                    provider: provider_id.to_owned(),
                    provider_subject: subject,
                    principal_id: principal_id.clone(),
                    linked_at: now,
                    last_seen_at: now,
                },
                completed_at: now,
                idempotency: idempotency(
                    &token_hash,
                    "account.identity.link",
                    &principal_id,
                    &oauth.state_id,
                    &digest,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?;
        if matches!(outcome, IdempotentOutcome::Replayed(_)) {
            return Err(HttpError::conflict("account_flow_consumed"));
        }
        let expected = oauth.version;
        oauth.status = AuthOAuthStatus::Completed;
        oauth.result_digest = Some(digest_parts(&[&principal_id]));
        oauth.version += 1;
        state.ephemeral.replace_oauth_state(expected, oauth).await?;
        return Ok(Redirect::temporary(&format!(
            "{}/_trellis/portal/account/complete",
            state.public_origin.trim_end_matches('/')
        ))
        .into_response());
    }
    let principal_id = if let Some(identity) = state
        .service
        .repository()
        .get_provider_identity(provider_id, &subject)
        .await?
    {
        let principal = state
            .service
            .repository()
            .get_principal(&identity.principal_id)
            .await?
            .ok_or_else(|| HttpError::internal("provider_principal_missing"))?;
        if principal.state != super::super::super::PrincipalState::Active {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::forbidden("account_inactive"));
        }
        principal.principal_id
    } else {
        if !federated_registration_enabled {
            mark_oauth_restart_required(&state.ephemeral, &mut oauth).await?;
            return Err(HttpError::unauthorized("federated_login_denied"));
        }
        let claim_digest = digest_parts(&[provider_id, &subject, &oauth.state_id]);
        match state
            .service
            .create_federated_user(CreateFederatedUserInput {
                provider: provider_id.to_owned(),
                provider_subject: subject,
                name: None,
                email: claims.email().map(|email| email.as_str().to_owned()),
                image: None,
                created_at: now,
                idempotency: idempotency(
                    &oauth.state_id,
                    "oauth.account.bind",
                    provider_id,
                    &oauth.state_id,
                    &claim_digest,
                    now,
                )?,
                actions: Vec::new(),
            })
            .await?
        {
            IdempotentOutcome::Applied(account) => account.principal.principal_id,
            IdempotentOutcome::Replayed(value) => value
                .get("principalId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_account_replay"))?
                .to_owned(),
        }
    };
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::Completed;
    oauth.result_digest = Some(digest_parts(&[&principal_id]));
    oauth.version += 1;
    state
        .ephemeral
        .replace_oauth_state(expected, oauth.clone())
        .await?;
    let mut flow = load_flow(&state.ephemeral, &oauth.flow_id).await?;
    if flow.state != AuthBrowserFlowState::ChooseProvider {
        return Err(HttpError::conflict("flow_not_pending"));
    }
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
    Ok(Redirect::temporary(&format!(
        "{}/_trellis/portal/auth?flowId={}",
        state.public_origin.trim_end_matches('/'),
        flow.flow_id
    ))
    .into_response())
}

async fn mark_oauth_restart_required(
    repository: &impl AuthEphemeralRepository,
    oauth: &mut AuthOAuthState,
) -> Result<(), HttpError> {
    let expected = oauth.version;
    oauth.status = AuthOAuthStatus::RestartRequired;
    oauth.version += 1;
    repository
        .replace_oauth_state(expected, oauth.clone())
        .await
        .map_err(Into::into)
}
