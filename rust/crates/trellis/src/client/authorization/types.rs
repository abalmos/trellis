use std::fmt;

use super::super::TrellisClientError;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Client authorization state wire format.
pub(crate) const AUTHORIZATION_CLIENT_STATE_FORMAT_V1: &str =
    "trellis.authorization-client-state.v1";

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

/// NATS-backed authorization evidence registry binding distributed with the
/// pinned trust root.
///
/// The binding is internal runtime/SDK material: service authors never receive
/// raw registry handles or subject names.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthorizationRegistryBinding {
    /// KV bucket holding immutable trust records.
    pub trust_bucket: String,
    /// KV bucket holding contexts and revocations.
    pub context_bucket: String,
}

#[cfg(feature = "runtime-internals")]
impl AuthorizationRegistryBinding {
    #[doc(hidden)]
    #[must_use]
    pub fn from_runtime_parts(trust_bucket: String, context_bucket: String) -> Self {
        Self {
            trust_bucket,
            context_bucket,
        }
    }
}

/// Pinned root plus the complete current verification chain.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationTrustBundle {
    /// Pinned public trust root.
    pub root: Value,
    /// Complete canonical issuer manifest embedded for local verification.
    pub manifest: Value,
    /// NATS-backed authorization evidence registry binding.
    pub(crate) authorization_registry: AuthorizationRegistryBinding,
    /// Verification policy bound to this runtime configuration.
    pub policy: AuthorizationTrustPolicy,
}

/// Signed authorization context and its minimal trust metadata.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthorizationContextBundle {
    /// Complete signed authorization context.
    pub context: Value,
    /// Pinned root and embedded verification chain.
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

/// Verified current-context material held by the own-context cache.
#[derive(Clone, Debug)]
pub(crate) struct CurrentContext {
    pub(crate) bundle: AuthorizationContextBundle,
    pub(crate) context_digest: String,
    pub(crate) manifest_generation: u64,
    pub(crate) session_id: String,
    pub(crate) participant_digest: String,
    pub(crate) needs_digest: String,
    pub(crate) not_before: i64,
    pub(crate) expires_at: i64,
    pub(crate) refresh_at: i64,
}

/// In-process own-context state.
#[derive(Clone, Debug, Default)]
pub(crate) struct CachedAuthorizationState {
    pub(crate) current: Option<CurrentContext>,
    pub(crate) session: Option<AuthorizationSessionBinding>,
    pub(crate) routing: Option<AuthorizationRoutingMaterial>,
}
