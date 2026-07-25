use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::{digest_json, GrantSetV1};

use super::domain::{
    require_digest, require_nonempty, require_positive, require_protocol_timestamp,
};
use super::AuthorizationStateError;

pub(super) const BROWSER_FLOW_FORMAT: &str = "trellis.auth-browser-flow.v1";
const OAUTH_STATE_FORMAT: &str = "trellis.auth-oauth-state.v1";

#[cfg(feature = "nats-leases")]
const BROWSER_FLOW_BUCKET: &str = "trellis_auth_browser_flows";
#[cfg(feature = "nats-leases")]
const OAUTH_STATE_BUCKET: &str = "trellis_auth_oauth_states";
#[cfg(feature = "nats-leases")]
const CONNECT_REPLAY_BUCKET: &str = "trellis_auth_connect_replay";
#[cfg(feature = "nats-leases")]
const CONNECTIONS_BUCKET: &str = "trellis_auth_connections";

/// Browser authentication flow kind.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthBrowserFlowKind {
    UserAuth,
    DeviceActivation,
}

/// Browser authentication flow state.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthBrowserFlowState {
    ChooseProvider,
    Authenticated,
    ApprovalRequired,
    ApprovalDenied,
    Approved,
    Consumed,
    Expired,
}

/// Server-owned authority and consent proposal bound to a browser flow.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct BrowserConsentProposal {
    pub participant_id: String,
    pub participant_artifact_digest: String,
    pub participant_needs_digest: String,
    pub consent_view: Value,
    pub consent_view_digest: String,
    pub proposal_digest: String,
    pub required_grant_set: GrantSetV1,
    pub optional_grant_bundles: BTreeMap<String, GrantSetV1>,
    pub required_capabilities: Vec<String>,
    pub optional_capability_definitions: BTreeMap<String, GrantSetV1>,
}

impl BrowserConsentProposal {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        participant_id: String,
        participant_artifact_digest: String,
        participant_needs_digest: String,
        consent_view: Value,
        required_grant_set: GrantSetV1,
        optional_grant_bundles: BTreeMap<String, GrantSetV1>,
        mut required_capabilities: Vec<String>,
        optional_capability_definitions: BTreeMap<String, GrantSetV1>,
    ) -> Result<Self, AuthorizationStateError> {
        required_capabilities.sort();
        required_capabilities.dedup();
        let consent_view_digest = digest_json(&consent_view)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proposal_digest = digest_json(&serde_json::json!({
            "participantId": participant_id,
            "participantArtifactDigest": participant_artifact_digest,
            "participantNeedsDigest": participant_needs_digest,
            "requiredGrantSet": required_grant_set,
            "optionalGrantBundles": optional_grant_bundles,
            "requiredCapabilities": required_capabilities,
            "optionalCapabilityDefinitions": optional_capability_definitions,
        }))
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        let proposal = Self {
            participant_id,
            participant_artifact_digest,
            participant_needs_digest,
            consent_view,
            consent_view_digest,
            proposal_digest,
            required_grant_set,
            optional_grant_bundles,
            required_capabilities,
            optional_capability_definitions,
        };
        proposal.validate()?;
        Ok(proposal)
    }

    fn validate(&self) -> Result<(), AuthorizationStateError> {
        require_nonempty("consent participantId", &self.participant_id)?;
        require_digest(
            "consent participantArtifactDigest",
            &self.participant_artifact_digest,
        )?;
        require_digest(
            "consent participantNeedsDigest",
            &self.participant_needs_digest,
        )?;
        require_digest("consentViewDigest", &self.consent_view_digest)?;
        require_digest("proposalDigest", &self.proposal_digest)?;
        if digest_json(&self.consent_view)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            != self.consent_view_digest
        {
            return invalid("consentViewDigest does not match consentView");
        }
        for bundle_id in self.optional_grant_bundles.keys() {
            require_nonempty("optionalGrantBundles key", bundle_id)?;
        }
        for capability in &self.required_capabilities {
            require_nonempty("consent capability", capability)?;
        }
        for capability_id in self.optional_capability_definitions.keys() {
            require_nonempty("optional capability definition", capability_id)?;
        }
        let machine_value = serde_json::json!({
            "participantId": self.participant_id,
            "participantArtifactDigest": self.participant_artifact_digest,
            "participantNeedsDigest": self.participant_needs_digest,
            "requiredGrantSet": self.required_grant_set,
            "optionalGrantBundles": self.optional_grant_bundles,
            "requiredCapabilities": self.required_capabilities,
            "optionalCapabilityDefinitions": self.optional_capability_definitions,
        });
        if digest_json(&machine_value)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?
            != self.proposal_digest
        {
            return invalid("proposalDigest does not match server-owned authority");
        }
        Ok(())
    }
}

/// Complete ephemeral browser authentication flow record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthBrowserFlow {
    pub format: String,
    pub flow_id: String,
    pub kind: AuthBrowserFlowKind,
    pub state: AuthBrowserFlowState,
    pub request_id: String,
    pub request_digest: String,
    pub participant_id: String,
    pub participant_artifact_digest: String,
    pub participant_needs_digest: String,
    pub consent: BrowserConsentProposal,
    pub session_public_key: String,
    pub session_nkey: String,
    pub portal_id: String,
    pub redirect_target: Option<String>,
    pub principal_id: Option<String>,
    pub claim_owner: Option<String>,
    pub claimed_at: Option<i64>,
    pub durable_result_digest: Option<String>,
    pub completed_at: Option<i64>,
    pub created_at: i64,
    pub expires_at: i64,
    pub version: u64,
}

impl AuthBrowserFlow {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        require_format("format", &self.format, BROWSER_FLOW_FORMAT)?;
        require_nonempty("flowId", &self.flow_id)?;
        require_nonempty("requestId", &self.request_id)?;
        require_digest("requestDigest", &self.request_digest)?;
        require_nonempty("participantId", &self.participant_id)?;
        require_digest(
            "participantArtifactDigest",
            &self.participant_artifact_digest,
        )?;
        require_digest("participantNeedsDigest", &self.participant_needs_digest)?;
        self.consent.validate()?;
        if self.consent.participant_id != self.participant_id
            || self.consent.participant_artifact_digest != self.participant_artifact_digest
            || self.consent.participant_needs_digest != self.participant_needs_digest
        {
            return invalid("consent proposal does not match participant binding");
        }
        require_nonempty("sessionPublicKey", &self.session_public_key)?;
        require_nonempty("sessionNkey", &self.session_nkey)?;
        require_nonempty("portalId", &self.portal_id)?;
        validate_optional_text("redirectTarget", self.redirect_target.as_deref())?;
        validate_optional_text("principalId", self.principal_id.as_deref())?;
        validate_optional_text("claimOwner", self.claim_owner.as_deref())?;
        if let Some(value) = self.claimed_at {
            require_protocol_timestamp("claimedAt", value)?;
        }
        if let Some(value) = self.durable_result_digest.as_deref() {
            require_digest("durableResultDigest", value)?;
        }
        if let Some(value) = self.completed_at {
            require_protocol_timestamp("completedAt", value)?;
        }
        require_protocol_timestamp("createdAt", self.created_at)?;
        require_protocol_timestamp("expiresAt", self.expires_at)?;
        require_positive("version", self.version)?;
        if self.expires_at < self.created_at {
            return invalid("expiresAt precedes createdAt");
        }
        for (field, value) in [
            ("claimedAt", self.claimed_at),
            ("completedAt", self.completed_at),
        ] {
            if value.is_some_and(|value| value < self.created_at || value > self.expires_at) {
                return invalid(format!("{field} must be between createdAt and expiresAt"));
            }
        }
        if self.claim_owner.is_some() != self.claimed_at.is_some() {
            return invalid("claimOwner and claimedAt must both be null or both be set");
        }
        let principal = self.principal_id.is_some();
        let claim = self.claim_owner.is_some();
        let result = self.durable_result_digest.is_some();
        let completed = self.completed_at.is_some();
        let valid = match self.state {
            AuthBrowserFlowState::ChooseProvider => !principal && !claim && !result && !completed,
            AuthBrowserFlowState::Authenticated | AuthBrowserFlowState::ApprovalRequired => {
                principal && !claim && !result && !completed
            }
            AuthBrowserFlowState::ApprovalDenied => principal && !claim && !result && completed,
            AuthBrowserFlowState::Approved => principal && !claim && result && completed,
            AuthBrowserFlowState::Consumed => principal && claim && result && completed,
            AuthBrowserFlowState::Expired => !claim && !result && completed,
        };
        if !valid {
            return invalid("browser flow claim/result fields do not match state");
        }
        Ok(())
    }

    fn preserves_transcript(&self, replacement: &Self) -> bool {
        self.format == replacement.format
            && self.flow_id == replacement.flow_id
            && self.kind == replacement.kind
            && self.request_id == replacement.request_id
            && self.request_digest == replacement.request_digest
            && self.participant_id == replacement.participant_id
            && self.participant_artifact_digest == replacement.participant_artifact_digest
            && self.participant_needs_digest == replacement.participant_needs_digest
            && self.consent == replacement.consent
            && self.session_public_key == replacement.session_public_key
            && self.session_nkey == replacement.session_nkey
            && self.portal_id == replacement.portal_id
            && self.redirect_target == replacement.redirect_target
            && self.created_at == replacement.created_at
            && self.expires_at == replacement.expires_at
    }
}

/// OAuth callback state lifecycle.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthOAuthStatus {
    Pending,
    Claimed,
    ExchangeStarted,
    Completed,
    RestartRequired,
    Expired,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AuthOAuthKind {
    Browser,
    AccountFlow,
}

/// Complete ephemeral OAuth callback state record.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthOAuthState {
    pub format: String,
    pub state_id: String,
    pub provider_id: String,
    pub kind: AuthOAuthKind,
    pub flow_id: String,
    pub status: AuthOAuthStatus,
    pub pkce_verifier: String,
    pub nonce: String,
    pub redirect_uri: String,
    pub browser_binding_digest: String,
    pub portal_id: Option<String>,
    pub portal_policy_digest: Option<String>,
    pub claim_owner: Option<String>,
    pub result_digest: Option<String>,
    pub created_at: i64,
    pub expires_at: i64,
    pub version: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ConnectReplayRecord {
    pub format: String,
    pub purpose: String,
    pub signer_key_id: String,
    pub request_id: String,
    pub transcript_digest: String,
    pub admitted_at: i64,
    pub expires_at: i64,
    pub version: u64,
}

impl ConnectReplayRecord {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        require_format("format", &self.format, "trellis.session-proof-replay.v1")?;
        require_format("purpose", &self.purpose, "natsConnectContext")?;
        require_digest("signerKeyId", &self.signer_key_id)?;
        require_nonempty("requestId", &self.request_id)?;
        require_digest("transcriptDigest", &self.transcript_digest)?;
        require_protocol_timestamp("admittedAt", self.admitted_at)?;
        require_protocol_timestamp("expiresAt", self.expires_at)?;
        if self.expires_at <= self.admitted_at || self.version != 1 {
            return invalid("connect replay expiry or version is invalid");
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct AuthConnectionPresence {
    pub format: String,
    pub connection_id: String,
    pub session_id: String,
    pub context_id: String,
    pub context_digest: String,
    pub server_id: String,
    pub client_id: String,
    pub user_nkey: String,
    pub remote_address: Option<String>,
    pub connected_at: i64,
    pub last_seen_at: i64,
    pub version: u64,
}

impl AuthConnectionPresence {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        require_format(
            "format",
            &self.format,
            "trellis.auth-connection-presence.v1",
        )?;
        require_digest("connectionId", &self.connection_id)?;
        require_nonempty("sessionId", &self.session_id)?;
        require_nonempty("contextId", &self.context_id)?;
        require_digest("contextDigest", &self.context_digest)?;
        require_nonempty("serverId", &self.server_id)?;
        require_nonempty("clientId", &self.client_id)?;
        require_nonempty("userNkey", &self.user_nkey)?;
        validate_optional_text("remoteAddress", self.remote_address.as_deref())?;
        require_protocol_timestamp("connectedAt", self.connected_at)?;
        require_protocol_timestamp("lastSeenAt", self.last_seen_at)?;
        if self.last_seen_at < self.connected_at || self.version != 1 {
            return invalid("connection presence timestamps or version are invalid");
        }
        Ok(())
    }
}

impl AuthOAuthState {
    fn validate(&self) -> Result<(), AuthorizationStateError> {
        require_format("format", &self.format, OAUTH_STATE_FORMAT)?;
        require_nonempty("stateId", &self.state_id)?;
        require_nonempty("providerId", &self.provider_id)?;
        require_nonempty("flowId", &self.flow_id)?;
        require_nonempty("pkceVerifier", &self.pkce_verifier)?;
        require_nonempty("nonce", &self.nonce)?;
        require_nonempty("redirectUri", &self.redirect_uri)?;
        require_digest("browserBindingDigest", &self.browser_binding_digest)?;
        validate_optional_text("portalId", self.portal_id.as_deref())?;
        if let Some(value) = self.portal_policy_digest.as_deref() {
            require_digest("portalPolicyDigest", value)?;
        }
        if self.portal_id.is_some() != self.portal_policy_digest.is_some() {
            return invalid("OAuth portal ID and policy digest must both be present or absent");
        }
        validate_optional_text("claimOwner", self.claim_owner.as_deref())?;
        if let Some(value) = self.result_digest.as_deref() {
            require_digest("resultDigest", value)?;
        }
        require_protocol_timestamp("createdAt", self.created_at)?;
        require_protocol_timestamp("expiresAt", self.expires_at)?;
        require_positive("version", self.version)?;
        if self.expires_at < self.created_at {
            return invalid("expiresAt precedes createdAt");
        }

        let claimed = self.claim_owner.is_some();
        let result = self.result_digest.is_some();
        let valid = match self.status {
            AuthOAuthStatus::Pending => !claimed && !result,
            AuthOAuthStatus::Claimed
            | AuthOAuthStatus::ExchangeStarted
            | AuthOAuthStatus::RestartRequired => claimed && !result,
            AuthOAuthStatus::Completed => claimed && result,
            AuthOAuthStatus::Expired => !result,
        };
        if !valid {
            return invalid("OAuth claim/result fields do not match status");
        }
        Ok(())
    }

    fn preserves_transcript(&self, replacement: &Self) -> bool {
        self.format == replacement.format
            && self.state_id == replacement.state_id
            && self.provider_id == replacement.provider_id
            && self.kind == replacement.kind
            && self.flow_id == replacement.flow_id
            && self.pkce_verifier == replacement.pkce_verifier
            && self.nonce == replacement.nonce
            && self.redirect_uri == replacement.redirect_uri
            && self.browser_binding_digest == replacement.browser_binding_digest
            && self.portal_id == replacement.portal_id
            && self.portal_policy_digest == replacement.portal_policy_digest
            && self.created_at == replacement.created_at
            && self.expires_at == replacement.expires_at
    }
}

/// Typed repository port for ephemeral browser and OAuth auth state.
#[async_trait]
pub(crate) trait AuthEphemeralRepository: Send + Sync {
    async fn create_browser_flow(
        &self,
        record: AuthBrowserFlow,
    ) -> Result<(), AuthorizationStateError>;
    async fn get_browser_flow(
        &self,
        flow_id: &str,
    ) -> Result<Option<AuthBrowserFlow>, AuthorizationStateError>;
    async fn replace_browser_flow(
        &self,
        expected_version: u64,
        replacement: AuthBrowserFlow,
    ) -> Result<(), AuthorizationStateError>;

    async fn create_oauth_state(
        &self,
        record: AuthOAuthState,
    ) -> Result<(), AuthorizationStateError>;
    async fn get_oauth_state(
        &self,
        state_id: &str,
    ) -> Result<Option<AuthOAuthState>, AuthorizationStateError>;
    async fn replace_oauth_state(
        &self,
        expected_version: u64,
        replacement: AuthOAuthState,
    ) -> Result<(), AuthorizationStateError>;
    async fn admit_connect_replay(
        &self,
        record: ConnectReplayRecord,
    ) -> Result<bool, AuthorizationStateError>;
    async fn put_connection_presence(
        &self,
        record: AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError>;
    async fn delete_connection_presence(
        &self,
        user_nkey: &str,
    ) -> Result<(), AuthorizationStateError>;
    async fn list_connection_presence(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<AuthConnectionPresence>, AuthorizationStateError>;
}

/// Atomically claims a pending OAuth callback state.
pub(crate) async fn claim_oauth_state(
    repository: &impl AuthEphemeralRepository,
    state_id: &str,
    claim_owner: &str,
) -> Result<AuthOAuthState, AuthorizationStateError> {
    require_nonempty("claimOwner", claim_owner)?;
    let current = repository
        .get_oauth_state(state_id)
        .await?
        .ok_or(AuthorizationStateError::StorageConflict)?;
    if current.status != AuthOAuthStatus::Pending {
        return Err(AuthorizationStateError::StorageConflict);
    }
    let mut replacement = current.clone();
    replacement.status = AuthOAuthStatus::Claimed;
    replacement.claim_owner = Some(claim_owner.to_owned());
    replacement.version = next_version(current.version)?;
    repository
        .replace_oauth_state(current.version, replacement.clone())
        .await?;
    Ok(replacement)
}

/// Constraint-faithful in-memory ephemeral auth repository.
#[cfg(test)]
#[derive(Clone, Debug, Default)]
pub(crate) struct InMemoryAuthEphemeralRepository {
    browser_flows: Arc<Mutex<BTreeMap<String, AuthBrowserFlow>>>,
    oauth_states: Arc<Mutex<BTreeMap<String, AuthOAuthState>>>,
    connect_replays: Arc<Mutex<BTreeMap<String, ConnectReplayRecord>>>,
    connections: Arc<Mutex<BTreeMap<String, AuthConnectionPresence>>>,
}

#[cfg(test)]
#[async_trait]
impl AuthEphemeralRepository for InMemoryAuthEphemeralRepository {
    async fn create_browser_flow(
        &self,
        record: AuthBrowserFlow,
    ) -> Result<(), AuthorizationStateError> {
        validate_create(record.version, || record.validate())?;
        let mut records = lock(&self.browser_flows)?;
        if records.contains_key(&record.flow_id) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        records.insert(record.flow_id.clone(), record);
        Ok(())
    }

    async fn get_browser_flow(
        &self,
        flow_id: &str,
    ) -> Result<Option<AuthBrowserFlow>, AuthorizationStateError> {
        Ok(lock(&self.browser_flows)?.get(flow_id).cloned())
    }

    async fn replace_browser_flow(
        &self,
        expected_version: u64,
        replacement: AuthBrowserFlow,
    ) -> Result<(), AuthorizationStateError> {
        validate_replacement_version(expected_version, replacement.version)?;
        let mut records = lock(&self.browser_flows)?;
        let current = records
            .get(&replacement.flow_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        validate_browser_replacement(current, expected_version, &replacement)?;
        replacement.validate()?;
        records.insert(replacement.flow_id.clone(), replacement);
        Ok(())
    }

    async fn create_oauth_state(
        &self,
        record: AuthOAuthState,
    ) -> Result<(), AuthorizationStateError> {
        validate_create(record.version, || record.validate())?;
        let mut records = lock(&self.oauth_states)?;
        if records.contains_key(&record.state_id) {
            return Err(AuthorizationStateError::StorageConflict);
        }
        records.insert(record.state_id.clone(), record);
        Ok(())
    }

    async fn get_oauth_state(
        &self,
        state_id: &str,
    ) -> Result<Option<AuthOAuthState>, AuthorizationStateError> {
        Ok(lock(&self.oauth_states)?.get(state_id).cloned())
    }

    async fn replace_oauth_state(
        &self,
        expected_version: u64,
        replacement: AuthOAuthState,
    ) -> Result<(), AuthorizationStateError> {
        replacement.validate()?;
        validate_replacement_version(expected_version, replacement.version)?;
        let mut records = lock(&self.oauth_states)?;
        let current = records
            .get(&replacement.state_id)
            .ok_or(AuthorizationStateError::StorageConflict)?;
        validate_oauth_replacement(current, expected_version, &replacement)?;
        records.insert(replacement.state_id.clone(), replacement);
        Ok(())
    }

    async fn admit_connect_replay(
        &self,
        record: ConnectReplayRecord,
    ) -> Result<bool, AuthorizationStateError> {
        record.validate()?;
        let key = replay_key(&record.signer_key_id, &record.request_id)?;
        let mut records = lock(&self.connect_replays)?;
        records.retain(|_, current| current.expires_at > record.admitted_at);
        if records.contains_key(&key) {
            return Ok(false);
        }
        records.insert(key, record);
        Ok(true)
    }

    async fn put_connection_presence(
        &self,
        record: AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError> {
        record.validate()?;
        lock(&self.connections)?.insert(record.user_nkey.clone(), record);
        Ok(())
    }

    async fn delete_connection_presence(
        &self,
        user_nkey: &str,
    ) -> Result<(), AuthorizationStateError> {
        lock(&self.connections)?.remove(user_nkey);
        Ok(())
    }

    async fn list_connection_presence(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<AuthConnectionPresence>, AuthorizationStateError> {
        Ok(lock(&self.connections)?
            .values()
            .filter(|record| session_id.is_none_or(|id| record.session_id == id))
            .cloned()
            .collect())
    }
}

fn replay_key(signer_key_id: &str, request_id: &str) -> Result<String, AuthorizationStateError> {
    trellis_protocol::digest_json(&serde_json::json!({
        "signerKeyId": signer_key_id,
        "requestId": request_id,
    }))
    .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))
}

#[cfg(test)]
fn lock<T>(value: &Mutex<T>) -> Result<std::sync::MutexGuard<'_, T>, AuthorizationStateError> {
    value
        .lock()
        .map_err(|_| AuthorizationStateError::Storage("in-memory lock poisoned".to_owned()))
}

fn validate_create(
    version: u64,
    validate: impl FnOnce() -> Result<(), AuthorizationStateError>,
) -> Result<(), AuthorizationStateError> {
    validate()?;
    if version != 1 {
        return invalid("new ephemeral auth records must have version 1");
    }
    Ok(())
}

fn validate_replacement_version(
    expected_version: u64,
    replacement_version: u64,
) -> Result<(), AuthorizationStateError> {
    if next_version(expected_version)? != replacement_version {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

fn validate_browser_replacement(
    current: &AuthBrowserFlow,
    expected_version: u64,
    replacement: &AuthBrowserFlow,
) -> Result<(), AuthorizationStateError> {
    if current.version != expected_version
        || !current.preserves_transcript(replacement)
        || !valid_browser_transition(current.state, replacement.state)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

fn validate_oauth_replacement(
    current: &AuthOAuthState,
    expected_version: u64,
    replacement: &AuthOAuthState,
) -> Result<(), AuthorizationStateError> {
    if current.version != expected_version
        || !current.preserves_transcript(replacement)
        || !valid_oauth_transition(current.status, replacement.status)
    {
        return Err(AuthorizationStateError::StorageConflict);
    }
    Ok(())
}

fn valid_browser_transition(current: AuthBrowserFlowState, next: AuthBrowserFlowState) -> bool {
    matches!(
        (current, next),
        (
            AuthBrowserFlowState::ChooseProvider,
            AuthBrowserFlowState::Authenticated | AuthBrowserFlowState::Expired
        ) | (
            AuthBrowserFlowState::Authenticated,
            AuthBrowserFlowState::ApprovalRequired
                | AuthBrowserFlowState::Approved
                | AuthBrowserFlowState::Expired
        ) | (
            AuthBrowserFlowState::ApprovalRequired,
            AuthBrowserFlowState::Approved
                | AuthBrowserFlowState::ApprovalDenied
                | AuthBrowserFlowState::Expired
        ) | (
            AuthBrowserFlowState::Approved,
            AuthBrowserFlowState::Consumed | AuthBrowserFlowState::Expired
        )
    )
}

fn valid_oauth_transition(current: AuthOAuthStatus, next: AuthOAuthStatus) -> bool {
    matches!(
        (current, next),
        (
            AuthOAuthStatus::Pending,
            AuthOAuthStatus::Claimed | AuthOAuthStatus::Expired
        ) | (
            AuthOAuthStatus::Claimed,
            AuthOAuthStatus::ExchangeStarted | AuthOAuthStatus::Expired
        ) | (
            AuthOAuthStatus::ExchangeStarted,
            AuthOAuthStatus::Completed | AuthOAuthStatus::RestartRequired
        ) | (AuthOAuthStatus::RestartRequired, AuthOAuthStatus::Expired)
    )
}

fn next_version(version: u64) -> Result<u64, AuthorizationStateError> {
    let version = version
        .checked_add(1)
        .ok_or(AuthorizationStateError::StorageConflict)?;
    require_positive("version", version)?;
    Ok(version)
}

fn require_format(
    field: &str,
    actual: &str,
    expected: &str,
) -> Result<(), AuthorizationStateError> {
    if actual != expected {
        return invalid(format!("{field} must be {expected}"));
    }
    Ok(())
}

fn validate_optional_text(field: &str, value: Option<&str>) -> Result<(), AuthorizationStateError> {
    if let Some(value) = value {
        require_nonempty(field, value)?;
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(message.into()))
}

#[cfg(feature = "nats-leases")]
mod nats {
    use std::error::Error as StdError;
    use std::time::Duration;

    use async_nats::jetstream::{self, context, kv};
    use bytes::Bytes;
    use futures_util::StreamExt;

    use super::*;

    /// NATS KV implementation of ephemeral auth storage.
    #[derive(Clone, Debug)]
    pub(crate) struct NatsAuthEphemeralRepository {
        browser_flows: kv::Store,
        oauth_states: kv::Store,
        connect_replays: kv::Store,
        connections: kv::Store,
    }

    impl NatsAuthEphemeralRepository {
        /// Opens or creates and validates both auth-owned KV buckets.
        pub(crate) async fn ensure(
            client: async_nats::Client,
        ) -> Result<Self, AuthorizationStateError> {
            let jetstream = jetstream::new(client);
            let browser_flows = open_or_create(
                &jetstream,
                BROWSER_FLOW_BUCKET,
                Duration::from_millis(86_400_000),
                65_536,
            )
            .await?;
            let oauth_states = open_or_create(
                &jetstream,
                OAUTH_STATE_BUCKET,
                Duration::from_millis(900_000),
                16_384,
            )
            .await?;
            let connect_replays = open_or_create(
                &jetstream,
                CONNECT_REPLAY_BUCKET,
                Duration::from_millis(660_000),
                4_096,
            )
            .await?;
            let connections = open_or_create(
                &jetstream,
                CONNECTIONS_BUCKET,
                Duration::from_millis(120_000),
                16_384,
            )
            .await?;
            Ok(Self {
                browser_flows,
                oauth_states,
                connect_replays,
                connections,
            })
        }

        /// Validates all required auth-owned KV buckets without creating or updating them.
        pub(crate) async fn check(
            client: async_nats::Client,
        ) -> Result<(), AuthorizationStateError> {
            let jetstream = jetstream::new(client);
            for (bucket, max_age, max_value_size) in [
                (
                    BROWSER_FLOW_BUCKET,
                    Duration::from_millis(86_400_000),
                    65_536,
                ),
                (OAUTH_STATE_BUCKET, Duration::from_millis(900_000), 16_384),
                (CONNECT_REPLAY_BUCKET, Duration::from_millis(660_000), 4_096),
                (CONNECTIONS_BUCKET, Duration::from_millis(120_000), 16_384),
            ] {
                let store = jetstream.get_key_value(bucket).await.map_err(|error| {
                    storage(format!(
                        "required auth KV bucket {bucket} is missing: {error}"
                    ))
                })?;
                validate_bucket(&store, max_age, max_value_size).await?;
            }
            Ok(())
        }
    }

    #[async_trait]
    impl AuthEphemeralRepository for NatsAuthEphemeralRepository {
        async fn create_browser_flow(
            &self,
            record: AuthBrowserFlow,
        ) -> Result<(), AuthorizationStateError> {
            validate_create(record.version, || record.validate())?;
            create(&self.browser_flows, &record.flow_id, &record).await
        }

        async fn get_browser_flow(
            &self,
            flow_id: &str,
        ) -> Result<Option<AuthBrowserFlow>, AuthorizationStateError> {
            get(&self.browser_flows, flow_id).await
        }

        async fn replace_browser_flow(
            &self,
            expected_version: u64,
            replacement: AuthBrowserFlow,
        ) -> Result<(), AuthorizationStateError> {
            validate_replacement_version(expected_version, replacement.version)?;
            let entry = current_entry(&self.browser_flows, &replacement.flow_id).await?;
            let current = decode::<AuthBrowserFlow>(&entry.value)?;
            validate_browser_replacement(&current, expected_version, &replacement)?;
            replacement.validate()?;
            update(
                &self.browser_flows,
                &replacement.flow_id,
                &replacement,
                entry.revision,
            )
            .await
        }

        async fn create_oauth_state(
            &self,
            record: AuthOAuthState,
        ) -> Result<(), AuthorizationStateError> {
            validate_create(record.version, || record.validate())?;
            create(&self.oauth_states, &record.state_id, &record).await
        }

        async fn get_oauth_state(
            &self,
            state_id: &str,
        ) -> Result<Option<AuthOAuthState>, AuthorizationStateError> {
            get(&self.oauth_states, state_id).await
        }

        async fn replace_oauth_state(
            &self,
            expected_version: u64,
            replacement: AuthOAuthState,
        ) -> Result<(), AuthorizationStateError> {
            replacement.validate()?;
            validate_replacement_version(expected_version, replacement.version)?;
            let entry = current_entry(&self.oauth_states, &replacement.state_id).await?;
            let current = decode::<AuthOAuthState>(&entry.value)?;
            validate_oauth_replacement(&current, expected_version, &replacement)?;
            update(
                &self.oauth_states,
                &replacement.state_id,
                &replacement,
                entry.revision,
            )
            .await
        }

        async fn admit_connect_replay(
            &self,
            record: ConnectReplayRecord,
        ) -> Result<bool, AuthorizationStateError> {
            record.validate()?;
            let key = replay_key(&record.signer_key_id, &record.request_id)?;
            match create(&self.connect_replays, &key, &record).await {
                Ok(()) => Ok(true),
                Err(AuthorizationStateError::StorageConflict) => Ok(false),
                Err(error) => Err(error),
            }
        }

        async fn put_connection_presence(
            &self,
            record: AuthConnectionPresence,
        ) -> Result<(), AuthorizationStateError> {
            record.validate()?;
            self.connections
                .put(record.user_nkey.clone(), encode(&record)?)
                .await
                .map(|_| ())
                .map_err(|error| storage(format!("failed to write connection presence: {error}")))
        }

        async fn delete_connection_presence(
            &self,
            user_nkey: &str,
        ) -> Result<(), AuthorizationStateError> {
            self.connections
                .delete(user_nkey)
                .await
                .map_err(|error| storage(format!("failed to delete connection presence: {error}")))
        }

        async fn list_connection_presence(
            &self,
            session_id: Option<&str>,
        ) -> Result<Vec<AuthConnectionPresence>, AuthorizationStateError> {
            let mut keys =
                self.connections.keys().await.map_err(|error| {
                    storage(format!("failed to list connection presence: {error}"))
                })?;
            let mut records = Vec::new();
            while let Some(key) = keys.next().await {
                let key = key.map_err(|error| {
                    storage(format!("failed to read connection presence key: {error}"))
                })?;
                if let Some(record) = get::<AuthConnectionPresence>(&self.connections, &key).await?
                {
                    if session_id.is_none_or(|id| record.session_id == id) {
                        records.push(record);
                    }
                }
            }
            records.sort_by(|left, right| left.connection_id.cmp(&right.connection_id));
            Ok(records)
        }
    }

    async fn open_or_create(
        jetstream: &jetstream::Context,
        bucket: &str,
        max_age: Duration,
        max_value_size: i32,
    ) -> Result<kv::Store, AuthorizationStateError> {
        let config = kv::Config {
            bucket: bucket.to_owned(),
            history: 1,
            max_age,
            max_value_size,
            ..Default::default()
        };
        let store = match jetstream.get_key_value(bucket).await {
            Ok(store) => store,
            Err(open_error) => match jetstream.create_key_value(config).await {
                Ok(store) => store,
                Err(create_error) if is_bucket_create_race(&create_error) => {
                    jetstream.get_key_value(bucket).await.map_err(|error| {
                        storage(format!(
                            "failed to open {bucket} after concurrent create: {error}"
                        ))
                    })?
                }
                Err(create_error) => {
                    return Err(storage(format!(
                        "failed to open {bucket} ({open_error}) or create it ({create_error})"
                    )))
                }
            },
        };
        validate_bucket(&store, max_age, max_value_size).await?;
        Ok(store)
    }

    async fn validate_bucket(
        store: &kv::Store,
        max_age: Duration,
        max_value_size: i32,
    ) -> Result<(), AuthorizationStateError> {
        let status = store
            .status()
            .await
            .map_err(|error| storage(format!("failed to inspect {}: {error}", store.name)))?;
        let actual_max_value_size = status.info.config.max_message_size;
        for (field, expected, actual) in [
            ("history", "1".to_owned(), status.history().to_string()),
            (
                "max_age",
                format!("{}ms", max_age.as_millis()),
                format!("{}ms", status.max_age().as_millis()),
            ),
            (
                "max_value_size",
                max_value_size.to_string(),
                actual_max_value_size.to_string(),
            ),
        ] {
            if expected != actual {
                return Err(storage(format!(
                    "bucket {} has incompatible {field}: expected {expected}, actual {actual}",
                    store.name
                )));
            }
        }
        Ok(())
    }

    async fn create<T: Serialize>(
        store: &kv::Store,
        key: &str,
        record: &T,
    ) -> Result<(), AuthorizationStateError> {
        match store.create(key, encode(record)?).await {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == kv::CreateErrorKind::AlreadyExists => {
                Err(AuthorizationStateError::StorageConflict)
            }
            Err(error) => Err(storage(format!(
                "failed to create {} key {key}: {error}",
                store.name
            ))),
        }
    }

    async fn get<T: for<'de> Deserialize<'de> + Validate>(
        store: &kv::Store,
        key: &str,
    ) -> Result<Option<T>, AuthorizationStateError> {
        match store
            .entry(key)
            .await
            .map_err(|error| storage(format!("failed to read {} key {key}: {error}", store.name)))?
        {
            Some(entry) if entry.operation == kv::Operation::Put => decode(&entry.value).map(Some),
            _ => Ok(None),
        }
    }

    async fn current_entry(
        store: &kv::Store,
        key: &str,
    ) -> Result<kv::Entry, AuthorizationStateError> {
        store
            .entry(key)
            .await
            .map_err(|error| storage(format!("failed to read {} key {key}: {error}", store.name)))?
            .filter(|entry| entry.operation == kv::Operation::Put)
            .ok_or(AuthorizationStateError::StorageConflict)
    }

    async fn update<T: Serialize>(
        store: &kv::Store,
        key: &str,
        record: &T,
        revision: u64,
    ) -> Result<(), AuthorizationStateError> {
        match store.update(key, encode(record)?, revision).await {
            Ok(_) => Ok(()),
            Err(error) if error.kind() == kv::UpdateErrorKind::WrongLastRevision => {
                Err(AuthorizationStateError::StorageConflict)
            }
            Err(error) => Err(storage(format!(
                "failed to update {} key {key}: {error}",
                store.name
            ))),
        }
    }

    fn encode(record: &impl Serialize) -> Result<Bytes, AuthorizationStateError> {
        serde_json::to_vec(record)
            .map(Bytes::from)
            .map_err(|error| storage(format!("failed to encode auth state: {error}")))
    }

    fn decode<T: for<'de> Deserialize<'de> + Validate>(
        value: &[u8],
    ) -> Result<T, AuthorizationStateError> {
        let record: T = serde_json::from_slice(value)
            .map_err(|error| storage(format!("invalid auth state JSON: {error}")))?;
        record.validate_record()?;
        Ok(record)
    }

    trait Validate {
        fn validate_record(&self) -> Result<(), AuthorizationStateError>;
    }

    impl Validate for AuthBrowserFlow {
        fn validate_record(&self) -> Result<(), AuthorizationStateError> {
            self.validate()
        }
    }

    impl Validate for AuthOAuthState {
        fn validate_record(&self) -> Result<(), AuthorizationStateError> {
            self.validate()
        }
    }

    impl Validate for AuthConnectionPresence {
        fn validate_record(&self) -> Result<(), AuthorizationStateError> {
            self.validate()
        }
    }

    fn is_bucket_create_race(error: &context::CreateKeyValueError) -> bool {
        error.kind() == context::CreateKeyValueErrorKind::BucketCreate
            && has_stream_exists_error(error)
    }

    fn has_stream_exists_error(error: &dyn StdError) -> bool {
        let mut source = error.source();
        while let Some(error) = source {
            if let Some(stream_error) = error.downcast_ref::<context::CreateStreamError>() {
                if matches!(
                    stream_error.kind(),
                    context::CreateStreamErrorKind::JetStream(error)
                        if error.kind() == jetstream::ErrorCode::STREAM_NAME_EXIST
                ) {
                    return true;
                }
            }
            if let Some(jetstream_error) = error.downcast_ref::<jetstream::Error>() {
                if jetstream_error.kind() == jetstream::ErrorCode::STREAM_NAME_EXIST {
                    return true;
                }
            }
            source = error.source();
        }
        false
    }

    fn storage(message: impl Into<String>) -> AuthorizationStateError {
        AuthorizationStateError::Storage(message.into())
    }
}

#[cfg(feature = "nats-leases")]
pub(crate) use nats::NatsAuthEphemeralRepository;

#[cfg(test)]
mod tests {
    use std::process::{Child, Command, Stdio};

    use super::*;

    const DIGEST: &str = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";

    fn browser_flow() -> AuthBrowserFlow {
        AuthBrowserFlow {
            format: BROWSER_FLOW_FORMAT.to_owned(),
            flow_id: "flow-1".to_owned(),
            kind: AuthBrowserFlowKind::UserAuth,
            state: AuthBrowserFlowState::ChooseProvider,
            request_id: "request-1".to_owned(),
            request_digest: DIGEST.to_owned(),
            participant_id: "app-1".to_owned(),
            participant_artifact_digest: DIGEST.to_owned(),
            participant_needs_digest: DIGEST.to_owned(),
            consent: BrowserConsentProposal::new(
                "app-1".to_owned(),
                DIGEST.to_owned(),
                DIGEST.to_owned(),
                serde_json::json!({ "title": "Authorize app" }),
                GrantSetV1::new(Vec::new()),
                BTreeMap::new(),
                Vec::new(),
                BTreeMap::new(),
            )
            .unwrap(),
            session_public_key: "session-key".to_owned(),
            session_nkey: "USESSIONKEY".to_owned(),
            portal_id: "builtin".to_owned(),
            redirect_target: Some("https://app.example/callback".to_owned()),
            principal_id: None,
            claim_owner: None,
            claimed_at: None,
            durable_result_digest: None,
            completed_at: None,
            created_at: 100,
            expires_at: 1_000,
            version: 1,
        }
    }

    fn oauth_state() -> AuthOAuthState {
        AuthOAuthState {
            format: OAUTH_STATE_FORMAT.to_owned(),
            state_id: "state-1".to_owned(),
            provider_id: "provider-1".to_owned(),
            kind: AuthOAuthKind::Browser,
            flow_id: "flow-1".to_owned(),
            status: AuthOAuthStatus::Pending,
            pkce_verifier: "pkce-verifier".to_owned(),
            nonce: "nonce".to_owned(),
            redirect_uri: "https://auth.example/callback".to_owned(),
            browser_binding_digest: DIGEST.to_owned(),
            portal_id: Some("builtin".to_owned()),
            portal_policy_digest: Some(DIGEST.to_owned()),
            claim_owner: None,
            result_digest: None,
            created_at: 100,
            expires_at: 1_000,
            version: 1,
        }
    }

    async fn repository_conformance(repository: impl AuthEphemeralRepository + Clone) {
        let flow = browser_flow();
        repository.create_browser_flow(flow.clone()).await.unwrap();
        assert_eq!(
            repository.get_browser_flow(&flow.flow_id).await.unwrap(),
            Some(flow.clone())
        );
        assert_eq!(
            repository.create_browser_flow(flow.clone()).await,
            Err(AuthorizationStateError::StorageConflict)
        );

        let mut authenticated = flow.clone();
        authenticated.state = AuthBrowserFlowState::Authenticated;
        authenticated.principal_id = Some("user-1".to_owned());
        authenticated.version = 2;
        repository
            .replace_browser_flow(1, authenticated.clone())
            .await
            .unwrap();
        assert_eq!(
            repository
                .replace_browser_flow(1, authenticated.clone())
                .await,
            Err(AuthorizationStateError::StorageConflict)
        );
        for changed_consent in [
            {
                let mut consent = authenticated.consent.clone();
                consent.consent_view = serde_json::json!({ "title": "Changed" });
                consent
            },
            {
                let mut consent = authenticated.consent.clone();
                consent.consent_view_digest = DIGEST.replace('A', "B");
                consent
            },
            {
                let mut consent = authenticated.consent.clone();
                consent.proposal_digest = DIGEST.replace('A', "C");
                consent
            },
            {
                let mut consent = authenticated.consent.clone();
                consent
                    .optional_capability_definitions
                    .insert("extra".to_owned(), GrantSetV1::new(Vec::new()));
                consent
            },
        ] {
            let mut changed = authenticated.clone();
            changed.state = AuthBrowserFlowState::ApprovalRequired;
            changed.consent = changed_consent;
            changed.version = 3;
            assert_eq!(
                repository.replace_browser_flow(2, changed).await,
                Err(AuthorizationStateError::StorageConflict)
            );
        }
        let mut changed_transcript = authenticated;
        changed_transcript.request_id = "changed".to_owned();
        changed_transcript.version = 3;
        assert_eq!(
            repository.replace_browser_flow(2, changed_transcript).await,
            Err(AuthorizationStateError::StorageConflict)
        );
        let mut skipped_state = flow.clone();
        skipped_state.flow_id = "flow-skipped".to_owned();
        repository
            .create_browser_flow(skipped_state.clone())
            .await
            .unwrap();
        skipped_state.state = AuthBrowserFlowState::Approved;
        skipped_state.principal_id = Some("user-1".to_owned());
        skipped_state.durable_result_digest = Some(DIGEST.to_owned());
        skipped_state.completed_at = Some(200);
        skipped_state.version = 2;
        assert_eq!(
            repository.replace_browser_flow(1, skipped_state).await,
            Err(AuthorizationStateError::StorageConflict)
        );

        let oauth = oauth_state();
        repository.create_oauth_state(oauth.clone()).await.unwrap();
        assert_eq!(
            repository.get_oauth_state(&oauth.state_id).await.unwrap(),
            Some(oauth)
        );
        let mut skipped_exchange = oauth_state();
        skipped_exchange.status = AuthOAuthStatus::ExchangeStarted;
        skipped_exchange.claim_owner = Some("owner-1".to_owned());
        skipped_exchange.version = 2;
        assert_eq!(
            repository.replace_oauth_state(1, skipped_exchange).await,
            Err(AuthorizationStateError::StorageConflict)
        );

        let first = repository.clone();
        let second = repository.clone();
        let (left, right) = tokio::join!(
            claim_oauth_state(&first, "state-1", "owner-1"),
            claim_oauth_state(&second, "state-1", "owner-2")
        );
        assert_ne!(left.is_ok(), right.is_ok());
        assert!(matches!(
            left.as_ref().err().or(right.as_ref().err()),
            Some(AuthorizationStateError::StorageConflict)
        ));

        let mut current = repository
            .get_oauth_state("state-1")
            .await
            .unwrap()
            .unwrap();
        current.status = AuthOAuthStatus::ExchangeStarted;
        current.version += 1;
        repository
            .replace_oauth_state(current.version - 1, current.clone())
            .await
            .unwrap();

        let stale = current.clone();
        current.status = AuthOAuthStatus::RestartRequired;
        current.version += 1;
        repository
            .replace_oauth_state(current.version - 1, current.clone())
            .await
            .unwrap();
        assert_eq!(
            repository.replace_oauth_state(stale.version, stale).await,
            Err(AuthorizationStateError::StorageConflict)
        );

        let mut completed = oauth_state();
        completed.state_id = "state-2".to_owned();
        repository
            .create_oauth_state(completed.clone())
            .await
            .unwrap();
        completed = claim_oauth_state(&repository, "state-2", "owner-1")
            .await
            .unwrap();
        completed.status = AuthOAuthStatus::ExchangeStarted;
        completed.version += 1;
        repository
            .replace_oauth_state(completed.version - 1, completed.clone())
            .await
            .unwrap();
        completed.status = AuthOAuthStatus::Completed;
        completed.result_digest = Some(DIGEST.to_owned());
        completed.version += 1;
        repository
            .replace_oauth_state(completed.version - 1, completed.clone())
            .await
            .unwrap();
        assert_eq!(
            repository.get_oauth_state("state-2").await.unwrap(),
            Some(completed)
        );

        let replay = ConnectReplayRecord {
            format: "trellis.session-proof-replay.v1".to_owned(),
            purpose: "natsConnectContext".to_owned(),
            signer_key_id: DIGEST.to_owned(),
            request_id: "01J00000000000000000000000".to_owned(),
            transcript_digest: DIGEST.to_owned(),
            admitted_at: 1_000,
            expires_at: 36_000,
            version: 1,
        };
        assert!(repository
            .admit_connect_replay(replay.clone())
            .await
            .unwrap());
        assert!(!repository.admit_connect_replay(replay).await.unwrap());

        repository
            .put_connection_presence(AuthConnectionPresence {
                format: "trellis.auth-connection-presence.v1".to_owned(),
                connection_id: DIGEST.to_owned(),
                session_id: "ses_01".to_owned(),
                context_id: "ctx_01".to_owned(),
                context_digest: DIGEST.to_owned(),
                server_id: "server-1".to_owned(),
                client_id: "42".to_owned(),
                user_nkey: "user-nkey".to_owned(),
                remote_address: Some("127.0.0.1".to_owned()),
                connected_at: 1_000,
                last_seen_at: 1_000,
                version: 1,
            })
            .await
            .unwrap();
        assert_eq!(
            repository
                .list_connection_presence(Some("ses_01"))
                .await
                .unwrap()
                .len(),
            1
        );
        repository
            .delete_connection_presence("user-nkey")
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn in_memory_repository_conforms() {
        repository_conformance(InMemoryAuthEphemeralRepository::default()).await;
    }

    #[tokio::test]
    #[ignore = "requires Podman for a live NATS JetStream container"]
    async fn nats_kv_repository_conforms() {
        struct Server {
            child: Child,
            name: String,
        }

        impl Drop for Server {
            fn drop(&mut self) {
                let _ = Command::new("podman")
                    .args(["rm", "-f", &self.name])
                    .stdout(Stdio::null())
                    .stderr(Stdio::null())
                    .status();
                let _ = self.child.wait();
            }
        }

        let listener = std::net::TcpListener::bind(("127.0.0.1", 0)).unwrap();
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let name = format!("trellis-auth-ephemeral-test-{}-{port}", std::process::id());
        let server = Server {
            child: Command::new("podman")
                .args([
                    "run",
                    "--rm",
                    "--name",
                    &name,
                    "-p",
                    &format!("127.0.0.1:{port}:4222"),
                    "docker.io/library/nats:2-alpine",
                    "-js",
                ])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn()
                .unwrap(),
            name,
        };
        let url = format!("nats://127.0.0.1:{port}");
        let mut client = None;
        for _ in 0..100 {
            match async_nats::connect(&url).await {
                Ok(connected) => {
                    client = Some(connected);
                    break;
                }
                Err(_) => tokio::time::sleep(std::time::Duration::from_millis(25)).await,
            }
        }
        let repository = NatsAuthEphemeralRepository::ensure(client.expect("NATS did not start"))
            .await
            .unwrap();
        repository_conformance(repository).await;
        drop(server);
    }

    #[test]
    fn strict_json_keeps_required_nullable_fields() {
        let value = serde_json::to_value(browser_flow()).unwrap();
        assert_eq!(value["principalId"], serde_json::Value::Null);
        assert_eq!(value["claimOwner"], serde_json::Value::Null);
        assert_eq!(value["claimedAt"], serde_json::Value::Null);
        assert_eq!(value["durableResultDigest"], serde_json::Value::Null);
        assert_eq!(value["completedAt"], serde_json::Value::Null);

        let mut value = serde_json::to_value(oauth_state()).unwrap();
        assert_eq!(value["claimOwner"], serde_json::Value::Null);
        assert_eq!(value["resultDigest"], serde_json::Value::Null);
        value["unknown"] = serde_json::json!(true);
        assert!(serde_json::from_value::<AuthOAuthState>(value).is_err());
    }
}
