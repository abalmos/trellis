use crate::app::connect_authenticated_cli_client;
use crate::cli::*;
use crate::output;
use miette::IntoDiagnostic;
use qrcode::{render::unicode, QrCode};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use trellis_rs::auth as authlib;
use trellis_runtime_apis::auth::types as auth_types;
use trellis_runtime_apis::auth::{rpc::AuthParticipantsGetError, AuthClient};
use ulid::Ulid;

pub(crate) fn render_agent_login_instructions(login_url: &str) -> miette::Result<String> {
    let qr = QrCode::new(login_url.as_bytes()).into_diagnostic()?;
    let qr = qr.render::<unicode::Dense1x2>().quiet_zone(false).build();
    Ok(format!(
        "Open this activation URL:\n{login_url}\n\nScan this QR code:\n{qr}"
    ))
}

pub(crate) fn pending_agent_login_json(login_url: &str) -> Value {
    json!({
        "status": "pending",
        "loginUrl": login_url,
    })
}

fn authenticated_user_json(me: &authlib::AuthenticatedUser) -> Value {
    json!({
        "userId": &me.user_id,
        "principalId": &me.principal_id,
        "state": &me.state,
        "name": &me.name,
    })
}

pub(super) async fn login(format: OutputFormat, args: &LoginArgs) -> miette::Result<()> {
    login_command(format, args).await
}

pub(super) async fn logout(format: OutputFormat) -> miette::Result<()> {
    logout_command(format).await
}

pub(super) async fn whoami(format: OutputFormat) -> miette::Result<()> {
    status_command(format).await
}

pub(super) async fn identity(format: OutputFormat, command: IdentityCommand) -> miette::Result<()> {
    match command.command {
        IdentitySubcommand::Grants(command) => match command.command {
            IdentityGrantsSubcommand::List(args) => {
                identity_grants_list_command(format, &args).await
            }
            IdentityGrantsSubcommand::Get(args) => identity_grants_get_command(format, &args).await,
            IdentityGrantsSubcommand::Set(args) => identity_grants_set_command(format, &args).await,
            IdentityGrantsSubcommand::Revoke(args) => {
                identity_grants_revoke_command(format, &args).await
            }
        },
    }
}

pub(super) async fn participants(
    format: OutputFormat,
    command: ParticipantsCommand,
) -> miette::Result<()> {
    match command.command {
        ParticipantsSubcommand::Install(args) => participants_install_command(format, &args).await,
    }
}

pub(super) async fn issuers(format: OutputFormat, command: IssuersCommand) -> miette::Result<()> {
    match command.command {
        IssuersSubcommand::Revoke(args) => issuers_revoke_command(format, &args).await,
    }
}

pub(super) async fn users(format: OutputFormat, command: UsersCommand) -> miette::Result<()> {
    match command.command {
        UsersSubcommand::List => users_list_command(format).await,
        UsersSubcommand::Show(args) => users_show_command(format, &args).await,
        UsersSubcommand::Create(args) => users_create_command(format, &args).await,
        UsersSubcommand::Edit(args) => users_edit_command(format, &args).await,
    }
}

pub(super) async fn portals(format: OutputFormat, command: PortalsCommand) -> miette::Result<()> {
    portals_command(format, command).await
}

async fn portals_command(format: OutputFormat, command: PortalsCommand) -> miette::Result<()> {
    let command_name = match command.command {
        PortalsSubcommand::List => "portals list",
        PortalsSubcommand::Login(login) => match login.command {
            PortalsLoginSubcommand::Default => "portals login default",
            PortalsLoginSubcommand::Selection => "portals login selection",
        },
    };
    if output::is_json(format) {
        output::print_json(&json!({
            "status": "not_implemented",
            "command": command_name,
            "message": "Portal admin RPC client wiring is pending; use Console or call Auth.Portals.* RPCs directly."
        }))?;
    } else {
        output::print_info(&format!(
            "{command_name}: portal admin RPC client wiring is pending; use Console or call Auth.Portals.* RPCs directly."
        ));
    }
    Ok(())
}

fn trimmed_optional(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn cli_idempotency_key() -> String {
    Ulid::new().to_string()
}

fn identity_labels(identities: &[Value]) -> String {
    identities
        .iter()
        .filter_map(|identity| {
            let provider = identity.get("provider")?.as_str()?;
            let subject = identity.get("subject")?.as_str()?;
            Some(format!("{provider}:{subject}"))
        })
        .collect::<Vec<_>>()
        .join(",")
}

fn latest_last_auth_by_user(sessions: &[Value]) -> BTreeMap<String, String> {
    let mut last_auth_by_user: BTreeMap<String, String> = BTreeMap::new();
    for session in sessions {
        let Some(user_id) = session
            .get("principal")
            .and_then(|principal| principal.get("userId"))
            .and_then(Value::as_str)
        else {
            continue;
        };
        let Some(last_auth) = session.get("lastAuth").and_then(Value::as_str) else {
            continue;
        };
        match last_auth_by_user.get(user_id) {
            Some(existing) if existing.as_str() >= last_auth => {}
            _ => {
                last_auth_by_user.insert(user_id.to_string(), last_auth.to_string());
            }
        }
    }
    last_auth_by_user
}

fn user_label(user: &Value) -> String {
    user.get("name")
        .and_then(Value::as_str)
        .or_else(|| user.get("email").and_then(Value::as_str))
        .unwrap_or("")
        .to_string()
}

fn user_email(user: &Value) -> String {
    user.get("email")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string()
}

fn string_array_field(user: &Value, field: &str) -> Vec<String> {
    user.get(field)
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(ToOwned::to_owned)
                .collect()
        })
        .unwrap_or_default()
}

fn direct_capabilities(user: &Value) -> String {
    string_array_field(user, "capabilities").join(",")
}

fn capability_groups(user: &Value) -> String {
    string_array_field(user, "capabilityGroups").join(",")
}

fn identities_field(user: &Value) -> String {
    user.get("identities")
        .and_then(Value::as_array)
        .map(|identities| identity_labels(identities))
        .unwrap_or_default()
}

fn user_row(user: &Value, last_auth_by_user: &BTreeMap<String, String>) -> Vec<String> {
    let user_id = user
        .get("userId")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    vec![
        user_id.clone(),
        user_label(user),
        user_email(user),
        user.get("active")
            .and_then(Value::as_bool)
            .map(|active| active.to_string())
            .unwrap_or_default(),
        direct_capabilities(user),
        capability_groups(user),
        identities_field(user),
        last_auth_by_user.get(&user_id).cloned().unwrap_or_default(),
    ]
}

async fn users_list_command(format: OutputFormat) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let auth_client = AuthClient::new(&connected);
    let users = auth_client
        .rpc()
        .auth()
        .users_list(&auth_types::AuthUsersListRequest {
            state: None,
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?;
    let user_values = users
        .entries
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    let sessions = auth_client
        .rpc()
        .auth()
        .sessions_list(&auth_types::AuthSessionsListRequest {
            principal_id: None,
            participant_id: None,
            state: None,
            cursor: None,
            limit: Some(100),
        })
        .await
        .map(|response| response.entries)
        .unwrap_or_default();
    let session_values = sessions
        .iter()
        .filter_map(|session| serde_json::to_value(session).ok())
        .collect::<Vec<_>>();
    let last_auth_by_user = latest_last_auth_by_user(&session_values);

    if output::is_json(format) {
        output::print_json(&json!({
            "users": users.entries,
            "lastAuthByUser": last_auth_by_user,
        }))?;
        return Ok(());
    }

    let rows = user_values
        .iter()
        .map(|user| user_row(user, &last_auth_by_user))
        .collect();
    println!(
        "{}",
        output::table(
            &[
                "userId",
                "label",
                "email",
                "active",
                "direct",
                "groups",
                "identities",
                "lastAuth"
            ],
            rows
        )
    );
    Ok(())
}

async fn users_show_command(format: OutputFormat, args: &UserRefArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let auth_client = AuthClient::new(&connected);
    let user = auth_client
        .rpc()
        .auth()
        .users_get(&auth_types::AuthUsersGetRequest {
            user_id: args.user_id.clone(),
        })
        .await
        .into_diagnostic()?
        .user;

    if output::is_json(format) {
        output::print_json(&json!({ "user": user }))?;
        return Ok(());
    }

    let user_value = serde_json::to_value(&user).into_diagnostic()?;
    output::print_info(&format!("userId={}", user.user_id));
    output::print_info(&format!("state={}", user.state));
    output::print_info(&format!("name={}", user.name.as_deref().unwrap_or("")));
    output::print_info(&format!("email={}", user.email.as_deref().unwrap_or("")));
    output::print_info(&format!("direct={}", direct_capabilities(&user_value)));
    output::print_info(&format!("groups={}", capability_groups(&user_value)));
    output::print_info(&format!("identities={}", identities_field(&user_value)));
    Ok(())
}

async fn users_create_command(format: OutputFormat, args: &UserCreateArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let auth_client = AuthClient::new(&connected);
    let _username = trimmed_optional(&args.username)
        .ok_or_else(|| miette::miette!("--username is required to create a local user"))?;
    let user = auth_client
        .rpc()
        .auth()
        .users_create(&auth_types::AuthUsersCreateRequest {
            email: trimmed_optional(&args.email),
            name: trimmed_optional(&args.name),
            image: None,
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .into_diagnostic()?
        .user;
    let setup_flow = auth_client
        .rpc()
        .auth()
        .users_password_reset_create(&auth_types::AuthUsersPasswordResetCreateRequest {
            user_id: user.user_id.clone(),
            return_target: None,
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .into_diagnostic()?;

    if output::is_json(format) {
        output::print_json(&json!({
            "user": user,
            "setupFlow": setup_flow,
        }))?;
        return Ok(());
    }

    output::print_success("created user");
    output::print_info(&format!("userId={}", user.user_id));
    output::print_info(&format!(
        "setupFlow={}",
        serde_json::to_string(&setup_flow).into_diagnostic()?
    ));
    Ok(())
}

async fn users_edit_command(format: OutputFormat, args: &UserEditArgs) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let auth_client = AuthClient::new(&connected);
    let current = auth_client
        .rpc()
        .auth()
        .users_get(&auth_types::AuthUsersGetRequest {
            user_id: args.user_id.clone(),
        })
        .await
        .into_diagnostic()?
        .user;
    let next_name = trimmed_optional(&args.name);
    let next_email = trimmed_optional(&args.email);

    let user = auth_client
        .rpc()
        .auth()
        .users_update(&auth_types::AuthUsersUpdateRequest {
            email: next_email.or(current.email),
            name: next_name.or(current.name),
            image: current.image,
            state: if args.active {
                auth_types::AuthUsersUpdateRequestState::Active
            } else if args.inactive {
                auth_types::AuthUsersUpdateRequestState::Disabled
            } else {
                match current.state {
                    auth_types::AuthUsersGetResponseUserState::Active => {
                        auth_types::AuthUsersUpdateRequestState::Active
                    }
                    auth_types::AuthUsersGetResponseUserState::Disabled
                    | auth_types::AuthUsersGetResponseUserState::Revoked => {
                        auth_types::AuthUsersUpdateRequestState::Disabled
                    }
                }
            },
            user_id: args.user_id.clone(),
            expected_version: current.version,
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .into_diagnostic()?
        .user;

    if output::is_json(format) {
        output::print_json(&json!({
            "user": user,
            "userId": args.user_id,
        }))?;
        return Ok(());
    }

    output::print_success("updated user");
    output::print_info(&format!("userId={}", args.user_id));
    Ok(())
}

async fn login_command(format: OutputFormat, args: &LoginArgs) -> miette::Result<()> {
    let challenge = authlib::start_agent_login(&authlib::StartAgentLoginOpts {
        trellis_url: &args.trellis_url,
    })
    .await
    .into_diagnostic()?;
    let login_url = challenge.login_url().to_string();

    if output::is_json(format) {
        output::print_json_progress(&pending_agent_login_json(&login_url))?;
    } else {
        output::print_info(&render_agent_login_instructions(&login_url)?);
    }

    let outcome = challenge
        .complete(&args.trellis_url)
        .await
        .into_diagnostic()?;
    let state = outcome.state;
    let me = outcome.user;

    authlib::save_admin_session(&state).into_diagnostic()?;

    if output::is_json(format) {
        let mut response = authenticated_user_json(&me);
        response["sessionKey"] = Value::String(state.session_key().into_diagnostic()?);
        response["expiresAt"] = state.expires_at.map(Value::from).unwrap_or(Value::Null);
        output::print_json(&response)?;
    } else {
        output::print_success("logged in delegated agent session");
        output::print_info(&format!("userId={}", me.user_id));
        output::print_info(&format!("identity={}", me.principal_id));
        output::print_info(&format!("name={}", me.name.as_deref().unwrap_or("")));
        output::print_info(&format!(
            "sessionKey={}",
            state.session_key().into_diagnostic()?
        ));
        output::print_info(&format!("expiresAt={:?}", state.expires_at));
    }

    Ok(())
}

async fn logout_command(format: OutputFormat) -> miette::Result<()> {
    let mut revoked = false;
    let mut revoke_error = None;
    if let Ok(state) = authlib::load_admin_session() {
        match authlib::connect_admin_client_async(&state).await {
            Ok(connected) => match revoke_current_session(&connected).await {
                Ok(()) => revoked = true,
                Err(error) => revoke_error = Some(error.to_string()),
            },
            Err(error) => revoke_error = Some(error.to_string()),
        }
    }
    let removed = authlib::clear_admin_session().into_diagnostic()?;
    if output::is_json(format) {
        let mut response = json!({ "cleared": removed, "revoked": revoked });
        if let Some(error) = &revoke_error {
            response["revokeError"] = Value::String(error.clone());
        }
        output::print_json(&response)?;
    } else if removed {
        if revoked {
            output::print_success("revoked remote session and cleared local agent session");
        } else if let Some(error) = &revoke_error {
            output::print_success("cleared stored agent session");
            output::print_info(&format!(
                "warning: remote session revocation failed: {error}"
            ));
        } else {
            output::print_success("cleared stored agent session");
        }
    } else {
        output::print_info("no stored agent session found");
    }
    Ok(())
}

pub(super) async fn current_user(
    connected: &trellis_rs::generated::Caller,
) -> Result<authlib::AuthenticatedUser, authlib::TrellisAuthError> {
    let response = AuthClient::new(connected)
        .rpc()
        .auth()
        .sessions_me()
        .await
        .map_err(|error| authlib::TrellisAuthError::OperationFailed(error.to_string()))?;
    let user = response.user.ok_or_else(|| {
        authlib::TrellisAuthError::NotUserSession(
            response.connection.principal_kind.as_str().to_owned(),
        )
    })?;
    Ok(serde_json::from_value(serde_json::to_value(user)?)?)
}

async fn revoke_current_session(
    connected: &trellis_rs::generated::Caller,
) -> Result<(), authlib::TrellisAuthError> {
    let auth = AuthClient::new(connected);
    auth.rpc()
        .auth()
        .sessions_logout()
        .await
        .map_err(|error| authlib::TrellisAuthError::OperationFailed(error.to_string()))?;
    Ok(())
}

async fn status_command(format: OutputFormat) -> miette::Result<()> {
    let (state, connected) = connect_authenticated_cli_client().await?;
    let me = current_user(&connected).await.into_diagnostic()?;

    if output::is_json(format) {
        let mut response = authenticated_user_json(&me);
        response["loggedIn"] = Value::Bool(true);
        response["sessionKey"] = Value::String(state.session_key().into_diagnostic()?);
        response["expiresAt"] = state.expires_at.map(Value::from).unwrap_or(Value::Null);
        output::print_json(&response)?;
    } else {
        output::print_success("delegated agent session is active");
        output::print_info(&format!("userId={}", me.user_id));
        output::print_info(&format!("identity={}", me.principal_id));
        output::print_info(&format!("name={}", me.name.as_deref().unwrap_or("")));
        output::print_info(&format!(
            "sessionKey={}",
            state.session_key().into_diagnostic()?
        ));
        output::print_info(&format!("expiresAt={:?}", state.expires_at));
    }

    Ok(())
}

async fn identity_grants_list_command(
    format: OutputFormat,
    args: &IdentityGrantsListArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let owner_id = match &args.user {
        Some(user) => user.clone(),
        None => current_user(&connected).await.into_diagnostic()?.user_id,
    };
    let response = AuthClient::new(&connected)
        .rpc()
        .auth()
        .grants_list(&auth_types::AuthGrantsListRequest {
            cursor: None,
            limit: Some(100),
            owner_id: Some(owner_id.clone()),
            owner_kind: Some(auth_types::AuthGrantsListRequestOwnerKind::User),
            participant_id: args.participant.clone(),
            state: None,
        })
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else {
        output::print_info(&format!("user={owner_id}"));
        output::print_info(&format!("matched grants={}", response.entries.len()));
        let rows = response
            .entries
            .iter()
            .map(|entry| {
                vec![
                    entry.owner_id.clone(),
                    entry.participant_id.clone(),
                    entry.state.to_string(),
                    entry.revision.to_string(),
                    entry.installed_revision.to_string(),
                ]
            })
            .collect();
        println!(
            "{}",
            output::table(
                &["owner", "participant", "state", "revision", "installed"],
                rows
            )
        );
    }
    Ok(())
}

async fn identity_grants_revoke_command(
    format: OutputFormat,
    args: &IdentityGrantsRevokeArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let owner_id = match &args.user {
        Some(user) => user.clone(),
        None => current_user(&connected).await.into_diagnostic()?.user_id,
    };
    let auth = AuthClient::new(&connected);
    let expected_revision = match args.expected_revision {
        Some(revision) => revision,
        None => current_grant_revision(&connected, &owner_id, &args.participant_id)
            .await?
            .ok_or_else(|| miette::miette!("grant binding does not exist"))?,
    };
    let response = auth
        .rpc()
        .auth()
        .grants_revoke(&auth_types::AuthGrantsRevokeRequest {
            expected_revision: i64::try_from(expected_revision).into_diagnostic()?,
            idempotency_key: cli_idempotency_key(),
            owner_id: owner_id.clone(),
            owner_kind: auth_types::AuthGrantsRevokeRequestOwnerKind::User,
            participant_id: args.participant_id.clone(),
            reason: args.reason.clone(),
        })
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else {
        output::print_success("revoked identity grant");
        output::print_info(&format!("user={owner_id}"));
        output::print_info(&format!("participant={}", args.participant_id));
    }
    Ok(())
}

async fn identity_grants_get_command(
    format: OutputFormat,
    args: &IdentityGrantsGetArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let owner_id = match &args.user {
        Some(user) => user.clone(),
        None => current_user(&connected).await.into_diagnostic()?.user_id,
    };
    let response = AuthClient::new(&connected)
        .rpc()
        .auth()
        .grants_get(&auth_types::AuthGrantsGetRequest {
            owner_id: owner_id.clone(),
            owner_kind: auth_types::AuthGrantsGetRequestOwnerKind::User,
            participant_id: args.participant_id.clone(),
        })
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else if let Some(binding) = response.binding {
        output::print_json(&binding)?;
    } else {
        output::print_info("no matching identity grant");
    }
    Ok(())
}

async fn identity_grants_set_command(
    format: OutputFormat,
    args: &IdentityGrantsSetArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let mut input: Value =
        serde_json::from_slice(&std::fs::read(&args.input).into_diagnostic()?).into_diagnostic()?;
    let object = input
        .as_object_mut()
        .ok_or_else(|| miette::miette!("grant input must be a JSON object"))?;
    let expected = [
        "expiresAt",
        "grants",
        "installedRevision",
        "platformPrivileges",
    ];
    if object.len() != expected.len() || expected.iter().any(|field| !object.contains_key(*field)) {
        return Err(miette::miette!(
            "grant input must contain exactly installedRevision, grants, platformPrivileges, and expiresAt"
        ));
    }
    let expected_revision = match args.expected_revision {
        Some(revision) => revision,
        None => current_grant_revision(&connected, &args.user, &args.participant_id)
            .await?
            .unwrap_or(0),
    };
    object.insert("ownerKind".to_owned(), json!("user"));
    object.insert("ownerId".to_owned(), json!(args.user));
    object.insert("participantId".to_owned(), json!(args.participant_id));
    object.insert("expectedRevision".to_owned(), json!(expected_revision));
    object.insert("idempotencyKey".to_owned(), json!(cli_idempotency_key()));
    let request: auth_types::AuthGrantsSetRequest =
        serde_json::from_value(input).into_diagnostic()?;
    let response = AuthClient::new(&connected)
        .rpc()
        .auth()
        .grants_set(&request)
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else {
        output::print_success("set identity grant");
        output::print_info(&format!("user={}", args.user));
        output::print_info(&format!("participant={}", args.participant_id));
    }
    Ok(())
}

async fn current_grant_revision(
    connected: &trellis_rs::generated::Caller,
    owner_id: &str,
    participant_id: &str,
) -> miette::Result<Option<u64>> {
    let response = AuthClient::new(connected)
        .rpc()
        .auth()
        .grants_get(&auth_types::AuthGrantsGetRequest {
            owner_id: owner_id.to_owned(),
            owner_kind: auth_types::AuthGrantsGetRequestOwnerKind::User,
            participant_id: participant_id.to_owned(),
        })
        .await
        .into_diagnostic()?;
    Ok(response
        .binding
        .and_then(|binding| binding.get("revision").and_then(Value::as_u64)))
}

async fn participants_install_command(
    format: OutputFormat,
    args: &ParticipantsInstallArgs,
) -> miette::Result<()> {
    let participant =
        super::deploy::compile_participant_input(&args.source, args.participant.as_deref(), None)?;
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let auth = AuthClient::new(&connected);
    let expected_revision = match args.expected_revision {
        Some(revision) => revision,
        None => match auth
            .rpc()
            .auth()
            .participants_get(&auth_types::AuthParticipantsGetRequest {
                participant_id: participant.participant_id.clone(),
                revision: None,
            })
            .await
        {
            Ok(current) => u64::try_from(current.participant.revision).into_diagnostic()?,
            Err(trellis_rs::client::CallError::Declared(error))
                if matches!(
                    error.as_ref(),
                    AuthParticipantsGetError::AuthError(error) if error.reason == "not_found"
                ) =>
            {
                0
            }
            Err(error) => return Err(error).into_diagnostic(),
        },
    };
    let response = auth
        .rpc()
        .auth()
        .participants_install(&auth_types::AuthParticipantsInstallRequest {
            api_artifacts: participant.api_artifacts,
            expected_revision: i64::try_from(expected_revision).into_diagnostic()?,
            idempotency_key: cli_idempotency_key(),
            participant_artifact: participant.participant_artifact,
        })
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else {
        output::print_success("installed participant definition");
        output::print_info(&format!(
            "participantId={}",
            response.participant.participant_id
        ));
        output::print_info(&format!("revision={}", response.participant.revision));
    }
    Ok(())
}

async fn issuers_revoke_command(
    format: OutputFormat,
    args: &IssuersRevokeArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client().await?;
    let response = AuthClient::new(&connected)
        .rpc()
        .auth()
        .issuers_revoke(&auth_types::AuthIssuersRevokeRequest {
            idempotency_key: cli_idempotency_key(),
            key_id: args.key_id.clone(),
            reason: args.reason.clone(),
        })
        .await
        .into_diagnostic()?;
    if output::is_json(format) {
        output::print_json(&serde_json::to_value(response).into_diagnostic()?)?;
    } else {
        output::print_success("revoked issuer");
        output::print_info(&format!("keyId={}", response.key_id));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        authenticated_user_json, pending_agent_login_json, render_agent_login_instructions,
    };
    use serde_json::json;
    use trellis_rs::auth::AuthenticatedUser;

    #[test]
    fn agent_login_instructions_include_plain_url_and_terminal_qr() {
        let instructions =
            render_agent_login_instructions("https://auth.example.com/login?flowId=flow_123")
                .expect("render instructions");

        assert!(instructions.contains("Open this activation URL:"));
        assert!(instructions.contains("https://auth.example.com/login?flowId=flow_123"));
        assert!(instructions.contains("Scan this QR code:"));
        assert!(
            instructions.contains("█") || instructions.contains("▀") || instructions.contains("▄")
        );
    }

    #[test]
    fn pending_agent_login_json_includes_login_url() {
        assert_eq!(
            pending_agent_login_json("https://auth.example.com/login?flowId=flow_123"),
            json!({
                "status": "pending",
                "loginUrl": "https://auth.example.com/login?flowId=flow_123",
            })
        );
    }

    #[test]
    fn authenticated_user_output_is_account_first() {
        let user = AuthenticatedUser {
            principal_id: "usr_123".to_string(),
            state: "active".to_string(),
            email: Some("ada@example.com".to_string()),
            image: None,
            name: Some("Ada".to_string()),
            user_id: "usr_123".to_string(),
        };

        assert_eq!(
            authenticated_user_json(&user),
            json!({
                "userId": "usr_123",
                "principalId": "usr_123",
                "state": "active",
                "name": "Ada",
            })
        );
    }
}
