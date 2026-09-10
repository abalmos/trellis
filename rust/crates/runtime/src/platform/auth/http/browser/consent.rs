use super::super::*;
use super::local::{portal_flow_response, PortalFlowResponse};
use crate::platform::auth::policy::portal_allows_authenticated_provider;

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ApprovalRequest {
    approved: bool,
    consent_view_digest: String,
    pub(crate) selected_optional_bundles: Vec<String>,
}

pub(crate) async fn decide_approval<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(request): Json<ApprovalRequest>,
) -> Result<Json<PortalFlowResponse>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    require_portal_binding(&flow, &headers)?;
    let request_value =
        serde_json::to_value(&request).map_err(|_| HttpError::bad_request("invalid_approval"))?;
    let request_digest = trellis_protocol::digest_json(&request_value)
        .map_err(|_| HttpError::bad_request("invalid_approval"))?;
    if matches!(
        flow.state,
        AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
    ) {
        let signer_id = super::super::super::domain::validate_ed25519_public_key(
            "sessionPublicKey",
            &flow.session_public_key,
        )?;
        let recorded = state
            .service
            .repository()
            .get_idempotency_result("browser.grant.accept", &signer_id, &flow_id)
            .await?;
        if !request.approved
            || request.consent_view_digest != flow.consent.consent_view_digest
            || recorded.as_ref().map(|record| &record.request_digest) != Some(&request_digest)
        {
            return Err(HttpError::conflict("approval_replay_mismatch"));
        }
        return Ok(Json(portal_flow_response(&state, flow).await?));
    }
    if flow.state != AuthBrowserFlowState::ApprovalRequired {
        return Err(HttpError::conflict("flow_not_awaiting_approval"));
    }
    let now = now_ms()?;
    let (_, binding) = state
        .service
        .repository()
        .get_installed_participant_record(
            flow.participant_id.clone(),
            Some(flow.installed_revision),
        )
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
        return Ok(Json(portal_flow_response(&state, flow).await?));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let (grant_set, _, _) =
        select_browser_authority(&flow.consent, &request.selected_optional_bundles)?;
    let current = state
        .service
        .repository()
        .get_grant_binding(
            GrantOwnerKind::User,
            principal_id.clone(),
            flow.participant_id.clone(),
        )
        .await?;
    let platform_privileges = current
        .as_ref()
        .filter(|binding| {
            binding.state == GrantBindingState::Active
                && binding.expires_at.is_none_or(|expires_at| expires_at > now)
        })
        .map_or_else(Vec::new, |binding| binding.platform_privileges.clone());
    let signer_id = super::super::super::domain::validate_ed25519_public_key(
        "sessionPublicKey",
        &flow.session_public_key,
    )?;
    let durable = state
        .service
        .repository()
        .set_grant_binding(
            GrantBindingReplacement {
                owner_kind: GrantOwnerKind::User,
                owner_id: principal_id.clone(),
                participant_id: flow.participant_id.clone(),
                installed_revision: flow.installed_revision,
                grants: grant_set,
                platform_privileges,
                expected_revision: flow.target_grant_revision,
                expected_current_installed_revision: Some(flow.installed_revision),
                state: GrantBindingState::Active,
                expires_at: None,
                provenance: None,
            },
            idempotency(
                &flow_id,
                "browser.grant.accept",
                &signer_id,
                &flow_id,
                &request_digest,
                now,
            )?,
        )
        .await
        .map_err(|error| match error {
            AuthorizationStateError::StorageConflict => HttpError::conflict("authority_changed"),
            error => error.into(),
        })?;
    let durable_result_digest = trellis_protocol::digest_json(&durable)
        .map_err(|_| HttpError::internal("authority_digest"))?;
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Approved;
    flow.durable_result_digest = Some(durable_result_digest);
    flow.completed_at = Some(now);
    flow.version += 1;
    if let Err(error) = state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await
    {
        if error != AuthorizationStateError::StorageConflict {
            return Err(error.into());
        }
        let current = load_flow(&state.ephemeral, &flow.flow_id).await?;
        if !matches!(
            current.state,
            AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
        ) || current.durable_result_digest != flow.durable_result_digest
        {
            return Err(HttpError::conflict("approval_completion_conflict"));
        }
        flow = current;
    }
    Ok(Json(portal_flow_response(&state, flow).await?))
}

pub(super) async fn apply_trusted_portal_authority<R, E>(
    state: &AuthHttpState<R, E>,
    mut flow: AuthBrowserFlow,
    attributes: ProviderLoginAttributes,
    now: i64,
) -> Result<Option<AuthBrowserFlow>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    let (_, binding) = state
        .service
        .repository()
        .get_installed_participant_record(
            flow.participant_id.clone(),
            Some(flow.installed_revision),
        )
        .await?
        .ok_or_else(|| HttpError::internal("participant_binding_missing"))?;
    let consent = browser_consent(&binding)?;
    if consent != flow.consent {
        return Err(HttpError::conflict("consent_view_changed"));
    }
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let signer_id = super::super::super::domain::validate_ed25519_public_key(
        "sessionPublicKey",
        &flow.session_public_key,
    )?;
    let durable = 'policy: {
        for attempt in 0..3 {
            let Some((portal, settings)) = state
                .service
                .repository()
                .get_login_portal(&flow.portal_id)
                .await?
            else {
                return Err(HttpError::gone("portal_unavailable"));
            };
            if portal.removed {
                return Err(HttpError::gone("portal_unavailable"));
            }
            if !portal_allows_authenticated_provider(&portal, &settings, &attributes.provider_id) {
                return Err(HttpError::forbidden(if portal.disabled {
                    "portal_disabled"
                } else if attributes.provider_id == "local" && !settings.local_login_enabled {
                    "local_login_disabled"
                } else {
                    "provider_not_allowed"
                }));
            }
            if attributes.provider_id != "local"
                && !state.oidc_providers.contains_key(&attributes.provider_id)
            {
                return Err(HttpError::not_found("provider_not_found"));
            }
            let Some(policy) = state
                .service
                .repository()
                .get_portal_grant_override(&flow.portal_id, &flow.participant_id)
                .await?
            else {
                return Ok(None);
            };
            let groups = state
                .service
                .repository()
                .list_capability_groups()
                .await?
                .into_iter()
                .map(|group| (group.group_key.clone(), group))
                .collect();
            let snapshot = portal_policy_snapshot(
                &portal,
                &settings,
                &flow.participant_id,
                Some(&policy),
                &groups,
            )?;
            let selection =
                resolve_portal_authority_selection(&policy, &groups, &consent, &attributes)?;
            let request_digest = trellis_protocol::digest_json(&json!({
                "flowId": flow.flow_id,
                "portalId": flow.portal_id,
                "portalVersion": snapshot.portal_version,
                "loginSettingsVersion": snapshot.login_settings_version,
                "participantId": flow.participant_id,
                "policyVersion": snapshot.policy_version,
                "capabilityGroupVersions": snapshot.capability_group_versions,
                "providerId": attributes.provider_id,
                "roles": attributes.roles,
                "effectivePolicyDigest": selection.effective_policy_digest,
            }))
            .map_err(|_| HttpError::internal("portal_policy_digest"))?;
            let result = state
                .service
                .repository()
                .set_portal_grant_binding(
                    GrantBindingReplacement {
                        owner_kind: GrantOwnerKind::User,
                        owner_id: principal_id.clone(),
                        participant_id: flow.participant_id.clone(),
                        installed_revision: flow.installed_revision,
                        grants: selection.grant_set,
                        platform_privileges: selection.platform_privileges,
                        expected_revision: flow.target_grant_revision,
                        expected_current_installed_revision: Some(flow.installed_revision),
                        state: GrantBindingState::Active,
                        expires_at: None,
                        provenance: Some(PortalGrantProvenance {
                            portal_id: flow.portal_id.clone(),
                            provider_id: attributes.provider_id.clone(),
                            roles: attributes.roles.clone(),
                            effective_policy_digest: selection.effective_policy_digest.clone(),
                        }),
                    },
                    snapshot,
                    idempotency(
                        &flow.flow_id,
                        "portal.grant.accept",
                        &signer_id,
                        &flow.flow_id,
                        &request_digest,
                        now,
                    )?,
                )
                .await;
            match result {
                Ok(durable) => break 'policy durable,
                Err(AuthorizationStateError::PortalPolicyChanged) if attempt < 2 => continue,
                Err(AuthorizationStateError::PortalPolicyChanged) => {
                    return Err(HttpError::conflict("portal_policy_changed"));
                }
                Err(AuthorizationStateError::StorageConflict) if attempt < 2 => {
                    continue;
                }
                Err(error) => return Err(error.into()),
            }
        }
        unreachable!("bounded portal policy retries return or break")
    };
    flow.state = AuthBrowserFlowState::Approved;
    flow.durable_result_digest = Some(
        trellis_protocol::digest_json(&durable)
            .map_err(|_| HttpError::internal("authority_digest"))?,
    );
    flow.completed_at = Some(now);
    Ok(Some(flow))
}

pub(super) async fn complete_authenticated_flow<R, E>(
    state: &AuthHttpState<R, E>,
    mut flow: AuthBrowserFlow,
    principal_id: String,
    attributes: ProviderLoginAttributes,
    portal_binding_digest: String,
    require_explicit_approval: bool,
    now: i64,
) -> Result<AuthBrowserFlow, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    if flow.state == AuthBrowserFlowState::ChooseProvider {
        flow.target_grant_revision = state
            .service
            .repository()
            .get_grant_binding(
                GrantOwnerKind::User,
                principal_id.clone(),
                flow.participant_id.clone(),
            )
            .await?
            .map_or(0, |binding| binding.revision);
        let expected = flow.version;
        flow.state = AuthBrowserFlowState::Authenticated;
        flow.principal_id = Some(principal_id.clone());
        flow.authenticated_provider_id = Some(attributes.provider_id.clone());
        flow.authenticated_roles = attributes.roles.clone();
        flow.portal_binding_digest = Some(portal_binding_digest.clone());
        flow.version += 1;
        match state
            .ephemeral
            .replace_browser_flow(expected, flow.clone())
            .await
        {
            Ok(()) => {}
            Err(AuthorizationStateError::StorageConflict) => {
                flow = load_flow(&state.ephemeral, &flow.flow_id).await?;
            }
            Err(error) => return Err(error.into()),
        }
    }
    if flow.principal_id.as_deref() != Some(&principal_id)
        || flow.authenticated_provider_id.as_deref() != Some(&attributes.provider_id)
        || flow.authenticated_roles != attributes.roles
        || flow.portal_binding_digest.as_deref() != Some(&portal_binding_digest)
    {
        return Err(HttpError::conflict("flow_authentication_conflict"));
    }
    if matches!(
        flow.state,
        AuthBrowserFlowState::ApprovalRequired
            | AuthBrowserFlowState::Approved
            | AuthBrowserFlowState::Consumed
    ) {
        return Ok(flow);
    }
    if flow.state != AuthBrowserFlowState::Authenticated {
        return Err(HttpError::conflict("flow_not_pending"));
    }
    let expected = flow.version;
    let allow_automatic_approval = automatic_approval_allowed(require_explicit_approval);
    // A portal policy governs this login even when an accepted authority
    // exists, so reconcile through policy instead of the fast path.
    let policy_governs = allow_automatic_approval
        && state
            .service
            .repository()
            .get_portal_grant_override(&flow.portal_id, &flow.participant_id)
            .await?
            .is_some();
    let existing_binding = if !allow_automatic_approval || policy_governs {
        None
    } else {
        state
            .service
            .repository()
            .get_grant_binding(
                GrantOwnerKind::User,
                principal_id.clone(),
                flow.participant_id.clone(),
            )
            .await?
            .filter(|binding| {
                binding.state == GrantBindingState::Active
                    && binding.expires_at.is_none_or(|expires_at| expires_at > now)
            })
    };
    let mut completed = if let Some(binding) = existing_binding {
        flow.state = AuthBrowserFlowState::Approved;
        flow.durable_result_digest = Some(
            trellis_protocol::digest_json(
                &serde_json::to_value(binding)
                    .map_err(|_| HttpError::internal("binding_encode"))?,
            )
            .map_err(|_| HttpError::internal("authority_digest"))?,
        );
        flow.completed_at = Some(now);
        flow
    } else if !allow_automatic_approval {
        flow.state = AuthBrowserFlowState::ApprovalRequired;
        flow
    } else if let Some(approved) =
        apply_trusted_portal_authority(state, flow.clone(), attributes, now).await?
    {
        approved
    } else {
        flow.state = AuthBrowserFlowState::ApprovalRequired;
        flow
    };
    completed.version += 1;
    match state
        .ephemeral
        .replace_browser_flow(expected, completed.clone())
        .await
    {
        Ok(()) => Ok(completed),
        Err(AuthorizationStateError::StorageConflict) => {
            let current = load_flow(&state.ephemeral, &completed.flow_id).await?;
            let converged = current.principal_id == completed.principal_id
                && (current.state == completed.state
                    || matches!(
                        (completed.state, current.state),
                        (
                            AuthBrowserFlowState::ApprovalRequired,
                            AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
                        ) | (
                            AuthBrowserFlowState::Approved,
                            AuthBrowserFlowState::Consumed
                        )
                    ))
                && completed
                    .durable_result_digest
                    .as_ref()
                    .is_none_or(|digest| current.durable_result_digest.as_ref() == Some(digest));
            if converged {
                Ok(current)
            } else {
                Err(HttpError::conflict("flow_completion_conflict"))
            }
        }
        Err(error) => Err(error.into()),
    }
}

fn automatic_approval_allowed(require_explicit_approval: bool) -> bool {
    !require_explicit_approval
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::auth::{builtins, sqlite::SqliteAuthorizationStore};
    use trellis_protocol::PlatformPrivilege;

    #[test]
    fn administrator_account_continuation_requires_explicit_approval() {
        assert!(!super::automatic_approval_allowed(true));
        assert!(super::automatic_approval_allowed(false));
    }

    #[tokio::test]
    async fn approval_is_fenced_by_installed_and_grant_revisions(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let store = SqliteAuthorizationStore::open_in_memory()?;
        let now = 1_700_000_000_000;
        let actor =
            crate::platform::auth::tests::conformance::fixtures::install_login_mutation_actor(
                &store, now,
            )
            .await?;
        let principal_id = actor.principal_id.clone();
        let participant_v1 = builtins::console_participant_binding(now)?;
        let participant_id = participant_v1.participant_id.clone();
        assert_eq!(
            store
                .put_participant_binding(participant_v1.clone())
                .await?,
            1
        );
        let consent = browser_consent(&participant_v1).expect("built-in consent is valid");
        let (grants, _, _) =
            select_browser_authority(&consent, &[]).expect("required authority is valid");
        let approval = GrantBindingReplacement {
            owner_kind: GrantOwnerKind::User,
            owner_id: principal_id.clone(),
            participant_id: participant_id.clone(),
            installed_revision: 1,
            grants: grants.clone(),
            platform_privileges: Vec::new(),
            state: GrantBindingState::Active,
            expires_at: None,
            provenance: None,
            expected_revision: 0,
            expected_current_installed_revision: Some(1),
        };
        let approval_idempotency = idempotency(
            "flow-r1",
            "browser.grant.accept",
            "browser-signer",
            "flow-r1",
            &digest_parts(&["approval-r1"]),
            now,
        )
        .expect("test idempotency is valid");
        let approved = store
            .set_grant_binding(approval.clone(), approval_idempotency.clone())
            .await?;
        assert_eq!(
            store
                .set_grant_binding(approval, approval_idempotency)
                .await?,
            approved
        );
        assert_eq!(
            store.list_ready_post_commit_actions(now, 10).await?.len(),
            1
        );

        let mut participant_v2 = participant_v1;
        participant_v2.projection.display_name = "Trellis Console R2".to_owned();
        participant_v2.needs_digest = trellis_protocol::digest_json(&participant_v2.projection)?;
        participant_v2.resolved_at = now + 1;
        assert_eq!(store.put_participant_binding(participant_v2).await?, 2);

        let stale_approval = GrantBindingReplacement {
            owner_kind: GrantOwnerKind::User,
            owner_id: principal_id.clone(),
            participant_id: participant_id.clone(),
            installed_revision: 1,
            grants: grants.clone(),
            platform_privileges: Vec::new(),
            state: GrantBindingState::Active,
            expires_at: None,
            provenance: None,
            expected_revision: 1,
            expected_current_installed_revision: Some(1),
        };
        assert_eq!(
            store
                .set_grant_binding(
                    stale_approval,
                    idempotency(
                        "flow-stale-install",
                        "browser.grant.accept",
                        "browser-signer",
                        "flow-stale-install",
                        &digest_parts(&["approval-stale-install"]),
                        now + 2,
                    )
                    .expect("test idempotency is valid"),
                )
                .await,
            Err(AuthorizationStateError::RevisionConflict {
                expected: 1,
                current: 2,
            })
        );
        let unchanged = store
            .get_grant_binding(
                GrantOwnerKind::User,
                principal_id.clone(),
                participant_id.clone(),
            )
            .await?
            .unwrap();
        assert_eq!(unchanged.revision, 1);
        assert_eq!(unchanged.installed_revision, 1);
        assert_eq!(unchanged.grants, grants);
        assert_eq!(
            store
                .list_ready_post_commit_actions(now + 2, 10)
                .await?
                .len(),
            1
        );

        let fresh_approval = GrantBindingReplacement {
            owner_kind: GrantOwnerKind::User,
            owner_id: principal_id.clone(),
            participant_id: participant_id.clone(),
            installed_revision: 2,
            grants: grants.clone(),
            platform_privileges: Vec::new(),
            state: GrantBindingState::Active,
            expires_at: None,
            provenance: None,
            expected_revision: 1,
            expected_current_installed_revision: Some(2),
        };
        store
            .set_grant_binding(
                fresh_approval,
                idempotency(
                    "flow-r2",
                    "browser.grant.accept",
                    "browser-signer",
                    "flow-r2",
                    &digest_parts(&["approval-r2"]),
                    now + 3,
                )
                .expect("test idempotency is valid"),
            )
            .await?;

        store
            .admin_set_grant_binding(
                actor,
                GrantBindingReplacement {
                    owner_kind: GrantOwnerKind::User,
                    owner_id: principal_id.clone(),
                    participant_id: participant_id.clone(),
                    installed_revision: 2,
                    grants: grants.clone(),
                    platform_privileges: vec![PlatformPrivilege::Admin],
                    state: GrantBindingState::Active,
                    expires_at: None,
                    provenance: None,
                    expected_revision: 2,
                    expected_current_installed_revision: None,
                },
                idempotency(
                    "admin-replacement",
                    "Auth.Grants.Set",
                    "admin-signer",
                    "admin-replacement",
                    &digest_parts(&["admin-replacement"]),
                    now + 4,
                )
                .expect("test idempotency is valid"),
            )
            .await?;
        assert!(matches!(
            store
                .set_grant_binding(
                    GrantBindingReplacement {
                        owner_kind: GrantOwnerKind::User,
                        owner_id: principal_id,
                        participant_id,
                        installed_revision: 2,
                        grants,
                        platform_privileges: Vec::new(),
                        state: GrantBindingState::Active,
                        expires_at: None,
                        provenance: None,
                        expected_revision: 2,
                        expected_current_installed_revision: Some(2),
                    },
                    idempotency(
                        "flow-stale-binding",
                        "browser.grant.accept",
                        "browser-signer",
                        "flow-stale-binding",
                        &digest_parts(&["approval-stale-binding"]),
                        now + 5,
                    )
                    .expect("test idempotency is valid"),
                )
                .await,
            Err(AuthorizationStateError::RevisionConflict {
                expected: 2,
                current: 3,
            })
        ));
        assert_eq!(
            store
                .list_ready_post_commit_actions(now + 5, 10)
                .await?
                .len(),
            3
        );
        Ok(())
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BindRequest {
    request_id: String,
    #[serde(rename = "issuedAt")]
    _issued_at: i64,
    proof: Value,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BrowserSessionBundle {
    server_now: i64,
    session: BrowserLoginSession,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct BrowserLoginSession {
    session_id: String,
    principal_id: String,
    participant_id: String,
    session_key: String,
    expires_at: Option<i64>,
}

pub(crate) async fn bind_flow<R, E>(
    State(state): State<AuthHttpState<R, E>>,
    Path(flow_id): Path<String>,
    headers: HeaderMap,
    Json(raw): Json<Value>,
) -> Result<Json<BrowserSessionBundle>, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    let redirect_target = flow
        .redirect_target
        .as_deref()
        .ok_or_else(|| HttpError::bad_request("missing_redirect_target"))?;
    require_portal_origin(&headers, redirect_target)?;
    if !matches!(
        flow.state,
        AuthBrowserFlowState::Approved | AuthBrowserFlowState::Consumed
    ) {
        return Err(HttpError::conflict("flow_not_approved"));
    }
    let mut unsigned_request = raw.clone();
    unsigned_request
        .as_object_mut()
        .ok_or_else(|| HttpError::bad_request("invalid_bind_request"))?
        .remove("proof");
    let request: BindRequest =
        serde_json::from_value(raw).map_err(|_| HttpError::bad_request("invalid_bind_request"))?;
    if ulid::Ulid::from_string(&request.request_id)
        .map(|request_id| request_id.to_string() != request.request_id)
        .unwrap_or(true)
    {
        return Err(HttpError::bad_request("invalid_bind_request_id"));
    }
    let input =
        SessionProofInput::user_auth_bind(trellis_protocol::UserAuthBindSessionProofInput {
            origin: state.public_origin.clone(),
            flow_id,
            session_public_key: flow.session_public_key.clone(),
            unsigned_request,
        })
        .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    verify_session_proof(
        &input,
        &parse_session_proof(&request.proof)
            .map_err(|_| HttpError::unauthorized("invalid_proof"))?,
        &flow.session_public_key,
        now_ms()?,
        state.proof_policy,
    )
    .map_err(|_| HttpError::unauthorized("invalid_proof"))?;
    let flow = complete_flow(&state, flow, now_ms()?).await?;
    Ok(Json(session_bundle(&state, &flow).await?))
}

async fn complete_flow<R, E>(
    state: &AuthHttpState<R, E>,
    mut flow: AuthBrowserFlow,
    now: i64,
) -> Result<AuthBrowserFlow, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    if flow.state == AuthBrowserFlowState::Consumed {
        let session_id = flow
            .claim_owner
            .as_deref()
            .ok_or_else(|| HttpError::internal("flow_session_missing"))?;
        let session = state
            .service
            .repository()
            .get_session(session_id)
            .await?
            .ok_or_else(|| HttpError::internal("flow_session_missing"))?;
        if session.principal_id != flow.principal_id.as_deref().unwrap_or_default()
            || session.participant_id != flow.participant_id
            || session.session_public_key != flow.session_public_key
        {
            return Err(HttpError::internal("flow_session_mismatch"));
        }
        return Ok(flow);
    }
    if flow.state != AuthBrowserFlowState::Approved {
        return Err(HttpError::conflict("flow_not_approved"));
    }
    let (_, binding) = state
        .service
        .repository()
        .get_installed_participant_record(
            flow.participant_id.clone(),
            Some(flow.installed_revision),
        )
        .await?
        .ok_or_else(|| HttpError::conflict("participant_unavailable"))?;
    let principal_id = flow
        .principal_id
        .clone()
        .ok_or_else(|| HttpError::conflict("flow_has_no_principal"))?;
    let signer_id = super::super::super::domain::validate_ed25519_public_key(
        "sessionPublicKey",
        &flow.session_public_key,
    )?;
    let digest = digest_parts(&["browser.session.complete", &flow.flow_id]);
    let outcome = state
        .service
        .create_session(CreateSessionInput {
            principal_id,
            participant_id: flow.participant_id.clone(),
            participant_kind: binding.participant_kind,
            session_public_key: flow.session_public_key.clone(),
            created_at: now,
            idempotency: idempotency(
                &flow.flow_id,
                "browser.session.complete",
                &signer_id,
                &flow.flow_id,
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
    let expected = flow.version;
    flow.state = AuthBrowserFlowState::Consumed;
    flow.claim_owner = Some(session.session_id.clone());
    flow.claimed_at = Some(now);
    flow.version += 1;
    match state
        .ephemeral
        .replace_browser_flow(expected, flow.clone())
        .await
    {
        Ok(()) => Ok(flow),
        Err(AuthorizationStateError::StorageConflict) => {
            let current = load_flow(&state.ephemeral, &flow.flow_id).await?;
            if current.state == AuthBrowserFlowState::Consumed
                && current.claim_owner.as_deref() == Some(session.session_id.as_str())
            {
                Ok(current)
            } else {
                Err(HttpError::conflict("flow_completion_conflict"))
            }
        }
        Err(error) => Err(error.into()),
    }
}

async fn session_bundle<R, E>(
    state: &AuthHttpState<R, E>,
    flow: &AuthBrowserFlow,
) -> Result<BrowserSessionBundle, HttpError>
where
    R: AccountRepository
        + AuthorityEvidenceRepository
        + GrantRepository
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
    if flow.state != AuthBrowserFlowState::Consumed {
        return Err(HttpError::conflict("flow_not_consumed"));
    }
    let now = now_ms()?;
    let session = state
        .service
        .repository()
        .get_session(
            flow.claim_owner
                .as_deref()
                .ok_or_else(|| HttpError::internal("flow_session_missing"))?,
        )
        .await?
        .ok_or_else(|| HttpError::internal("flow_session_missing"))?;
    if session.state != crate::platform::auth::SessionState::Active
        || session
            .expires_at
            .is_some_and(|expires_at| expires_at <= now)
    {
        return Err(HttpError::unauthorized("session_inactive"));
    }
    Ok(BrowserSessionBundle {
        server_now: now,
        session: BrowserLoginSession {
            session_id: session.session_id,
            principal_id: session.principal_id,
            participant_id: session.participant_id,
            session_key: session.session_public_key,
            expires_at: session.expires_at,
        },
    })
}
