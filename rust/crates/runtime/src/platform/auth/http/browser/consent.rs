use super::super::*;
use super::local::BrowserFlowResponse;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApprovalRequest {
    approved: bool,
    consent_view_digest: String,
    #[serde(default)]
    pub(crate) selected_optional_bundles: Vec<String>,
    idempotency_key: String,
}

pub(crate) async fn decide_approval<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ApprovalRequest>,
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
    let (portal, _) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if flow.state != AuthBrowserFlowState::ApprovalRequired {
        return Err(HttpError::conflict("flow_not_awaiting_approval"));
    }
    let now = now_ms()?;
    let binding = state
        .service
        .repository()
        .get_participant_binding(&flow.participant_id, &flow.participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::internal("participant_binding_missing"))?;
    let current_consent = browser_consent(&binding)?;
    if request.consent_view_digest != flow.consent.consent_view_digest
        || current_consent != flow.consent
    {
        return Err(HttpError::conflict("consent_view_changed"));
    }
    if !request.approved {
        let expected = flow.version;
        flow.state = AuthBrowserFlowState::ApprovalDenied;
        flow.completed_at = Some(now);
        flow.version += 1;
        state
            .ephemeral
            .replace_browser_flow(expected, flow.clone())
            .await?;
        return Ok(Json(flow_response(flow)));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let current = state
        .service
        .repository()
        .get_identity_authority(&principal_id, &flow.participant_id)
        .await?;
    let (grant_set, capabilities, selected_optional_bundles) =
        select_browser_authority(&flow.consent, &request.selected_optional_bundles)?;
    let requests_reserved = capabilities
        .iter()
        .any(|capability| matches!(capability.as_str(), "admin" | "provision" | "activate"));
    if requests_reserved
        && (flow.participant_artifact_digest != administration_participant_digest()?
            || current.as_ref().is_none_or(|authority| {
                capabilities
                    .iter()
                    .any(|capability| !authority.desired_capabilities.contains(capability))
            }))
    {
        return Err(HttpError::forbidden("reserved_capability"));
    }
    let authority_id = current.as_ref().map_or_else(
        || {
            format!(
                "ida_{}",
                digest_parts(&[&principal_id, &flow.participant_id])
            )
        },
        |authority| authority.authority_id.clone(),
    );
    let request_value =
        serde_json::to_value(&request).map_err(|_| HttpError::bad_request("invalid_approval"))?;
    let request_digest = trellis_protocol::digest_json(&request_value)
        .map_err(|_| HttpError::bad_request("invalid_approval"))?;
    let signer_id = super::super::super::domain::validate_ed25519_public_key(
        "sessionPublicKey",
        &flow.session_public_key,
    )?;
    let proposal = state
        .service
        .create_authority_proposal(CreateAuthorityProposalInput {
            authority_kind: AuthorityKind::Identity,
            authority_id: authority_id.clone(),
            deployment_id: None,
            proposal_kind: if current.is_some() {
                AuthorityProposalKind::Update
            } else {
                AuthorityProposalKind::Initial
            },
            participant_id: flow.participant_id.clone(),
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
            grant_set: grant_set.clone(),
            capabilities: capabilities.clone(),
            base_authority_version: current.as_ref().map(|authority| authority.version),
            payload: json!({
                "source": "browser_approval",
                "flowId": flow_id,
                "consentViewDigest": flow.consent.consent_view_digest,
                "proposalDigest": flow.consent.proposal_digest,
                "selectedOptionalBundles": selected_optional_bundles,
                "baseAuthorityVersion": current.as_ref().map(|authority| authority.version),
            }),
            created_at: now,
            expires_at: current.as_ref().and_then(|authority| authority.expires_at),
            idempotency: idempotency(
                &flow_id,
                "browser.authority.propose",
                &signer_id,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let proposal_id = match proposal {
        IdempotentOutcome::Applied(proposal) => proposal.proposal_id,
        IdempotentOutcome::Replayed(value) => value
            .get("proposalId")
            .and_then(Value::as_str)
            .ok_or_else(|| HttpError::internal("invalid_proposal_replay"))?
            .to_owned(),
    };
    let (proposal, _) = state
        .service
        .repository()
        .get_authority_proposal(&proposal_id)
        .await?
        .ok_or_else(|| HttpError::internal("proposal_missing"))?;
    let desired = DesiredAuthorityRecord::Identity(IdentityAuthorityRecord {
        authority_id: authority_id.clone(),
        principal_id,
        participant_id: flow.participant_id.clone(),
        participant_artifact_digest: flow.participant_artifact_digest.clone(),
        accepted_needs_digest: flow.participant_needs_digest.clone(),
        desired_grant_set: grant_set,
        desired_capabilities: capabilities,
        state: AuthorityState::Accepted,
        version: current
            .as_ref()
            .map_or(1, |authority| authority.version + 1),
        created_at: current
            .as_ref()
            .map_or(now, |authority| authority.created_at),
        updated_at: now,
        expires_at: current.as_ref().and_then(|authority| authority.expires_at),
        decision: Some(AuthorityDecision {
            decided_at: now,
            decided_by: flow.session_public_key.clone(),
            reason: None,
        }),
    });
    let durable_result_digest = trellis_protocol::digest_json(
        &serde_json::to_value(&desired).map_err(|_| HttpError::internal("authority_encode"))?,
    )
    .map_err(|_| HttpError::internal("authority_digest"))?;
    state
        .service
        .decide_authority_proposal(DecideAuthorityProposalInput {
            proposal_id,
            expected_version: proposal.version,
            expected_base_authority_version: None,
            outcome: AuthorityDecisionOutcome::Accepted,
            decided_by: flow.session_public_key.clone(),
            reason: None,
            desired_authority: Some(desired),
            decided_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.authority.accept",
                &signer_id,
                &request.idempotency_key,
                &request_digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let authority_target =
        AuthorityTarget::new(AuthorityKind::Identity, authority_id).map_err(HttpError::from)?;
    super::super::super::ensure_authority_dependencies(
        state.service.repository(),
        AuthorityEvidenceScope {
            target: authority_target.clone(),
            participant_id: flow.participant_id.clone(),
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
        },
        &binding,
        now,
    )
    .await?;
    state
        .service
        .authorization()
        .reconcile_authority(&authority_target, now)
        .await?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Approved;
    flow.durable_result_digest = Some(durable_result_digest);
    flow.completed_at = Some(now);
    flow.version += 1;
    state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await?;
    Ok(Json(flow_response(flow)))
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindRequest {
    idempotency_key: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BindResponse {
    server_now: i64,
    session: SessionRecord,
    nats: NatsBootstrapResponse,
    authorization_context: super::super::super::AuthorizationContextBundle,
    redirect_target: Option<String>,
}

pub(crate) async fn bind_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<BindRequest>,
) -> Result<Json<BindResponse>, HttpError>
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
    let (portal, _) = state
        .service
        .repository()
        .get_login_portal(&flow.portal_id)
        .await?
        .ok_or_else(|| HttpError::gone("portal_unavailable"))?;
    require_selected_portal_origin(&headers, &portal, &state.public_origin)?;
    if !matches!(
        flow.state,
        AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
    ) {
        return Err(HttpError::conflict("flow_not_approved"));
    }
    let binding = state
        .service
        .repository()
        .get_participant_binding(&flow.participant_id, &flow.participant_artifact_digest)
        .await?
        .ok_or_else(|| HttpError::conflict("participant_unavailable"))?;
    if binding.needs_digest != flow.participant_needs_digest {
        return Err(HttpError::conflict("participant_needs_changed"));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let now = now_ms()?;
    let signer_id = super::super::super::domain::validate_ed25519_public_key(
        "sessionPublicKey",
        &flow.session_public_key,
    )?;
    let digest = digest_parts(&[&flow_id, &request.idempotency_key]);
    let outcome = state
        .service
        .create_session(CreateSessionInput {
            principal_id,
            principal_kind: PrincipalKind::User,
            participant_id: flow.participant_id.clone(),
            participant_kind: binding.participant_kind,
            participant_artifact_digest: flow.participant_artifact_digest.clone(),
            participant_needs_digest: flow.participant_needs_digest.clone(),
            session_public_key: flow.session_public_key.clone(),
            deployment_id: None,
            instance_id: None,
            desired_authority: None,
            created_at: now,
            idempotency: idempotency(
                &flow_id,
                "browser.session.bind",
                &signer_id,
                &request.idempotency_key,
                &digest,
                now,
            )?,
            actions: Vec::new(),
        })
        .await?;
    let session = match outcome {
        IdempotentOutcome::Applied(session) => session,
        IdempotentOutcome::Replayed(value) => {
            let session_id = value
                .get("sessionId")
                .and_then(Value::as_str)
                .ok_or_else(|| HttpError::internal("invalid_session_replay"))?;
            state
                .service
                .repository()
                .get_session(session_id)
                .await?
                .ok_or_else(|| HttpError::internal("session_missing"))?
        }
    };
    if flow.state == AuthBrowserFlowState::Approved {
        let expected = flow.version;
        flow.state = AuthBrowserFlowState::Consumed;
        flow.claim_owner = Some(session.session_id.clone());
        flow.claimed_at = Some(now);
        flow.version += 1;
        state
            .ephemeral
            .replace_browser_flow(expected, flow.clone())
            .await?;
    }
    let issuance = state
        .service
        .authorization()
        .resolve_issuable_state(&session.session_id, now)
        .await
        .map_err(map_issuance_error)?;
    let expires_at = [
        issuance.session_expires_at,
        issuance.effective_authority_expires_at,
    ]
    .into_iter()
    .flatten()
    .min()
    .ok_or_else(|| HttpError::internal("credential_expiry_missing"))?;
    let route =
        state
            .issuer
            .deny_all_user_jwt(&flow.session_nkey, expires_at / 1_000, now / 1_000)?;
    let authorization_context = state
        .authorization_contexts
        .issue(
            super::super::super::AuthorizationContextIssueRequest {
                session_id: session.session_id.clone(),
                request_id: request.idempotency_key,
                request_digest: digest,
            },
            now / 1_000,
        )
        .await
        .map_err(map_issuance_error)?;
    Ok(Json(BindResponse {
        server_now: now,
        session,
        nats: NatsBootstrapResponse {
            jwt: route.jwt,
            jwt_expires_at: route.expires_at,
            servers: state.websocket_nats_servers,
        },
        authorization_context,
        redirect_target: flow.redirect_target,
    }))
}
