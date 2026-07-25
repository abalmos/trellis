use std::{
    fmt, fs,
    io::Write as _,
    path::{Path, PathBuf},
    sync::{
        atomic::{AtomicI64, Ordering},
        Arc, Mutex, RwLock,
    },
};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::{
    authorization_context_refresh_at_v1, parse_authorization_context_token_v1,
    parse_authorization_context_v1, parse_issuer_certificate_v1, parse_issuer_manifest_v1,
    session_proof_request_digest_v1, verify_authorization_context_v1, verify_issuer_manifest_v1,
    AuthorizationTrustRootV1, AuthorizationVerificationPolicyV1, SessionProofInputV1,
    SignedAuthorizationContextV1,
};

use super::{proof::new_request_id, SessionAuth, TrellisClientError};

const AUTHORIZATION_CLIENT_STATE_FORMAT_V1: &str = "trellis.authorization-client-state.v1";

/// Client-side verification limits distributed with the pinned trust root.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTrustPolicy {
    /// Symmetric clock skew accepted by the issuer.
    pub allowed_clock_skew_seconds: u32,
    /// Maximum context lease duration.
    pub maximum_context_lifetime_seconds: u32,
    /// Maximum canonical signed-context JSON size in UTF-8 bytes.
    pub maximum_context_bytes: usize,
    /// Maximum exact permission atoms.
    pub maximum_permissions: usize,
    /// Maximum platform capability names.
    pub maximum_capabilities: usize,
    /// Safety lead before expiry used for proactive refresh.
    pub refresh_lead_seconds: u32,
    /// Deterministic earlier-only refresh jitter window.
    pub refresh_jitter_seconds: u32,
}

/// Pinned root plus lazy trust and context registry locators.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTrustBundle {
    /// Pinned public trust root.
    pub root: Value,
    /// Monotonic issuer-manifest generation.
    pub issuer_manifest_generation: u64,
    /// Exact canonical signed manifest digest.
    pub issuer_manifest_digest: String,
    /// HTTP locator for the exact issuer manifest.
    pub issuer_manifest_locator: String,
    /// HTTP locator for the active issuer certificate.
    pub issuer_certificate_locator: String,
    /// HTTP locator prefix for signed contexts and revocations.
    pub context_registry_locator: String,
    /// Verification policy bound to this runtime configuration.
    pub policy: AuthorizationTrustPolicy,
}

/// Signed authorization context and its minimal trust metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationContextBundle {
    /// Compact signed authorization-context token.
    pub context: String,
    /// Canonical digest carried by transport proofs.
    pub context_digest: String,
    /// Protocol-derived proactive refresh time.
    pub refresh_at: i64,
    /// Pinned root and lazy registry locators.
    pub trust: AuthorizationTrustBundle,
}

/// Route-selection JWT installed atomically with an authorization context.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationRoutingMaterial {
    /// Deny-all Auth-account JWT used only to select the Auth Callout route.
    pub bootstrap_jwt: String,
    /// JWT expiry as Unix seconds.
    pub bootstrap_jwt_expires_at: i64,
}

/// Stable session evidence retained when a short-lived context expires.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationSessionBinding {
    /// Stable session identifier.
    pub session_id: String,
    /// Exact participant artifact digest expected during recovery.
    pub participant_digest: String,
    /// Exact participant needs digest expected during recovery.
    pub needs_digest: String,
}

/// Complete installation-scoped authorization trust rollback floor.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationClientTrustState {
    /// Client trust-state wire format.
    pub format: String,
    /// Authorization namespace pinned by the installation.
    pub authority: String,
    /// Content-derived root key identifier.
    pub root_key_id: String,
    /// Canonical digest of the exact pinned root object.
    pub root_digest: String,
    /// Lowest issuer-manifest generation accepted by the installation.
    pub minimum_manifest_generation: u64,
    /// Exact manifest digest accepted at the generation floor.
    pub manifest_digest_at_minimum_generation: String,
}

/// Atomic client authorization state persisted by a runtime installation.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationClientState {
    /// Client state wire format.
    pub format: String,
    /// Caller-owned storage binding, such as a service instance or device identity.
    pub binding: String,
    /// Durable installation trust floor.
    pub trust: AuthorizationClientTrustState,
    /// Stable proof-bound session evidence retained across context expiry.
    pub session: AuthorizationSessionBinding,
    /// Current signed context, or `None` after session clearing.
    pub context: Option<AuthorizationContextBundle>,
    /// Route JWT paired atomically with the current context.
    pub routing: Option<AuthorizationRoutingMaterial>,
}

/// Narrow persistence port for one client installation's trust floor and context.
pub trait AuthorizationContextStore: fmt::Debug + Send + Sync {
    /// Load the atomically persisted client state.
    fn load(&self) -> Result<Option<AuthorizationClientState>, TrellisClientError>;

    /// Atomically advance the trust floor and current context.
    fn commit(
        &self,
        state: AuthorizationClientState,
    ) -> Result<AuthorizationClientState, TrellisClientError>;

    /// Clear only the session-bound context while retaining installation trust.
    fn clear_context(&self) -> Result<(), TrellisClientError>;

    /// Explicitly reset both context and installation trust.
    fn reset_trust(&self) -> Result<(), TrellisClientError>;
}

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

fn validate_client_state_transition(
    current: Option<&AuthorizationClientState>,
    next: &AuthorizationClientState,
) -> Result<(), TrellisClientError> {
    if next.format != AUTHORIZATION_CLIENT_STATE_FORMAT_V1
        || next.trust.format != "trellis.authorization-client-trust.v1"
        || next.binding.trim().is_empty()
        || next.session.session_id.trim().is_empty()
        || next.session.participant_digest.trim().is_empty()
        || next.session.needs_digest.trim().is_empty()
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

#[derive(Clone, Debug)]
struct CurrentContext {
    bundle: AuthorizationContextBundle,
    manifest_generation: u64,
    session_id: String,
    participant_digest: String,
    needs_digest: String,
    not_before: i64,
    expires_at: i64,
    refresh_at: i64,
}

#[derive(Clone, Debug, Default)]
struct CachedAuthorizationState {
    current: Option<CurrentContext>,
    session: Option<AuthorizationSessionBinding>,
    routing: Option<AuthorizationRoutingMaterial>,
}

/// Verified process-local authorization context used by reconnect callbacks.
#[derive(Clone)]
pub struct AuthorizationContextCache {
    trellis_url: reqwest::Url,
    binding: String,
    store: Arc<dyn AuthorizationContextStore>,
    client: reqwest::Client,
    state: Arc<RwLock<CachedAuthorizationState>>,
    clock_offset_ms: Arc<AtomicI64>,
    update: Arc<tokio::sync::Mutex<()>>,
}

impl AuthorizationContextCache {
    /// Create a cache using explicit caller-owned persistence.
    pub fn new(
        trellis_url: &str,
        binding: impl Into<String>,
        store: Arc<dyn AuthorizationContextStore>,
    ) -> Result<Self, TrellisClientError> {
        let binding = binding.into();
        if binding.trim().is_empty() {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage binding is empty".into(),
            ));
        }
        Ok(Self {
            trellis_url: reqwest::Url::parse(trellis_url)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?,
            binding,
            store,
            client: reqwest::Client::builder()
                .redirect(reqwest::redirect::Policy::none())
                .build()
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?,
            state: Arc::new(RwLock::new(CachedAuthorizationState::default())),
            clock_offset_ms: Arc::new(AtomicI64::new(0)),
            update: Arc::new(tokio::sync::Mutex::new(())),
        })
    }

    /// Create an explicitly ephemeral cache for tests or short-lived clients.
    pub fn ephemeral(
        trellis_url: &str,
        binding: impl Into<String>,
    ) -> Result<Self, TrellisClientError> {
        Self::new(
            trellis_url,
            binding,
            Arc::new(MemoryAuthorizationContextStore::default()),
        )
    }

    /// Restore and verify the atomically persisted current context, if present.
    pub async fn restore(&self, now_unix_seconds: i64) -> Result<bool, TrellisClientError> {
        let Some(state) = self.store.load()? else {
            return Ok(false);
        };
        if state.binding != self.binding {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage belongs to another identity".into(),
            ));
        }
        let (bundle, routing) = match (state.context, state.routing) {
            (Some(bundle), Some(routing)) => (bundle, routing),
            (None, None) => {
                self.state
                    .write()
                    .map_err(|_| {
                        TrellisClientError::Bootstrap("context cache lock poisoned".into())
                    })?
                    .session = Some(state.session);
                return Ok(false);
            }
            _ => {
                return Err(TrellisClientError::Bootstrap(
                    "persisted context and routing material are not atomic".into(),
                ));
            }
        };
        self.install_recoverable(bundle, routing, now_unix_seconds)
            .await
    }

    pub(crate) async fn install_recoverable(
        &self,
        bundle: AuthorizationContextBundle,
        routing: AuthorizationRoutingMaterial,
        now_unix_seconds: i64,
    ) -> Result<bool, TrellisClientError> {
        let signed = persisted_signed_context(&bundle)?;
        let verification_now = if signed.unsigned.expires_at <= now_unix_seconds {
            signed
                .unsigned
                .expires_at
                .saturating_sub(1)
                .max(signed.unsigned.not_before)
        } else {
            now_unix_seconds
        };
        self.install(bundle, routing.clone(), verification_now)
            .await?;
        if signed.unsigned.expires_at <= now_unix_seconds
            || routing.bootstrap_jwt_expires_at <= now_unix_seconds
        {
            self.clear()?;
            return Ok(false);
        }
        Ok(true)
    }

    /// Fetch and verify the complete trust chain before replacing the current context.
    pub async fn install(
        &self,
        bundle: AuthorizationContextBundle,
        routing: AuthorizationRoutingMaterial,
        now_unix_seconds: i64,
    ) -> Result<(), TrellisClientError> {
        let _update = self.update.lock().await;
        let durable = self.store.load()?;
        if durable
            .as_ref()
            .is_some_and(|state| state.binding != self.binding)
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization context storage belongs to another identity".into(),
            ));
        }
        let minimum_generation =
            durable
                .as_ref()
                .map_or(bundle.trust.issuer_manifest_generation, |state| {
                    state
                        .trust
                        .minimum_manifest_generation
                        .max(bundle.trust.issuer_manifest_generation)
                });
        let manifest: Value = self
            .fetch_json(
                &bundle.trust.issuer_manifest_locator,
                AuthorizationLocatorKind::Manifest,
            )
            .await?;
        let certificate: Value = self
            .fetch_json(
                &bundle.trust.issuer_certificate_locator,
                AuthorizationLocatorKind::Certificate,
            )
            .await?;
        let root = AuthorizationTrustRootV1::parse(&bundle.trust.root)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let root_digest = root
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if durable.as_ref().is_some_and(|state| {
            state.trust.authority != root.authority()
                || state.trust.root_key_id != root.key_id()
                || state.trust.root_digest != root_digest
        }) {
            return Err(TrellisClientError::Bootstrap(
                "authorization trust root changed".into(),
            ));
        }
        let policy = AuthorizationVerificationPolicyV1::new(
            now_unix_seconds,
            bundle.trust.policy.allowed_clock_skew_seconds,
            bundle.trust.policy.maximum_context_lifetime_seconds,
            bundle.trust.policy.maximum_context_bytes,
            bundle.trust.policy.maximum_permissions,
            bundle.trust.policy.maximum_capabilities,
            minimum_generation,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let manifest = parse_issuer_manifest_v1(&manifest)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let verified_manifest = verify_issuer_manifest_v1(&root, &manifest, &policy)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let manifest_digest = verified_manifest
            .digest()
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if verified_manifest.generation() != bundle.trust.issuer_manifest_generation
            || manifest_digest != bundle.trust.issuer_manifest_digest
        {
            return Err(TrellisClientError::Bootstrap(
                "authorization issuer manifest identity mismatch".into(),
            ));
        }
        let certificate = parse_issuer_certificate_v1(&certificate)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let context = parse_authorization_context_token_v1(&bundle.context, &policy)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let verified = verify_authorization_context_v1(
            &root,
            &verified_manifest,
            &certificate,
            &context,
            &policy,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if verified.context_digest() != bundle.context_digest {
            return Err(TrellisClientError::Bootstrap(
                "authorization context digest mismatch".into(),
            ));
        }
        let context = &verified.signed_context().unsigned;
        if routing.bootstrap_jwt.trim().is_empty() || routing.bootstrap_jwt_expires_at <= 0 {
            return Err(TrellisClientError::Bootstrap(
                "authorization routing material is expired or empty".into(),
            ));
        }
        let refresh_at = authorization_context_refresh_at_v1(
            &bundle.context_digest,
            context.issued_at,
            context.not_before,
            context.expires_at,
            bundle.trust.policy.refresh_lead_seconds,
            bundle.trust.policy.refresh_jitter_seconds,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        if refresh_at != bundle.refresh_at {
            return Err(TrellisClientError::Bootstrap(
                "authorization context refresh schedule mismatch".into(),
            ));
        }
        let trust = AuthorizationClientTrustState {
            format: "trellis.authorization-client-trust.v1".into(),
            authority: root.authority().to_owned(),
            root_key_id: root.key_id().to_owned(),
            root_digest,
            minimum_manifest_generation: verified_manifest.generation(),
            manifest_digest_at_minimum_generation: manifest_digest,
        };
        let session = AuthorizationSessionBinding {
            session_id: context.session_id.clone(),
            participant_digest: context.participant.artifact_digest.clone(),
            needs_digest: context.participant.needs_digest.clone(),
        };
        let next = AuthorizationClientState {
            format: AUTHORIZATION_CLIENT_STATE_FORMAT_V1.into(),
            binding: self.binding.clone(),
            trust,
            session: session.clone(),
            context: Some(bundle.clone()),
            routing: Some(routing.clone()),
        };
        let persisted = self.store.commit(next.clone())?;
        if persisted != next {
            return Err(TrellisClientError::Bootstrap(
                "authorization context persistence did not commit exact state".into(),
            ));
        }
        let current = CurrentContext {
            bundle,
            manifest_generation: verified_manifest.generation(),
            session_id: context.session_id.clone(),
            participant_digest: context.participant.artifact_digest.clone(),
            needs_digest: context.participant.needs_digest.clone(),
            not_before: context.not_before,
            expires_at: context.expires_at,
            refresh_at,
        };
        *self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
            CachedAuthorizationState {
                current: Some(current),
                session: Some(session),
                routing: Some(routing),
            };
        Ok(())
    }

    /// Clear the active session context while retaining the durable trust floor.
    pub fn clear(&self) -> Result<(), TrellisClientError> {
        self.store.clear_context()?;
        let mut state = self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        state.current = None;
        state.routing = None;
        Ok(())
    }

    /// Explicitly clear both active context and installation trust.
    pub fn reset_trust(&self) -> Result<(), TrellisClientError> {
        self.store.reset_trust()?;
        *self
            .state
            .write()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))? =
            CachedAuthorizationState::default();
        Ok(())
    }

    /// Return the current context digest for a reconnect proof.
    pub fn context_digest(&self) -> Result<String, TrellisClientError> {
        let now_unix_seconds = self.corrected_now_seconds()?;
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let current = state
            .current
            .as_ref()
            .filter(|context| {
                context.not_before <= now_unix_seconds && context.expires_at > now_unix_seconds
            })
            .ok_or_else(|| TrellisClientError::Bootstrap("authorization context expired".into()))?;
        Ok(current.bundle.context_digest.clone())
    }

    /// Return a copy of the currently verified bundle.
    pub fn bundle(&self) -> Result<AuthorizationContextBundle, TrellisClientError> {
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .current
            .as_ref()
            .map(|current| current.bundle.clone())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization context unavailable".into())
            })
    }

    /// Refresh the current context through the proof-bound auth endpoint.
    pub async fn refresh(&self, auth: &SessionAuth) -> Result<(), TrellisClientError> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct RefreshRequest {
            request_id: String,
            issued_at: i64,
            session_id: String,
            session_nkey: String,
            current_context_digest: Option<String>,
            expected_participant_digest: Option<String>,
            expected_needs_digest: Option<String>,
            known_root_key_id: String,
            minimum_manifest_generation: i64,
            proof: Value,
        }

        #[derive(Deserialize)]
        #[serde(rename_all = "camelCase")]
        struct RefreshResponse {
            server_now: i64,
            authorization_context: AuthorizationContextBundle,
            bootstrap_jwt: String,
            bootstrap_jwt_expires_at: i64,
        }

        let now = self.corrected_now_seconds()?;
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .clone();
        let session = state.session.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization session unavailable".into())
        })?;
        let durable = self.store.load()?.ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization trust floor unavailable".into())
        })?;
        let request_started_at = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                .as_millis(),
        )
        .map_err(|_| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
        let issued_at = request_started_at
            .checked_add(self.clock_offset_ms.load(Ordering::Relaxed))
            .ok_or_else(|| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
        let mut request = RefreshRequest {
            request_id: new_request_id(),
            issued_at,
            session_id: session.session_id,
            session_nkey: auth.nkey_pair()?.public_key(),
            current_context_digest: state
                .current
                .as_ref()
                .filter(|value| value.not_before <= now && value.expires_at > now)
                .map(|value| value.bundle.context_digest.clone()),
            expected_participant_digest: Some(session.participant_digest),
            expected_needs_digest: Some(session.needs_digest),
            known_root_key_id: durable.trust.root_key_id,
            minimum_manifest_generation: i64::try_from(durable.trust.minimum_manifest_generation)
                .map_err(|_| {
                TrellisClientError::Bootstrap("manifest generation overflow".into())
            })?,
            proof: serde_json::json!({
                "format": "trellis.session-proof.v1",
                "signature": "",
            }),
        };
        let request_value = serde_json::to_value(&request)?;
        let request_digest = session_proof_request_digest_v1(&request_value)
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let input = SessionProofInputV1::authorization_context_refresh(
            &request.request_id,
            request.issued_at,
            &request.session_id,
            auth.key_id(),
            request.current_context_digest.clone(),
            request.expected_participant_digest.clone(),
            request.expected_needs_digest.clone(),
            &request.known_root_key_id,
            request.minimum_manifest_generation,
            &request_digest,
        )
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        request.proof = serde_json::to_value(auth.sign_session_proof(&input)?)?;
        let response = self
            .client
            .post(
                self.trellis_url
                    .join("/auth/context/refresh")
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?,
            )
            .json(&request)
            .send()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let status = response.status();
        if !status.is_success() {
            return Err(TrellisClientError::BootstrapHttp {
                status: status.as_u16(),
                body: response.text().await.unwrap_or_default(),
            });
        }
        let response = response
            .json::<RefreshResponse>()
            .await
            .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
        let response_received_at = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                .as_millis(),
        )
        .map_err(|_| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
        let midpoint = request_started_at
            .checked_add(response_received_at)
            .and_then(|sum| sum.checked_div(2))
            .ok_or_else(|| TrellisClientError::Bootstrap("context refresh time overflow".into()))?;
        self.clock_offset_ms
            .store(response.server_now - midpoint, Ordering::Relaxed);
        self.install(
            response.authorization_context,
            AuthorizationRoutingMaterial {
                bootstrap_jwt: response.bootstrap_jwt,
                bootstrap_jwt_expires_at: response.bootstrap_jwt_expires_at,
            },
            response.server_now.div_euclid(1_000),
        )
        .await
    }

    pub(crate) fn refresh_delay(&self) -> Result<std::time::Duration, TrellisClientError> {
        let now_unix_seconds = self.corrected_now_seconds()?;
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let Some(current) = state.current.as_ref() else {
            return Ok(std::time::Duration::from_secs(1));
        };
        let route_refresh_at = state.routing.as_ref().map_or(now_unix_seconds, |routing| {
            routing
                .bootstrap_jwt_expires_at
                .saturating_sub(i64::from(current.bundle.trust.policy.refresh_lead_seconds))
        });
        Ok(std::time::Duration::from_secs(
            u64::try_from(
                current
                    .refresh_at
                    .min(route_refresh_at)
                    .saturating_sub(now_unix_seconds)
                    .max(5),
            )
            .map_err(|_| TrellisClientError::Bootstrap("context refresh delay overflow".into()))?,
        ))
    }

    pub(crate) fn routing_jwt(&self) -> Result<String, TrellisClientError> {
        let now = self.corrected_now_seconds()?;
        self.state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?
            .routing
            .as_ref()
            .filter(|routing| routing.bootstrap_jwt_expires_at > now)
            .map(|routing| routing.bootstrap_jwt.clone())
            .ok_or_else(|| {
                TrellisClientError::Bootstrap("authorization routing JWT expired".into())
            })
    }

    pub(crate) fn set_server_clock_offset_ms(&self, offset_ms: i64) {
        self.clock_offset_ms.store(offset_ms, Ordering::Relaxed);
    }

    fn corrected_now_seconds(&self) -> Result<i64, TrellisClientError> {
        self.corrected_now_millis()
            .map(|value| value.div_euclid(1_000))
    }

    pub(crate) fn corrected_now_millis(&self) -> Result<i64, TrellisClientError> {
        let now = i64::try_from(
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?
                .as_millis(),
        )
        .map_err(|_| TrellisClientError::Bootstrap("context time overflow".into()))?;
        now.checked_add(self.clock_offset_ms.load(Ordering::Relaxed))
            .ok_or_else(|| TrellisClientError::Bootstrap("context time overflow".into()))
    }

    pub(crate) fn refresh_evidence(
        &self,
    ) -> Result<(String, String, String, String, u64), TrellisClientError> {
        let state = self
            .state
            .read()
            .map_err(|_| TrellisClientError::Bootstrap("context cache lock poisoned".into()))?;
        let current = state.current.as_ref().ok_or_else(|| {
            TrellisClientError::Bootstrap("authorization context unavailable".into())
        })?;
        Ok((
            current.session_id.clone(),
            current.bundle.context_digest.clone(),
            current.participant_digest.clone(),
            current.needs_digest.clone(),
            current.manifest_generation,
        ))
    }

    async fn fetch_json(
        &self,
        locator: &str,
        kind: AuthorizationLocatorKind,
    ) -> Result<Value, TrellisClientError> {
        let url = resolve_authorization_locator(&self.trellis_url, locator, kind)?;
        for attempt in 0..3 {
            let response = self
                .client
                .get(url.clone())
                .send()
                .await
                .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
            if response.url().origin() != url.origin() {
                return Err(TrellisClientError::Bootstrap(
                    "authorization registry redirected outside the Trellis origin".into(),
                ));
            }
            if response.status().is_success() {
                return response
                    .json()
                    .await
                    .map_err(|error| TrellisClientError::Bootstrap(error.to_string()));
            }
            if response.status() != reqwest::StatusCode::NOT_FOUND || attempt == 2 {
                return Err(TrellisClientError::Bootstrap(format!(
                    "authorization registry returned HTTP {}",
                    response.status()
                )));
            }
            tokio::time::sleep(std::time::Duration::from_millis(50 * (attempt + 1))).await;
        }
        Err(TrellisClientError::Bootstrap(
            "authorization registry object is unavailable".into(),
        ))
    }
}

#[derive(Clone, Copy)]
enum AuthorizationLocatorKind {
    Manifest,
    Certificate,
}

fn resolve_authorization_locator(
    base: &reqwest::Url,
    locator: &str,
    kind: AuthorizationLocatorKind,
) -> Result<reqwest::Url, TrellisClientError> {
    if !matches!(base.scheme(), "http" | "https") {
        return Err(TrellisClientError::Bootstrap(
            "Trellis authorization origin must use HTTP(S)".into(),
        ));
    }
    if (!locator.starts_with('/')
        && !locator.starts_with("http://")
        && !locator.starts_with("https://"))
        || locator.contains(['\\', '%', '?', '#'])
        || locator.starts_with("//")
        || locator
            .split('/')
            .any(|segment| matches!(segment, "." | ".."))
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization locator is not a canonical HTTP(S) path".into(),
        ));
    }
    let url = base
        .join(locator)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    if !matches!(url.scheme(), "http" | "https")
        || url.origin() != base.origin()
        || !url.username().is_empty()
        || url.password().is_some()
        || url.query().is_some()
        || url.fragment().is_some()
    {
        return Err(TrellisClientError::Bootstrap(
            "authorization locator must use the Trellis HTTP(S) origin".into(),
        ));
    }
    let valid = match kind {
        AuthorizationLocatorKind::Manifest => url
            .path()
            .strip_prefix("/.well-known/trellis/authorization/trust/manifest.")
            .is_some_and(|generation| {
                !generation.is_empty()
                    && !generation.starts_with('0')
                    && generation.bytes().all(|byte| byte.is_ascii_digit())
            }),
        AuthorizationLocatorKind::Certificate => url
            .path()
            .strip_prefix("/.well-known/trellis/authorization/trust/certificate.")
            .and_then(|suffix| suffix.split_once('.'))
            .is_some_and(|(key_id, digest)| {
                [key_id, digest].into_iter().all(|part| {
                    !part.is_empty()
                        && part
                            .bytes()
                            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
                })
            }),
    };
    if !valid {
        return Err(TrellisClientError::Bootstrap(
            "authorization locator path is invalid".into(),
        ));
    }
    Ok(url)
}

fn persisted_signed_context(
    bundle: &AuthorizationContextBundle,
) -> Result<SignedAuthorizationContextV1, TrellisClientError> {
    let bytes = URL_SAFE_NO_PAD
        .decode(&bundle.context)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))?;
    let value: Value = serde_json::from_slice(&bytes)?;
    parse_authorization_context_v1(&value)
        .map_err(|error| TrellisClientError::Bootstrap(error.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state(generation: u64, manifest_digest: &str) -> AuthorizationClientState {
        AuthorizationClientState {
            format: AUTHORIZATION_CLIENT_STATE_FORMAT_V1.into(),
            binding: "service:dep:instance".into(),
            trust: AuthorizationClientTrustState {
                format: "trellis.authorization-client-trust.v1".into(),
                authority: "trellis-test".into(),
                root_key_id: "root-key".into(),
                root_digest: "root-digest".into(),
                minimum_manifest_generation: generation,
                manifest_digest_at_minimum_generation: manifest_digest.into(),
            },
            session: AuthorizationSessionBinding {
                session_id: "ses_test".into(),
                participant_digest: "participant".into(),
                needs_digest: "needs".into(),
            },
            context: None,
            routing: None,
        }
    }

    #[test]
    fn authorization_locators_are_same_origin_canonical_well_known_paths() {
        let base = reqwest::Url::parse("https://trellis.test/base").expect("base URL");
        let manifest = "/.well-known/trellis/authorization/trust/manifest.7";
        let certificate =
            "/.well-known/trellis/authorization/trust/certificate.issuer_key.digest_7";
        assert_eq!(
            resolve_authorization_locator(&base, manifest, AuthorizationLocatorKind::Manifest)
                .expect("relative manifest")
                .as_str(),
            "https://trellis.test/.well-known/trellis/authorization/trust/manifest.7"
        );
        assert!(resolve_authorization_locator(
            &base,
            "https://trellis.test/.well-known/trellis/authorization/trust/manifest.7",
            AuthorizationLocatorKind::Manifest
        )
        .is_ok());
        for locator in [
            "https://attacker.test/.well-known/trellis/authorization/trust/manifest.7",
            "data:application/json,{}",
            "//trellis.test/.well-known/trellis/authorization/trust/manifest.7",
            "/.well-known/trellis/authorization/trust/../trust/manifest.7",
            "/.well-known/trellis/authorization/contexts/manifest.7",
            "/.well-known/trellis/authorization/trust/manifest.7?x=1",
        ] {
            assert!(
                resolve_authorization_locator(&base, locator, AuthorizationLocatorKind::Manifest)
                    .is_err(),
                "accepted {locator}"
            );
        }
        assert!(resolve_authorization_locator(
            &base,
            certificate,
            AuthorizationLocatorKind::Certificate
        )
        .is_ok());
    }

    #[test]
    fn file_store_persists_complete_floor_and_rejects_restart_rollback() {
        let path = std::env::temp_dir().join(format!(
            "trellis-authorization-context-{}.json",
            ulid::Ulid::new()
        ));
        let store = FileAuthorizationContextStore::new(&path);
        store.commit(state(7, "manifest-7")).expect("commit floor");
        store.commit(state(8, "manifest-8")).expect("advance floor");

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
        assert!(reopened.commit(state(7, "manifest-7")).is_err());
        assert!(reopened.commit(state(8, "equivocated")).is_err());
        let mut replaced_root = state(9, "manifest-9");
        replaced_root.trust.root_digest = "another-root".into();
        assert!(reopened.commit(replaced_root).is_err());
        let mut rebound = state(9, "manifest-9");
        rebound.binding = "device:other".into();
        assert!(reopened.commit(rebound).is_err());
        reopened.reset_trust().expect("remove test store");
    }
}
