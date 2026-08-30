use crate::app::connect_authenticated_cli_client;
use crate::cli::*;
use crate::output;
use miette::IntoDiagnostic;
use qrcode::{render::unicode, QrCode};
use serde_json::{json, Value};
use std::collections::BTreeMap;
use std::io::{self, Write as _};
use trellis_rs::auth as authlib;
use trellis_rs::sdk::auth::types as auth_types;

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
        "capabilities": &me.capabilities,
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
            IdentityGrantsSubcommand::Revoke(args) => {
                identity_grants_revoke_command(format, &args).await
            }
        },
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
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    format!("cli_{nanos:x}")
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

fn value_string(value: &Value, field: &str) -> String {
    value
        .get(field)
        .and_then(|value| match value {
            Value::String(value) => Some(value.clone()),
            Value::Number(value) => Some(value.to_string()),
            _ => None,
        })
        .unwrap_or_default()
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
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
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
            deployment_id: None,
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
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
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
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
    let _username = trimmed_optional(&args.username)
        .ok_or_else(|| miette::miette!("--username is required to create a local user"))?;
    if !args.capabilities.is_empty() || !args.groups.is_empty() {
        return Err(miette::miette!(
            "direct capabilities and capability groups were removed; authorize the participant proposal instead"
        ));
    }
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
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
    let current = auth_client
        .rpc()
        .auth()
        .users_get(&auth_types::AuthUsersGetRequest {
            user_id: args.user_id.clone(),
        })
        .await
        .into_diagnostic()?
        .user;
    if args.clear_capabilities
        || args.clear_groups
        || !args.set_capabilities.is_empty()
        || !args.add_capabilities.is_empty()
        || !args.remove_capabilities.is_empty()
        || !args.set_groups.is_empty()
        || !args.add_groups.is_empty()
        || !args.remove_groups.is_empty()
    {
        return Err(miette::miette!(
            "direct capabilities and capability groups were removed; update participant authority instead"
        ));
    }
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

    let state = challenge
        .wait_for_session(&args.trellis_url)
        .await
        .into_diagnostic()?;
    let replace_trust = approve_login_trust(format, args.reset_trust, &state)?;
    let outcome = if replace_trust {
        authlib::authenticate_admin_session_replacing_trust(state).await
    } else {
        authlib::authenticate_admin_session(state).await
    }
    .into_diagnostic()?;
    let state = outcome.state;
    let me = outcome.user;
    let accepted_trust = if replace_trust {
        let root = trellis_protocol::AuthorizationTrustRoot::parse(
            &state.authorization_context.trust.root,
        )
        .into_diagnostic()?;
        Some((root.key_id().to_string(), root.digest().into_diagnostic()?))
    } else {
        None
    };

    authlib::save_admin_session(&state).into_diagnostic()?;

    if output::is_json(format) {
        let mut response = authenticated_user_json(&me);
        response["sessionKey"] = Value::String(state.session_key);
        response["expiresAt"] = state.expires_at.map(Value::from).unwrap_or(Value::Null);
        if let Some((key_id, fingerprint)) = accepted_trust {
            response["trustKeyId"] = Value::String(key_id);
            response["trustFingerprint"] = Value::String(fingerprint);
        }
        output::print_json(&response)?;
    } else {
        output::print_success("logged in delegated agent session");
        output::print_info(&format!("userId={}", me.user_id));
        output::print_info(&format!("identity={}", me.principal_id));
        output::print_info(&format!("name={}", me.name.as_deref().unwrap_or("")));
        output::print_info(&format!("sessionKey={}", state.session_key));
        output::print_info(&format!("expiresAt={:?}", state.expires_at));
        if let Some((key_id, fingerprint)) = accepted_trust {
            output::print_info(&format!("trustedRoot={key_id} {fingerprint}"));
        }
    }

    Ok(())
}

fn approve_login_trust(
    format: OutputFormat,
    reset_trust: bool,
    state: &authlib::AdminSessionState,
) -> miette::Result<bool> {
    let root =
        trellis_protocol::AuthorizationTrustRoot::parse(&state.authorization_context.trust.root)
            .into_diagnostic()?;
    let root_digest = root.digest().into_diagnostic()?;
    let stored = match authlib::load_admin_authorization_state() {
        Ok(Some(stored)) => StoredTrust::Valid {
            binding: stored.binding,
            authority: stored.trust.authority,
            root_key_id: stored.trust.root_key_id,
            root_digest: stored.trust.root_digest,
        },
        Ok(None) => StoredTrust::Missing,
        Err(error) => StoredTrust::Unreadable(error.to_string()),
    };
    let binding = format!("installation:{}", state.trellis_url);
    let prompt = match login_trust_prompt(
        stored,
        &binding,
        root.authority(),
        root.key_id(),
        &root_digest,
    ) {
        None => return Ok(false),
        Some(prompt) => prompt,
    };

    if !reset_trust {
        if output::is_json(format) {
            return Err(miette::miette!(
                "{prompt}\nauthorization trust requires confirmation; rerun with --reset-trust"
            ));
        }
        println!("{prompt}");
        print!("Continue? [y/N] ");
        io::stdout().flush().into_diagnostic()?;
        let mut answer = String::new();
        io::stdin().read_line(&mut answer).into_diagnostic()?;
        if !matches!(answer.trim().to_ascii_lowercase().as_str(), "y" | "yes") {
            return Err(miette::miette!("authorization trust was not accepted"));
        }
    }

    Ok(true)
}

enum StoredTrust {
    Missing,
    Valid {
        binding: String,
        authority: String,
        root_key_id: String,
        root_digest: String,
    },
    Unreadable(String),
}

fn login_trust_prompt(
    stored: StoredTrust,
    binding: &str,
    authority: &str,
    root_key_id: &str,
    root_digest: &str,
) -> Option<String> {
    match stored {
        StoredTrust::Valid {
            binding: stored_binding,
            authority: stored_authority,
            root_key_id: stored_key,
            root_digest: stored_digest,
        } if stored_binding == binding
            && stored_authority == authority
            && stored_key == root_key_id
            && stored_digest == root_digest => None,
        StoredTrust::Valid {
            binding: old_binding,
            authority: old_authority,
            root_key_id: old_key,
            root_digest: old_digest,
        } => Some(format!(
            "WARNING: authorization trust changed.\nOld deployment: {old_binding}\nOld authority: {old_authority}\nOld key: {old_key}\nOld fingerprint: {old_digest}\nNew deployment: {binding}\nNew authority: {authority}\nNew key: {root_key_id}\nNew fingerprint: {root_digest}\nReplace the stored trust root?"
        )),
        StoredTrust::Missing => Some(format!(
            "Trust this Trellis deployment?\nAuthority: {authority}\nKey: {root_key_id}\nFingerprint: {root_digest}"
        )),
        StoredTrust::Unreadable(error) => Some(format!(
            "WARNING: stored authorization trust cannot be read ({error}).\nNew authority: {authority}\nNew key: {root_key_id}\nNew fingerprint: {root_digest}\nDiscard the unreadable state and trust this root?"
        )),
    }
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
    let response = authlib::AuthClient::new(connected)
        .rpc()
        .auth()
        .sessions_me()
        .await
        .map_err(|error| authlib::TrellisAuthError::OperationFailed(error.to_string()))?;
    let user = response.user.ok_or_else(|| {
        authlib::TrellisAuthError::NotUserSession(
            response.session.participant_kind.as_str().to_owned(),
        )
    })?;
    Ok(serde_json::from_value(serde_json::to_value(user)?)?)
}

async fn revoke_current_session(
    connected: &trellis_rs::generated::Caller,
) -> Result<(), authlib::TrellisAuthError> {
    let auth = authlib::AuthClient::new(connected);
    let current = auth
        .rpc()
        .auth()
        .sessions_me()
        .await
        .map_err(|error| authlib::TrellisAuthError::OperationFailed(error.to_string()))?;
    auth.rpc()
        .auth()
        .sessions_revoke(&auth_types::AuthSessionsRevokeRequest {
            session_id: current.session.session_id,
            expected_version: Some(current.session.version),
            reason: Some("CLI logout".to_owned()),
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .map_err(|error| authlib::TrellisAuthError::OperationFailed(error.to_string()))?;
    Ok(())
}

async fn status_command(format: OutputFormat) -> miette::Result<()> {
    let (state, connected) = connect_authenticated_cli_client(format).await?;
    let me = current_user(&connected).await.into_diagnostic()?;

    if output::is_json(format) {
        let mut response = authenticated_user_json(&me);
        response["loggedIn"] = Value::Bool(true);
        response["sessionKey"] = Value::String(state.session_key);
        response["expiresAt"] = state.expires_at.map(Value::from).unwrap_or(Value::Null);
        output::print_json(&response)?;
    } else {
        output::print_success("delegated agent session is active");
        output::print_info(&format!("userId={}", me.user_id));
        output::print_info(&format!("identity={}", me.principal_id));
        output::print_info(&format!("name={}", me.name.as_deref().unwrap_or("")));
        output::print_info(&format!("sessionKey={}", state.session_key));
        output::print_info(&format!("expiresAt={:?}", state.expires_at));
    }

    Ok(())
}

async fn identity_grants_list_command(
    format: OutputFormat,
    args: &IdentityGrantsListArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
    let identity_grants = auth_client
        .rpc()
        .auth()
        .identity_authority_list(&auth_types::AuthIdentityAuthorityListRequest {
            principal_id: args.user.clone(),
            participant_id: None,
            state: None,
            cursor: None,
            limit: Some(100),
        })
        .await
        .into_diagnostic()?
        .entries;
    let identity_values = identity_grants
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .into_diagnostic()?;
    let identity_values = identity_values
        .into_iter()
        .filter(|entry| {
            args.digest.as_deref().is_none_or(|digest| {
                entry
                    .get("participantArtifactDigest")
                    .and_then(Value::as_str)
                    == Some(digest)
            })
        })
        .collect::<Vec<_>>();

    if output::is_json(format) {
        output::print_json(&json!({
            "user": args.user,
            "digest": args.digest,
            "identityAuthorities": identity_values,
        }))?;
        return Ok(());
    }

    output::print_info(&format!(
        "matched identity grants={}",
        identity_values.len()
    ));
    if let Some(user) = &args.user {
        output::print_info(&format!("user={user}"));
    }
    if let Some(digest) = &args.digest {
        output::print_info(&format!("digest={digest}"));
    }

    let rows = identity_values
        .into_iter()
        .map(|entry| {
            vec![
                value_string(&entry, "authorityId"),
                value_string(&entry, "participantId"),
                value_string(&entry, "state"),
                value_string(&entry, "participantArtifactDigest"),
                value_string(&entry, "updatedAt"),
            ]
        })
        .collect();
    println!(
        "{}",
        output::table(
            &["authorityId", "participantId", "state", "digest", "updated"],
            rows
        )
    );
    Ok(())
}

async fn identity_grants_revoke_command(
    format: OutputFormat,
    args: &IdentityGrantsRevokeArgs,
) -> miette::Result<()> {
    let (_state, connected) = connect_authenticated_cli_client(format).await?;
    let auth_client = authlib::AuthClient::new(&connected);
    let authority = auth_client
        .rpc()
        .auth()
        .identity_authority_get(&auth_types::AuthIdentityAuthorityGetRequest {
            authority_id: args.identity_grant_id.clone(),
        })
        .await
        .into_diagnostic()?
        .authority;
    if args
        .user
        .as_deref()
        .is_some_and(|user| authority.principal_id != user)
    {
        return Err(miette::miette!(
            "identity authority does not belong to requested user"
        ));
    }
    auth_client
        .rpc()
        .auth()
        .identity_authority_revoke(&auth_types::AuthIdentityAuthorityRevokeRequest {
            authority_id: args.identity_grant_id.clone(),
            expected_version: authority.version,
            reason: None,
            idempotency_key: cli_idempotency_key(),
        })
        .await
        .into_diagnostic()?;
    let success = true;

    if output::is_json(format) {
        output::print_json(&json!({
            "success": success,
            "identityGrantId": args.identity_grant_id,
            "user": args.user,
        }))?;
        return Ok(());
    }

    if success {
        output::print_success("revoked identity grant");
    } else {
        output::print_info("no matching identity grant found");
    }
    output::print_info(&format!("identityGrantId={}", args.identity_grant_id));
    if let Some(user) = &args.user {
        output::print_info(&format!("user={user}"));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        authenticated_user_json, login_trust_prompt, pending_agent_login_json,
        render_agent_login_instructions, StoredTrust,
    };
    use serde_json::json;
    use trellis_rs::auth::AuthenticatedUser;

    #[test]
    fn login_trust_prompts_only_for_first_changed_or_unreadable_roots() {
        assert!(login_trust_prompt(
            StoredTrust::Valid {
                binding: "installation:https://trellis.example.com".into(),
                authority: "authority".into(),
                root_key_id: "key".into(),
                root_digest: "digest".into(),
            },
            "installation:https://trellis.example.com",
            "authority",
            "key",
            "digest",
        )
        .is_none());
        assert!(login_trust_prompt(
            StoredTrust::Missing,
            "installation:https://trellis.example.com",
            "authority",
            "key",
            "digest",
        )
        .expect("first trust prompt")
        .contains("Fingerprint: digest"));
        assert!(login_trust_prompt(
            StoredTrust::Valid {
                binding: "installation:https://trellis.example.com".into(),
                authority: "authority".into(),
                root_key_id: "old-key".into(),
                root_digest: "old-digest".into(),
            },
            "installation:https://trellis.example.com",
            "authority",
            "new-key",
            "new-digest",
        )
        .expect("changed trust prompt")
        .contains("Old fingerprint: old-digest"));
        assert!(login_trust_prompt(
            StoredTrust::Unreadable("invalid json".into()),
            "installation:https://trellis.example.com",
            "authority",
            "key",
            "digest",
        )
        .expect("unreadable trust prompt")
        .contains("invalid json"));
    }

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
            capabilities: vec!["admin".to_string()],
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
                "capabilities": ["admin"],
            })
        );
    }
}
