use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::{AdminSessionState, TrellisAuthError};
use crate::client::{
    AuthorizationClientState, AuthorizationContextStore, FileAuthorizationContextStore,
    TrellisClientError,
};

fn cli_config_dir() -> PathBuf {
    if let Ok(dir) = env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(dir).join("trellis");
    }
    if let Ok(home) = env::var("HOME") {
        return PathBuf::from(home).join(".config/trellis");
    }
    PathBuf::from(".trellis")
}

fn admin_session_state_path() -> PathBuf {
    cli_config_dir().join("admin-session.json")
}

pub(crate) fn admin_authorization_context_state_path() -> PathBuf {
    cli_config_dir().join("authorization-context.json")
}

/// Load the CLI installation's durable authorization trust and context state.
#[doc(hidden)]
pub fn load_admin_authorization_state() -> Result<Option<AuthorizationClientState>, TrellisAuthError>
{
    Ok(FileAuthorizationContextStore::new(admin_authorization_context_state_path()).load()?)
}

pub(crate) fn replacing_admin_authorization_context_store() -> Arc<dyn AuthorizationContextStore> {
    Arc::new(ReplacingAdminAuthorizationContextStore(
        FileAuthorizationContextStore::new(admin_authorization_context_state_path()),
        Mutex::new(true),
    ))
}

#[derive(Debug)]
struct ReplacingAdminAuthorizationContextStore(FileAuthorizationContextStore, Mutex<bool>);

impl AuthorizationContextStore for ReplacingAdminAuthorizationContextStore {
    fn load(&self) -> Result<Option<AuthorizationClientState>, TrellisClientError> {
        if *self
            .1
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?
        {
            Ok(None)
        } else {
            self.0.load()
        }
    }

    fn commit(
        &self,
        state: AuthorizationClientState,
    ) -> Result<AuthorizationClientState, TrellisClientError> {
        let mut replace = self
            .1
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        let committed = if *replace {
            self.0.replace_trust(state)
        } else {
            self.0.commit(state)
        }?;
        *replace = false;
        Ok(committed)
    }

    fn clear_context(&self) -> Result<(), TrellisClientError> {
        self.0.clear_context()
    }

    fn reset_trust(&self) -> Result<(), TrellisClientError> {
        self.0.reset_trust()
    }
}

fn write_private_file(path: &Path, contents: &str) -> Result<(), TrellisAuthError> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, contents)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    }
    Ok(())
}

/// Persist an admin session to the CLI config directory.
#[doc = concat!("Trellis API operation `", stringify!(save_admin_session), "`.")]
pub fn save_admin_session(state: &AdminSessionState) -> Result<(), TrellisAuthError> {
    let state_json = serde_json::to_string_pretty(state)?;
    write_private_file(&admin_session_state_path(), &state_json)
}

/// Load the current admin session from disk.
#[doc = concat!("Trellis API operation `", stringify!(load_admin_session), "`.")]
pub fn load_admin_session() -> Result<AdminSessionState, TrellisAuthError> {
    let path = admin_session_state_path();
    if !path.exists() {
        return Err(TrellisAuthError::AuthFlowFailed(
            "no stored admin session; run `trellis auth login`".to_string(),
        ));
    }
    let state = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&state)?)
}

/// Remove the stored admin session and related local credential files.
#[doc = concat!("Trellis API operation `", stringify!(clear_admin_session), "`.")]
pub fn clear_admin_session() -> Result<bool, TrellisAuthError> {
    FileAuthorizationContextStore::new(admin_authorization_context_state_path()).clear_context()?;
    let path = admin_session_state_path();
    if path.exists() {
        fs::remove_file(path)?;
        return Ok(true);
    }
    Ok(false)
}
