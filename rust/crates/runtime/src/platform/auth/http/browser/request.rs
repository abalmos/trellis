use super::super::*;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(in crate::platform::auth::http) struct AuthStartRequest {
    request_id: String,
    issued_at: i64,
    session_public_key: String,
    session_nkey: String,
    participant_id: String,
    participant_artifact_digest: String,
    participant_needs_digest: String,
    participant_artifact: Option<Value>,
    #[serde(default)]
    referenced_api_artifacts: Vec<Value>,
    redirect_target: String,
    proof: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AuthStartResponse {
    flow_id: String,
    login_url: String,
}

pub(crate) async fn start_auth<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Json(raw): Json<Value>,
) -> Result<Json<AuthStartResponse>, HttpError>
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
    let request: AuthStartRequest = serde_json::from_value(raw.clone()).map_err(|error| {
        tracing::warn!(%error, "invalid auth request shape");
        HttpError::bad_request("invalid_auth_request")
    })?;
    validate_redirect(&request.redirect_target, &state.allowed_redirect_origins)?;
    let request_digest = proof_request_digest(&raw).map_err(|error| {
        tracing::warn!(%error, "invalid auth request proof envelope");
        HttpError::bad_request("invalid_auth_request")
    })?;
    let input = SessionProofInput::user_auth_request(UserAuthRequestSessionProofInput {
        request_id: request.request_id.clone(),
        issued_at: request.issued_at,
        session_public_key: request.session_public_key.clone(),
        session_nkey: request.session_nkey.clone(),
        participant_id: request.participant_id.clone(),
        participant_digest: request.participant_artifact_digest.clone(),
        redirect_target: request.redirect_target.clone(),
        request_digest: request_digest.clone(),
    })
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let proof = parse_session_proof(&request.proof)
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let now = now_ms()?;
    let (portal, _) = select_login_portal(
        state.service.repository(),
        &request.participant_id,
        &request.redirect_target,
    )
    .await?;
    verify_session_proof(
        &input,
        &proof,
        &request.session_public_key,
        now,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    if let Some(participant_value) = &request.participant_artifact {
        let participant = parse_participant(participant_value)
            .map_err(|_| HttpError::bad_request("invalid_participant_artifact"))?;
        let mut apis = BTreeMap::new();
        let mut api_values = BTreeMap::new();
        for value in &request.referenced_api_artifacts {
            let api =
                parse_api(value).map_err(|_| HttpError::bad_request("invalid_api_artifact"))?;
            api_values.insert(
                api.id().to_owned(),
                api.normalized_value()
                    .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
            );
            apis.insert(api.id().to_owned(), api);
        }
        let resolved = resolve_participant(&participant, &apis)
            .map_err(|_| HttpError::bad_request("participant_resolution_failed"))?;
        let needs_digest = resolved
            .needs()
            .digest()
            .map_err(|_| HttpError::bad_request("participant_resolution_failed"))?;
        if resolved.participant_id() != request.participant_id
            || resolved.participant_digest() != request.participant_artifact_digest
            || needs_digest != request.participant_needs_digest
        {
            tracing::warn!(
                participant_id = %request.participant_id,
                "auth request participant presentation does not match its declared digests"
            );
            return Err(HttpError::bad_request("participant_binding_mismatch"));
        }
        state
            .service
            .repository()
            .put_participant_binding(ParticipantBindingRecord {
                participant_id: resolved.participant_id().to_owned(),
                participant_kind: resolved.participant_kind(),
                artifact_digest: resolved.participant_digest().to_owned(),
                needs_digest,
                participant_json: participant
                    .canonical_json()
                    .map_err(|_| HttpError::bad_request("invalid_participant_artifact"))?,
                api_artifacts_json: canonicalize_json(
                    &serde_json::to_value(api_values)
                        .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
                )
                .map_err(|_| HttpError::bad_request("invalid_api_artifact"))?,
                resolved_at: now,
                state: ParticipantBindingState::Resolved,
                error: None,
            })
            .await?;
    }
    let binding = state
        .service
        .repository()
        .get_participant_binding(
            &request.participant_id,
            &request.participant_artifact_digest,
        )
        .await?
        .ok_or_else(|| {
            tracing::warn!(participant_id = %request.participant_id, "auth request participant binding is unknown");
            HttpError::bad_request("participant_binding_unknown")
        })?;
    if binding.state != ParticipantBindingState::Resolved
        || binding.needs_digest != request.participant_needs_digest
    {
        tracing::warn!(participant_id = %request.participant_id, "auth request participant binding does not match declared needs");
        return Err(HttpError::bad_request("participant_binding_mismatch"));
    }
    let consent = browser_consent(&binding)?;
    let flow_id = format!(
        "flow_{}",
        digest_parts(&[
            "user_auth_request",
            input.signer_key_id(),
            input.request_id(),
        ])
    );
    let flow = AuthBrowserFlow {
        format: BROWSER_FLOW_FORMAT.to_owned(),
        flow_id: flow_id.clone(),
        kind: AuthBrowserFlowKind::UserAuth,
        state: AuthBrowserFlowState::ChooseProvider,
        request_id: request.request_id,
        request_digest,
        participant_id: request.participant_id,
        participant_artifact_digest: request.participant_artifact_digest,
        participant_needs_digest: request.participant_needs_digest,
        consent,
        session_public_key: request.session_public_key,
        session_nkey: request.session_nkey,
        portal_id: portal.portal_id.clone(),
        redirect_target: Some(request.redirect_target),
        principal_id: None,
        claim_owner: None,
        claimed_at: None,
        durable_result_digest: None,
        completed_at: None,
        created_at: now,
        expires_at: checked_add(now, FLOW_TTL_MS)?,
        version: 1,
    };
    match state.ephemeral.create_browser_flow(flow.clone()).await {
        Ok(()) => {}
        Err(AuthorizationStateError::StorageConflict) => {
            let existing = state
                .ephemeral
                .get_browser_flow(&flow_id)
                .await?
                .ok_or_else(|| HttpError::conflict("proof_replay"))?;
            if existing.request_digest != flow.request_digest
                || existing.session_public_key != flow.session_public_key
            {
                return Err(HttpError::conflict("proof_replay"));
            }
        }
        Err(error) => return Err(error.into()),
    }
    Ok(Json(AuthStartResponse {
        flow_id: flow_id.clone(),
        login_url: portal_url(&portal, &state.public_origin, &flow_id)?,
    }))
}

async fn select_login_portal(
    repository: &impl PortalRepository,
    participant_id: &str,
    redirect_target: &str,
) -> Result<(LoginPortalRecord, LoginSettingsRecord), HttpError> {
    let origin = canonical_origin(redirect_target)
        .map_err(|_| HttpError::bad_request("invalid_redirect_target"))?;
    for route in repository.list_portal_routes().await? {
        if route.deployment_id.is_some()
            || route
                .participant_id
                .as_deref()
                .is_some_and(|value| value != participant_id)
            || route.origin.as_deref().is_some_and(|value| value != origin)
        {
            continue;
        }
        if let Some((portal, settings)) = repository.get_login_portal(&route.portal_id).await? {
            if !portal.disabled && !portal.removed {
                return Ok((portal, settings));
            }
        }
    }
    repository
        .get_login_portal("builtin")
        .await?
        .filter(|(portal, _)| !portal.disabled && !portal.removed)
        .ok_or_else(|| HttpError::internal("builtin_portal_unavailable"))
}

pub(crate) async fn select_device_portal(
    repository: &impl PortalRepository,
    participant_id: &str,
    deployment_id: &str,
) -> Result<LoginPortalRecord, HttpError> {
    for route in repository.list_portal_routes().await? {
        if route.origin.is_some()
            || route
                .participant_id
                .as_deref()
                .is_some_and(|value| value != participant_id)
            || route
                .deployment_id
                .as_deref()
                .is_some_and(|value| value != deployment_id)
        {
            continue;
        }
        if let Some((portal, _)) = repository.get_login_portal(&route.portal_id).await? {
            if !portal.disabled && !portal.removed {
                return Ok(portal);
            }
        }
    }
    repository
        .get_login_portal("builtin")
        .await?
        .map(|(portal, _)| portal)
        .filter(|portal| !portal.disabled && !portal.removed)
        .ok_or_else(|| HttpError::internal("builtin_portal_unavailable"))
}

fn portal_url(
    portal: &LoginPortalRecord,
    public_origin: &str,
    flow_id: &str,
) -> Result<String, HttpError> {
    let entry = portal.entry_url.as_deref().map_or_else(
        || {
            format!(
                "{}/_trellis/portal/auth",
                public_origin.trim_end_matches('/')
            )
        },
        ToOwned::to_owned,
    );
    let mut url = Url::parse(&entry).map_err(|_| HttpError::internal("portal_entry_invalid"))?;
    url.query_pairs_mut().append_pair("flowId", flow_id);
    Ok(url.into())
}

pub(crate) async fn portal_index<R, E>(State(state): State<AuthHttpState<R, E>>) -> Response
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
    portal_file(&state, "200.html").await
}

pub(crate) async fn portal_page<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(path): Path<String>,
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
    let direct = format!("_trellis/portal/{path}");
    let response = portal_file(&state, &direct).await;
    if response.status() == StatusCode::NOT_FOUND {
        portal_file(&state, "200.html").await
    } else {
        response
    }
}

pub(crate) async fn portal_asset<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(path): Path<String>,
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
    portal_file(&state, &format!("_trellis/assets/{path}")).await
}

async fn portal_file<R, E>(state: &AuthHttpState<R, E>, path: &str) -> Response {
    if std::path::Path::new(path)
        .components()
        .any(|component| !matches!(component, Component::Normal(_)))
    {
        return StatusCode::NOT_FOUND.into_response();
    }
    let bytes = if let Some(directory) = &state.portal_override_dir {
        match tokio::fs::read(directory.join(path)).await {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        }
    } else {
        EMBEDDED_PORTAL_ASSETS
            .iter()
            .find_map(|(asset_path, bytes)| (*asset_path == path).then(|| bytes.to_vec()))
    };
    let Some(bytes) = bytes else {
        return StatusCode::NOT_FOUND.into_response();
    };
    let content_type = match path.rsplit_once('.').map(|(_, extension)| extension) {
        Some("css") => "text/css; charset=utf-8",
        Some("html") => "text/html; charset=utf-8",
        Some("ico") => "image/x-icon",
        Some("js") => "text/javascript; charset=utf-8",
        Some("json") => "application/json; charset=utf-8",
        Some("png") => "image/png",
        Some("svg") => "image/svg+xml",
        Some("txt") => "text/plain; charset=utf-8",
        Some("webp") => "image/webp",
        _ => "application/octet-stream",
    };
    ([(CONTENT_TYPE, content_type)], bytes).into_response()
}

#[cfg(test)]
mod tests {
    #[test]
    fn start_response_has_one_final_shape() {
        assert_eq!(
            serde_json::to_value(super::AuthStartResponse {
                flow_id: "flow_01".to_owned(),
                login_url: "https://auth.example/login?flowId=flow_01".to_owned(),
            })
            .unwrap(),
            serde_json::json!({
                "flowId": "flow_01",
                "loginUrl": "https://auth.example/login?flowId=flow_01",
            })
        );
    }
}
