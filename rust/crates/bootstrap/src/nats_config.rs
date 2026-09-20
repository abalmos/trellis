use crate::types::{GeneratedMetadata, NatsBootstrapNames};

/// Render the local development NATS server config.
#[must_use]
pub fn render_nats_config(
    server_name: &str,
    nats_port: u16,
    monitor_port: u16,
    websocket_port: u16,
) -> String {
    format!(
        r#"server_name: {server_name}

listen: 0.0.0.0:{nats_port}
http: 0.0.0.0:{monitor_port}

authorization {{
  timeout: "30s"
}}

websocket {{
  listen: 0.0.0.0:{websocket_port}
  no_tls: true
}}

jetstream {{
  store_dir: /data
}}

include ./jwt.conf
"#
    )
}

/// Parse the native, monitoring, and websocket listen ports from an authored `nats.conf`.
///
/// Returns `None` when any of the three listeners is missing or malformed so callers can
/// fall back to their own defaults.
#[must_use]
pub fn parse_nats_listen_ports(config: &str) -> Option<(u16, u16, u16)> {
    let mut nats = None;
    let mut monitor = None;
    let mut websocket = None;
    let mut in_websocket = false;
    for line in config.lines() {
        let line = line.trim();
        if line == "websocket {" {
            in_websocket = true;
            continue;
        }
        if in_websocket && line == "}" {
            in_websocket = false;
            continue;
        }
        if let Some(value) = line.strip_prefix("listen:") {
            let port = parse_listen_port(value)?;
            if in_websocket {
                websocket = Some(port);
            } else {
                nats = Some(port);
            }
            continue;
        }
        if let Some(value) = line.strip_prefix("http:") {
            monitor = Some(parse_listen_port(value)?);
        }
    }
    Some((nats?, monitor?, websocket?))
}

fn parse_listen_port(value: &str) -> Option<u16> {
    value.trim().rsplit(':').next()?.trim().parse().ok()
}

/// Render the local development NATS server config with host-path JetStream store and JWT config.
///
/// All listeners bind to loopback only; the container-facing [`render_nats_config`] keeps
/// `0.0.0.0` for quadlet deployments.
#[must_use]
pub fn render_local_nats_config(
    server_name: &str,
    store_dir: &str,
    jwt_config_path: &str,
    nats_port: u16,
    websocket_port: u16,
    monitor_port: u16,
) -> String {
    format!(
        r#"server_name: {server_name}

listen: 127.0.0.1:{nats_port}
http: 127.0.0.1:{monitor_port}

authorization {{
  timeout: "30s"
}}

websocket {{
  listen: 127.0.0.1:{websocket_port}
  no_tls: true
}}

jetstream {{
  store_dir: {store_dir}
}}

include {jwt_config_path}
"#
    )
}

/// Render a generated JWT resolver config with mutable resolver data in `resolver_dir`.
#[must_use]
pub fn render_local_jwt_config(config: &str, resolver_dir: &str) -> String {
    config
        .lines()
        .map(|line| {
            if line.trim_start().starts_with("dir:") {
                format!("dir: {resolver_dir}")
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Builds the default NATS server-name slug from the Trellis name.
#[must_use]
pub fn slug_from_name(trellis_name: &str) -> String {
    let mut slug = String::new();
    let mut previous_was_separator = false;
    for character in trellis_name
        .trim()
        .chars()
        .map(|character| character.to_ascii_lowercase())
    {
        if character.is_ascii_alphanumeric() {
            slug.push(character);
            previous_was_separator = false;
        } else if !slug.is_empty() && !previous_was_separator {
            slug.push('-');
            previous_was_separator = true;
        }
    }
    while slug.ends_with('-') {
        slug.pop();
    }
    if slug.is_empty() {
        "trellis".to_string()
    } else {
        slug
    }
}

pub(crate) fn resolved_server_name(names: &NatsBootstrapNames, trellis_name: &str) -> String {
    names
        .server_name
        .as_ref()
        .map_or_else(|| slug_from_name(trellis_name), Clone::clone)
}

/// Render the auth callout environment file without seed material in the manifest.
#[must_use]
pub fn render_auth_callout_env(generated: &GeneratedMetadata) -> String {
    format!(
        r#"AUTH_ACCOUNT={auth_account}
AUTH_ACCOUNT_PUBLIC_KEY={auth_public}
TRELLIS_ACCOUNT={trellis_account}
TRELLIS_ACCOUNT_PUBLIC_KEY={trellis_public}
AUTH_USER_PUBLIC_KEY={auth_user}
TRELLIS_USER_PUBLIC_KEY={trellis_user}
AUTH_ISSUER_SIGNING_SEED_FILE=./secrets/auth-issuer-signing.seed
AUTH_TARGET_SIGNING_SEED_FILE=./secrets/auth-target-signing.seed
AUTH_CALLOUT_XKEY_SEED_FILE=./secrets/auth-sx.seed
AUTH_SERVICE_CREDS_FILE=./creds/auth-auth.creds
TRELLIS_SERVICE_CREDS_FILE=./creds/trellis-auth.creds
"#,
        auth_account = generated.auth_account_name,
        auth_public = generated.auth_account_public_key,
        trellis_account = generated.trellis_account_name,
        trellis_public = generated.trellis_account_public_key,
        auth_user = generated.auth_user_public_key,
        trellis_user = generated.trellis_user_public_key,
    )
}
