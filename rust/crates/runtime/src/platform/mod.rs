//! Platform subsystem scaffold.

use std::sync::Arc;

/// Rust-owned authorization state and materialization.
pub mod auth;
pub mod auth_callout;
mod auth_operation;
mod auth_post_commit;
mod auth_rpc;
pub mod bootstrap;
pub mod catalog;

use auth::{
    authorization_reconciliation_channel, AuthService, AuthServiceConfig, AuthorityDecision,
    AuthorityEvidenceScope, AuthorityKind, AuthorityState, AuthorityTarget,
    AuthorizationStateService, CreateSessionInput, DeploymentAuthorityRecord,
    DeploymentAuthorityRepository, DeploymentRecord, EvidenceRepository, FirstAdminAuthorityTarget,
    IdempotencyResultRecord, LoginPortalMutation, LoginPortalRecord, LoginPortalRepository,
    LoginSettingsRecord, ParticipantBindingRecord, ParticipantBindingRepository, PrincipalKind,
    PrincipalRecord, PrincipalRepository, PrincipalState, ResourceBindingEvidence,
    ResourceBindingState, ResourceProviderIdentity, RuntimeInstanceRecord, RuntimeInstanceState,
    SqliteAuthorizationStore,
};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use trellis_rs::client::SessionAuth;

use crate::shutdown::StopHandle;
use crate::supervisor::{RuntimeContext, RuntimeError, SubsystemHandle};
use crate::SubsystemName;
use auth_callout::AuthCallout;
use auth_operation::AuthOperationRuntime;
use auth_post_commit::AuthPostCommitRuntime;
use auth_rpc::AuthRpcRuntime;

pub(crate) async fn start(context: &RuntimeContext) -> Result<SubsystemHandle, RuntimeError> {
    let _owner = context.owner(crate::ownership::OwnerGroup::Platform)?;
    let auth_store = SqliteAuthorizationStore::open(context.stores.platform()?)?;
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .as_millis()
        .try_into()
        .map_err(|_| RuntimeError::Platform("current time exceeds i64 milliseconds".to_owned()))?;
    let authorization_config = context
        .config
        .resolve_authorization()
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let administration = auth::administration_participant_binding(now)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    auth_store
        .put_participant_binding(administration.clone())
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let auth_participant = auth::auth_runtime_participant_binding(now)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    auth_store
        .put_participant_binding(auth_participant.clone())
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    ensure_builtin_portal(&auth_store, now).await?;
    let authorization = AuthorizationStateService::new(auth_store.clone());
    authorization
        .reconcile_all(now)
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let (reconciliation, reconciliation_worker) =
        authorization_reconciliation_channel(authorization.clone(), 256);
    let ephemeral = auth::NatsAuthEphemeralRepository::ensure(context.trellis_nats.clone())
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let authorization_contexts = auth::AuthorizationContextService::start(
        Arc::new(auth_store.clone()),
        context.trellis_nats.clone(),
        authorization_config.clone(),
        now / 1_000,
    )
    .await
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let nats = context
        .config
        .resolve_nats_runtime()
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let callout = context
        .config
        .resolve_nats_auth_callout()
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let issuer = auth::NatsBootstrapIssuer::from_files(
        &callout.issuer_signing_seed_file,
        &nats.auth_creds_path,
        authorization_config.maximum_bootstrap_jwt_lifetime_seconds,
    )
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let auth_nats = async_nats::ConnectOptions::new()
        .subscription_capacity(256)
        .credentials_file(&nats.auth_creds_path)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?
        .connect(&nats.servers)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?;
    let system_nats = async_nats::ConnectOptions::new()
        .subscription_capacity(256)
        .credentials_file(&nats.system_creds_path)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?
        .connect(&nats.servers)
        .await
        .map_err(|error| RuntimeError::Nats(error.to_string()))?;
    let rpc_nats = context.trellis_nats.clone();
    let post_commit_nats = context.trellis_nats.clone();
    let rpc_system_nats = system_nats.clone();
    let post_commit_system_nats = system_nats.clone();
    let callout_runtime = AuthCallout::start(
        auth_nats,
        system_nats,
        authorization.clone(),
        auth_store.clone(),
        ephemeral.clone(),
        authorization_contexts.clone(),
        &callout.issuer_signing_seed_file,
        &callout.target_signing_seed_file,
        &callout.xkey_seed_file,
        &nats.auth_creds_path,
        &nats.trellis_creds_path,
        context
            .config
            .platform
            .as_ref()
            .and_then(|platform| platform.ttl_ms.as_ref())
            .and_then(|ttl| ttl.nats_jwt),
    )
    .await
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let auth_service = AuthService::new(auth_store.clone(), AuthServiceConfig::default())
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let (auth_event_session, auth_operation_session) =
        ensure_auth_event_session(&auth_service, &authorization, &auth_participant, now).await?;
    let auth_operation = AuthOperationRuntime::new(
        context.trellis_nats.clone(),
        auth_operation_session,
        auth_service.clone(),
    );
    let public_origin = context
        .config
        .http
        .as_ref()
        .and_then(|http| http.public_origin.clone())
        .unwrap_or_else(|| format!("http://localhost:{}", context.config.http_port()));
    let native_nats_servers = context
        .config
        .client
        .as_ref()
        .and_then(|client| client.nats_servers.clone())
        .unwrap_or_else(|| nats.servers.split(',').map(str::to_owned).collect());
    let websocket_nats_servers = context
        .config
        .client
        .as_ref()
        .and_then(|client| client.ws_nats_servers.clone())
        .unwrap_or_default();
    let auth_rpc = AuthRpcRuntime::start(
        rpc_nats,
        rpc_system_nats,
        auth_service.clone(),
        ephemeral.clone(),
        public_origin.clone(),
        native_nats_servers,
        websocket_nats_servers.clone(),
    )
    .await
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let auth_post_commit = AuthPostCommitRuntime::new(
        auth_service.repository().clone(),
        ephemeral.clone(),
        post_commit_nats,
        post_commit_system_nats,
        auth_event_session,
        authorization_contexts.clone(),
    );
    let bootstrap = if context.rotate_first_admin {
        auth_service
            .rotate_first_admin_flow(
                &public_origin,
                &FirstAdminAuthorityTarget {
                    participant_id: administration.participant_id.clone(),
                    participant_artifact_digest: administration.artifact_digest.clone(),
                    participant_needs_digest: administration.needs_digest.clone(),
                },
                now,
            )
            .await
    } else {
        auth_service
            .ensure_first_admin_flow(
                &public_origin,
                &FirstAdminAuthorityTarget {
                    participant_id: administration.participant_id,
                    participant_artifact_digest: administration.artifact_digest,
                    participant_needs_digest: administration.needs_digest,
                },
                now,
            )
            .await
    }
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    if let Some(bootstrap) = bootstrap {
        if let Some(bootstrap_url) = bootstrap.bootstrap_url {
            tracing::warn!(
                bootstrapUrl = %bootstrap_url,
                expiresAt = bootstrap.expires_at,
                "first administrator bootstrap required"
            );
        } else {
            tracing::warn!(
                status = "pending",
                flowIdHash = %bootstrap.flow_id_hash,
                expiresAt = bootstrap.expires_at,
                "first administrator bootstrap already pending"
            );
        }
    }
    let stop = StopHandle::new();
    let validator_stop = stop.clone();
    let validator_contexts = authorization_contexts.clone();
    let mut validator_join =
        tokio::spawn(async move { validator_contexts.run_validator_cache(validator_stop).await });
    tokio::select! {
        result = authorization_contexts.wait_for_validator_cache() => {
            if let Err(error) = result {
                stop.stop();
                validator_join.abort();
                return Err(error);
            }
        }
        result = &mut validator_join => {
            return match result {
                Ok(result) => result.and_then(|_| Err(RuntimeError::Platform(
                    "authorization validator cache exited during startup".to_owned(),
                ))),
                Err(error) => Err(RuntimeError::Platform(format!(
                    "authorization validator cache task failed: {error}"
                ))),
            };
        }
    }
    let http = context.config.http.as_ref();
    let oidc_providers =
        match auth::discover_oidc_providers(context.config.oauth.as_ref(), &public_origin).await {
            Ok(providers) => providers,
            Err(error) => {
                stop.stop();
                validator_join.abort();
                return Err(RuntimeError::Platform(error.to_string()));
            }
        };
    let router = match auth::auth_http_router(auth::AuthHttpOptions {
        service: auth_service,
        ephemeral,
        issuer,
        authorization_contexts: authorization_contexts.clone(),
        public_origin,
        allowed_origins: http
            .and_then(|http| http.origins.clone())
            .unwrap_or_default(),
        websocket_nats_servers,
        oidc_providers,
        rate_limit_max: http.and_then(|http| http.rate_limit_max).unwrap_or(100),
        rate_limit_window_ms: http
            .and_then(|http| http.rate_limit_window_ms)
            .unwrap_or(60_000),
        portal_override_dir: std::env::var_os("TRELLIS_BUILTIN_PORTAL_DIR").map(Into::into),
    }) {
        Ok(router) => router,
        Err(error) => {
            stop.stop();
            validator_join.abort();
            return Err(RuntimeError::Platform(error.to_string()));
        }
    };
    if let Err(error) = context.register_http_router(router) {
        stop.stop();
        validator_join.abort();
        return Err(error);
    }
    let task_stop = stop.clone();
    let join = tokio::spawn(async move {
        let _authorization = authorization;
        let _reconciliation = reconciliation;
        tokio::select! {
            result = reconciliation_worker.run(task_stop.clone()) => {
                result.map_err(|error| RuntimeError::Platform(error.to_string()))
            }
            result = callout_runtime.run(task_stop.clone()) => result,
            result = auth_rpc.run(task_stop.clone()) => result,
            result = auth_operation.run(task_stop.clone()) => result,
            result = auth_post_commit.run(task_stop.clone()) => result,
            result = authorization_contexts.clone().run_janitor(task_stop.clone()) => result,
            result = &mut validator_join => {
                match result {
                    Ok(result) => result,
                    Err(error) => Err(RuntimeError::Platform(format!(
                        "authorization validator cache task failed: {error}"
                    ))),
                }
            },
        }
    });

    Ok(SubsystemHandle {
        name: SubsystemName::Platform,
        stop,
        join,
    })
}

async fn ensure_auth_event_session(
    service: &AuthService<SqliteAuthorizationStore>,
    authorization: &AuthorizationStateService<SqliteAuthorizationStore>,
    participant: &ParticipantBindingRecord,
    now: i64,
) -> Result<(SessionAuth, SessionAuth), RuntimeError> {
    const PRINCIPAL_ID: &str = "svc_trellis_auth_runtime";
    const DEPLOYMENT_ID: &str = "dep_trellis_auth_runtime";
    const INSTANCE_ID: &str = "inst_trellis_auth_runtime";
    const AUTHORITY_ID: &str = "dpa_trellis_auth_runtime";

    if service
        .repository()
        .get_principal(PRINCIPAL_ID)
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .is_none()
    {
        service
            .repository()
            .create_principal(PrincipalRecord {
                principal_id: PRINCIPAL_ID.to_owned(),
                kind: PrincipalKind::Service,
                state: PrincipalState::Active,
                created_at: now,
                updated_at: now,
                version: 1,
                disabled_at: None,
                revoked_at: None,
            })
            .await
            .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    }
    service
        .repository()
        .put_deployment_evidence(DeploymentRecord {
            deployment_id: DEPLOYMENT_ID.to_owned(),
            participant_id: participant.participant_id.clone(),
            participant_kind: participant.participant_kind,
            active: true,
            expires_at: None,
        })
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    if service
        .repository()
        .get_runtime_instance(INSTANCE_ID)
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .is_none()
    {
        service
            .repository()
            .put_runtime_instance(RuntimeInstanceRecord {
                instance_id: INSTANCE_ID.to_owned(),
                deployment_id: DEPLOYMENT_ID.to_owned(),
                principal_id: PRINCIPAL_ID.to_owned(),
                state: RuntimeInstanceState::Active,
                created_at: now,
                updated_at: now,
                version: 1,
            })
            .await
            .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    }
    if service
        .repository()
        .list_deployment_authorities()
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .into_iter()
        .all(|authority| authority.authority_id != AUTHORITY_ID)
    {
        let proposal = participant
            .resolve()
            .map_err(|error| RuntimeError::Platform(error.to_string()))?
            .proposal()
            .clone();
        service
            .repository()
            .put_deployment_authority(
                DeploymentAuthorityRecord {
                    authority_id: AUTHORITY_ID.to_owned(),
                    deployment_id: DEPLOYMENT_ID.to_owned(),
                    participant_id: participant.participant_id.clone(),
                    participant_kind: participant.participant_kind,
                    participant_artifact_digest: participant.artifact_digest.clone(),
                    accepted_needs_digest: participant.needs_digest.clone(),
                    desired_grant_set: proposal.required().grant_set().clone(),
                    desired_capabilities: proposal
                        .required()
                        .capabilities()
                        .iter()
                        .map(|capability| capability.name().to_owned())
                        .collect(),
                    state: AuthorityState::Accepted,
                    version: 1,
                    created_at: now,
                    updated_at: now,
                    expires_at: None,
                    decision: Some(AuthorityDecision {
                        decided_at: now,
                        decided_by: "system:startup".to_owned(),
                        reason: Some("Rust auth runtime event publisher".to_owned()),
                    }),
                },
                None,
            )
            .await
            .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    }
    let target = AuthorityTarget::new(AuthorityKind::Deployment, AUTHORITY_ID)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    service
        .repository()
        .replace_resource_evidence(
            AuthorityEvidenceScope {
                target: target.clone(),
                participant_id: participant.participant_id.clone(),
                participant_artifact_digest: participant.artifact_digest.clone(),
                participant_needs_digest: participant.needs_digest.clone(),
            },
            [
                ("browserFlows", "trellis_auth_browser_flows"),
                ("oauthStates", "trellis_auth_oauth_states"),
                ("connectReplay", "trellis_auth_connect_replay"),
                ("connections", "trellis_auth_connections"),
            ]
            .into_iter()
            .map(|(local_name, bucket)| ResourceBindingEvidence {
                resource_kind: "kv".to_owned(),
                local_name: local_name.to_owned(),
                binding_id: format!("binding:{DEPLOYMENT_ID}:kv:{local_name}"),
                owner_participant_id: participant.participant_id.clone(),
                provider_identity: ResourceProviderIdentity::Kv {
                    bucket: bucket.to_owned(),
                },
                state: ResourceBindingState::Available,
                materialized_at: now,
                error: None,
            })
            .collect(),
        )
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let event_authority = authorization
        .reconcile_authority(&target, now)
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    if !event_authority
        .materialization
        .as_ref()
        .is_some_and(|value| value.authority.state == auth::MaterializationState::Available)
    {
        return Err(RuntimeError::Platform(format!(
            "auth event authority did not materialize: {event_authority:?}"
        )));
    }

    let mut seed = [0_u8; 32];
    getrandom::fill(&mut seed).map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let seed = URL_SAFE_NO_PAD.encode(seed);
    let session_auth = SessionAuth::from_seed_base64url(&seed)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let operation_auth = SessionAuth::from_seed_base64url(&seed)
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    let request_digest = trellis_protocol::digest_json(&serde_json::json!({
        "principalId": PRINCIPAL_ID,
        "sessionKey": session_auth.session_key,
        "createdAt": now,
    }))
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    service
        .create_session(CreateSessionInput {
            principal_id: PRINCIPAL_ID.to_owned(),
            principal_kind: PrincipalKind::Service,
            participant_id: participant.participant_id.clone(),
            participant_kind: participant.participant_kind,
            participant_artifact_digest: participant.artifact_digest.clone(),
            participant_needs_digest: participant.needs_digest.clone(),
            session_public_key: session_auth.session_key.clone(),
            desired_authority: None,
            deployment_id: Some(DEPLOYMENT_ID.to_owned()),
            instance_id: Some(INSTANCE_ID.to_owned()),
            created_at: now,
            idempotency: IdempotencyResultRecord {
                scope_key: request_digest.clone(),
                purpose: "auth.event_session.start".to_owned(),
                signer_id: PRINCIPAL_ID.to_owned(),
                request_id: request_digest.clone(),
                request_digest,
                result: serde_json::Value::Null,
                created_at: now,
                expires_at: now.saturating_add(86_400_000),
            },
            actions: Vec::new(),
        })
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    Ok((session_auth, operation_auth))
}

async fn ensure_builtin_portal(
    store: &SqliteAuthorizationStore,
    now: i64,
) -> Result<(), RuntimeError> {
    if store
        .get_login_portal("builtin")
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?
        .is_some()
    {
        return Ok(());
    }
    let request_digest = trellis_protocol::digest_json(&serde_json::json!({
        "portalId": "builtin",
        "version": 1,
    }))
    .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    store
        .put_login_portal(LoginPortalMutation {
            portal: LoginPortalRecord {
                portal_id: "builtin".to_owned(),
                display_name: "Trellis".to_owned(),
                entry_url: None,
                builtin: true,
                disabled: false,
                removed: false,
                local_registration_enabled: false,
                provider_ids: vec!["local".to_owned()],
                created_at: now,
                updated_at: now,
                version: 1,
            },
            settings: LoginSettingsRecord {
                portal_id: "builtin".to_owned(),
                default_provider_id: Some("local".to_owned()),
                local_login_enabled: true,
                federated_registration_enabled: true,
                provider_selection_enabled: false,
                updated_at: now,
                version: 1,
            },
            expected_version: None,
            idempotency: IdempotencyResultRecord {
                scope_key: request_digest.clone(),
                purpose: "portal.ensure_builtin".to_owned(),
                signer_id: "system:startup".to_owned(),
                request_id: "builtin-v1".to_owned(),
                request_digest,
                result: serde_json::json!({ "portalId": "builtin" }),
                created_at: now,
                expires_at: now.checked_add(86_400_000).ok_or_else(|| {
                    RuntimeError::Platform("portal idempotency expiry overflow".to_owned())
                })?,
            },
            actions: Vec::new(),
        })
        .await
        .map_err(|error| RuntimeError::Platform(error.to_string()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use auth::SessionRepository;

    #[tokio::test]
    async fn auth_event_session_materializes_owned_event_permissions() {
        let store = SqliteAuthorizationStore::open_in_memory().unwrap();
        let participant = auth::auth_runtime_participant_binding(1_700_000_000_000).unwrap();
        store
            .put_participant_binding(participant.clone())
            .await
            .unwrap();
        let authorization = AuthorizationStateService::new(store.clone());
        let service = AuthService::new(store.clone(), AuthServiceConfig::default()).unwrap();
        let (session_auth, _) =
            ensure_auth_event_session(&service, &authorization, &participant, 1_700_000_000_000)
                .await
                .unwrap();
        let session = store
            .get_session_by_public_key(&session_auth.session_key)
            .await
            .unwrap()
            .unwrap();
        let issuable = authorization
            .resolve_issuable_state(&session.session_id, 1_700_000_000_001)
            .await
            .unwrap();
        assert!(issuable.grant_set.permissions().iter().any(|permission| {
            permission.action() == trellis_protocol::PermissionActionV1::Publish
                && matches!(
                    permission.target(),
                    trellis_protocol::PermissionTargetV1::ApiSurface {
                        api,
                        surface: trellis_protocol::ApiSurfaceKindV1::Event,
                        ..
                    } if api == "trellis.auth@v1"
                )
        }));
        let permissions = auth::compile_transport_permissions(&issuable, &participant).unwrap();
        assert!(permissions
            .publish
            .contains(&"events.v1.Auth.Sessions.Revoked".to_owned()));
        assert!(permissions
            .publish
            .contains(&"events.v1.Auth.DeviceUserAuthorities.Resolved.*".to_owned()));
    }
}
