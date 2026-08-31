use std::fs;
use std::io::Write as _;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use super::super::TrellisClientError;
use super::types::{
    AuthorizationClientState, AuthorizationContextStore, AUTHORIZATION_CLIENT_STATE_FORMAT_,
};

#[cfg(test)]
use super::types::AuthorizationClientTrustState;

/// Explicitly ephemeral client authorization storage for tests and short-lived clients.
#[derive(Debug, Default)]
pub struct MemoryAuthorizationContextStore {
    state: Mutex<Option<AuthorizationClientState>>,
}

impl AuthorizationContextStore for MemoryAuthorizationContextStore {
    fn load(&self) -> Result<Option<AuthorizationClientState>, TrellisClientError> {
        Ok(self
            .state
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?
            .clone())
    }

    fn commit(
        &self,
        state: AuthorizationClientState,
    ) -> Result<AuthorizationClientState, TrellisClientError> {
        let mut current = self
            .state
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        validate_client_state_transition(current.as_ref(), &state)?;
        *current = Some(state.clone());
        Ok(state)
    }

    fn clear_context(&self) -> Result<(), TrellisClientError> {
        if let Some(state) = self
            .state
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?
            .as_mut()
        {
            state.context = None;
            state.routing = None;
        }
        Ok(())
    }

    fn reset_trust(&self) -> Result<(), TrellisClientError> {
        *self
            .state
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))? =
            None;
        Ok(())
    }
}

/// Atomic JSON-file client authorization storage for CLI and native runtimes.
#[derive(Clone, Debug)]
pub struct FileAuthorizationContextStore {
    path: PathBuf,
    update: Arc<Mutex<()>>,
}

impl FileAuthorizationContextStore {
    /// Create a file-backed store at the caller-owned private path.
    #[must_use]
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            update: Arc::new(Mutex::new(())),
        }
    }

    fn load_file(&self) -> Result<Option<AuthorizationClientState>, TrellisClientError> {
        match fs::read_to_string(&self.path) {
            Ok(value) => serde_json::from_str(&value)
                .map(Some)
                .map_err(TrellisClientError::from),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(error) => Err(error.into()),
        }
    }

    fn write_file(&self, state: &AuthorizationClientState) -> Result<(), TrellisClientError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let parent = self.path.parent().unwrap_or_else(|| Path::new("."));
        let name = self
            .path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or("authorization-context");
        let temporary = parent.join(format!(".{name}.{}.tmp", ulid::Ulid::new()));
        let result = (|| {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temporary)?;
            file.write_all(&serde_json::to_vec_pretty(state)?)?;
            set_private_permissions(&temporary)?;
            file.sync_all()?;
            fs::rename(&temporary, &self.path)?;
            fs::File::open(parent)?.sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            let _ = fs::remove_file(temporary);
        }
        result
    }
}

impl AuthorizationContextStore for FileAuthorizationContextStore {
    fn load(&self) -> Result<Option<AuthorizationClientState>, TrellisClientError> {
        let _update = self
            .update
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        self.load_file()
    }

    fn commit(
        &self,
        state: AuthorizationClientState,
    ) -> Result<AuthorizationClientState, TrellisClientError> {
        let _update = self
            .update
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        validate_client_state_transition(self.load_file()?.as_ref(), &state)?;
        self.write_file(&state)?;
        Ok(state)
    }

    fn clear_context(&self) -> Result<(), TrellisClientError> {
        let _update = self
            .update
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        if let Some(mut state) = self.load_file()? {
            state.context = None;
            state.routing = None;
            self.write_file(&state)?;
        }
        Ok(())
    }

    fn reset_trust(&self) -> Result<(), TrellisClientError> {
        let _update = self
            .update
            .lock()
            .map_err(|_| TrellisClientError::Bootstrap("context store lock poisoned".into()))?;
        match fs::remove_file(&self.path) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error.into()),
        }
    }
}

#[cfg(unix)]
fn set_private_permissions(path: &Path) -> Result<(), TrellisClientError> {
    use std::os::unix::fs::PermissionsExt as _;

    fs::set_permissions(path, fs::Permissions::from_mode(0o600))?;
    Ok(())
}

#[cfg(not(unix))]
fn set_private_permissions(_path: &Path) -> Result<(), TrellisClientError> {
    Ok(())
}

pub(crate) fn validate_client_state_transition(
    current: Option<&AuthorizationClientState>,
    next: &AuthorizationClientState,
) -> Result<(), TrellisClientError> {
    if next.format != AUTHORIZATION_CLIENT_STATE_FORMAT_
        || next.trust.format != "trellis.authorization-client-trust.v1"
        || next.binding.trim().is_empty()
        || next.runtime.session_id.trim().is_empty()
        || next.runtime.participant_id.trim().is_empty()
        || next.runtime.participant_digest.trim().is_empty()
        || next.runtime.needs_digest.trim().is_empty()
        || next.runtime.inbox_prefix.trim().is_empty()
        || next.runtime.transports.native.nats_servers.is_empty()
        || next
            .runtime
            .transports
            .native
            .nats_servers
            .iter()
            .any(|server| server.trim().is_empty())
        || next.context.is_some() != next.routing.is_some()
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization client state is invalid".into(),
        ));
    }
    let Some(current) = current else {
        return Ok(());
    };
    if current.binding != next.binding
        || current.trust.authority != next.trust.authority
        || current.trust.root_key_id != next.trust.root_key_id
        || current.trust.root_digest != next.trust.root_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization trust root changed".into(),
        ));
    }
    if (current.runtime.session_id != next.runtime.session_id
        && (current.context.is_some() || current.routing.is_some()))
        || current.runtime.participant_id != next.runtime.participant_id
        || current.runtime.participant_digest != next.runtime.participant_digest
        || current.runtime.needs_digest != next.runtime.needs_digest
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization runtime session binding changed".into(),
        ));
    }
    if next.trust.minimum_manifest_generation < current.trust.minimum_manifest_generation {
        return Err(TrellisClientError::Bootstrap(
            "authorization issuer manifest rolled back".into(),
        ));
    }
    if next.trust.minimum_manifest_generation == current.trust.minimum_manifest_generation
        && next.trust.manifest_digest_at_minimum_generation
            != current.trust.manifest_digest_at_minimum_generation
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization issuer manifest equivocated".into(),
        ));
    }
    Ok(())
}

#[cfg(test)]
pub(crate) fn test_state(generation: u64, manifest_digest: &str) -> AuthorizationClientState {
    AuthorizationClientState {
        format: AUTHORIZATION_CLIENT_STATE_FORMAT_.into(),
        binding: "service:dep:instance".into(),
        trust: AuthorizationClientTrustState {
            format: "trellis.authorization-client-trust.v1".into(),
            authority: "trellis-test".into(),
            root_key_id: "root-key".into(),
            root_digest: "root-digest".into(),
            minimum_manifest_generation: generation,
            manifest_digest_at_minimum_generation: manifest_digest.into(),
        },
        runtime: super::types::AuthorizationRuntimeBinding {
            session_id: "ses_test".into(),
            participant_id: "participant".into(),
            participant_digest: "participant".into(),
            needs_digest: "needs".into(),
            inbox_prefix: "_INBOX.test".into(),
            transports: super::types::AuthorizationRuntimeTransports {
                native: super::types::AuthorizationNativeTransport {
                    nats_servers: vec!["nats://localhost:4222".into()],
                },
            },
        },
        context: None,
        routing: None,
        server_clock_offset_ms: 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_store_persists_complete_floor_and_rejects_restart_rollback() {
        let path = std::env::temp_dir().join(format!(
            "trellis-authorization-context-{}.json",
            ulid::Ulid::new()
        ));
        let store = FileAuthorizationContextStore::new(&path);
        store
            .commit(test_state(7, "manifest-7"))
            .expect("commit floor");
        store
            .commit(test_state(8, "manifest-8"))
            .expect("advance floor");

        let reopened = FileAuthorizationContextStore::new(&path);
        assert_eq!(
            reopened
                .load()
                .expect("load floor")
                .expect("stored state")
                .trust
                .manifest_digest_at_minimum_generation,
            "manifest-8"
        );
        assert!(reopened.commit(test_state(7, "manifest-7")).is_err());
        assert!(reopened.commit(test_state(8, "equivocated")).is_err());
        let mut replaced_root = test_state(9, "manifest-9");
        replaced_root.trust.root_digest = "another-root".into();
        assert!(reopened.commit(replaced_root).is_err());
        let mut rebound = test_state(9, "manifest-9");
        rebound.binding = "device:other".into();
        assert!(reopened.commit(rebound).is_err());
        reopened.reset_trust().expect("remove test store");
    }

    #[test]
    fn file_store_accepts_new_session_after_context_clear_without_resetting_trust() {
        let path = std::env::temp_dir().join(format!(
            "trellis-authorization-context-{}.json",
            ulid::Ulid::new()
        ));
        let store = FileAuthorizationContextStore::new(&path);
        let first = test_state(8, "manifest-8");
        store.commit(first.clone()).expect("commit first session");
        store.clear_context().expect("clear first session context");

        let mut second = first;
        second.runtime.session_id = "session-2".into();
        store.commit(second).expect("commit replacement session");
        let persisted = store.load().expect("load replacement").expect("state");
        assert_eq!(persisted.runtime.session_id, "session-2");
        assert_eq!(persisted.trust.root_digest, "root-digest");
        assert_eq!(persisted.trust.minimum_manifest_generation, 8);
        store.reset_trust().expect("remove test store");
    }
}
