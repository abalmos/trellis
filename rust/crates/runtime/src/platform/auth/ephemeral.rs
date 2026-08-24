use std::collections::BTreeMap;
#[cfg(test)]
use std::sync::{Arc, Mutex};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use trellis_protocol::{digest_json, GrantSet};

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
    pub required_grant_set: GrantSet,
    pub optional_grant_bundles: BTreeMap<String, GrantSet>,
    pub required_capabilities: Vec<String>,
    pub optional_capability_definitions: BTreeMap<String, GrantSet>,
}

impl BrowserConsentProposal {
    pub(crate) fn validate(&self) -> Result<(), AuthorizationStateError> {
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
pub(crate) struct AuthConnectionPresence {
    pub format: String,
    pub connection_id: String,
    pub session_id: String,
    pub context_digest: String,
    pub server_id: String,
    pub client_id: String,
    pub user_nkey: String,
    pub remote_address: Option<String>,
    pub connected_at: i64,
    pub last_seen_at: i64,
    pub version: u64,
}

pub(crate) fn validate_connection_kick_response(
    payload: &[u8],
) -> Result<(), AuthorizationStateError> {
    let response: Value = serde_json::from_slice(payload)
        .map_err(|error| AuthorizationStateError::Storage(error.to_string()))?;
    let Some(error) = response.get("error") else {
        return Ok(());
    };
    let description = error
        .get("description")
        .and_then(Value::as_str)
        .unwrap_or("unknown NATS system API error");
    if description == "no such client or leafnode id" {
        return Ok(());
    }
    Err(AuthorizationStateError::Storage(format!(
        "NATS connection kick failed: {description}"
    )))
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
    async fn put_connection_presence(
        &self,
        record: AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError>;
    async fn delete_connection_presence(
        &self,
        connection_id: &str,
    ) -> Result<(), AuthorizationStateError>;
    async fn list_connection_presence(
        &self,
        session_id: Option<&str>,
    ) -> Result<Vec<AuthConnectionPresence>, AuthorizationStateError>;
    async fn list_connection_presence_by_context(
        &self,
        context_digest: &str,
    ) -> Result<Vec<AuthConnectionPresence>, AuthorizationStateError> {
        Ok(self
            .list_connection_presence(None)
            .await?
            .into_iter()
            .filter(|connection| connection.context_digest == context_digest)
            .collect())
    }
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

    async fn put_connection_presence(
        &self,
        record: AuthConnectionPresence,
    ) -> Result<(), AuthorizationStateError> {
        record.validate()?;
        lock(&self.connections)?.insert(record.connection_id.clone(), record);
        Ok(())
    }

    async fn delete_connection_presence(
        &self,
        connection_id: &str,
    ) -> Result<(), AuthorizationStateError> {
        lock(&self.connections)?.remove(connection_id);
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
        connections: kv::Store,
    }

    impl NatsAuthEphemeralRepository {
        /// Opens or creates and validates both auth-owned KV buckets.
        pub(crate) async fn ensure(
            client: async_nats::Client,
            connection_max_age: Duration,
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
            let connections =
                open_or_create(&jetstream, CONNECTIONS_BUCKET, connection_max_age, 16_384).await?;
            Ok(Self {
                browser_flows,
                oauth_states,
                connections,
            })
        }

        /// Validates all required auth-owned KV buckets without creating or updating them.
        pub(crate) async fn check(
            client: async_nats::Client,
            connection_max_age: Duration,
        ) -> Result<(), AuthorizationStateError> {
            let jetstream = jetstream::new(client);
            for (bucket, max_age, max_value_size) in [
                (
                    BROWSER_FLOW_BUCKET,
                    Duration::from_millis(86_400_000),
                    65_536,
                ),
                (OAUTH_STATE_BUCKET, Duration::from_millis(900_000), 16_384),
                (CONNECTIONS_BUCKET, connection_max_age, 16_384),
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

        async fn put_connection_presence(
            &self,
            record: AuthConnectionPresence,
        ) -> Result<(), AuthorizationStateError> {
            record.validate()?;
            self.connections
                .put(record.connection_id.clone(), encode(&record)?)
                .await
                .map(|_| ())
                .map_err(|error| storage(format!("failed to write connection presence: {error}")))
        }

        async fn delete_connection_presence(
            &self,
            connection_id: &str,
        ) -> Result<(), AuthorizationStateError> {
            self.connections
                .delete(connection_id)
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
                let entry = self.connections.entry(&key).await.map_err(|error| {
                    storage(format!(
                        "failed to read connection presence key {key}: {error}"
                    ))
                })?;
                let Some(entry) = entry.filter(|entry| entry.operation == kv::Operation::Put)
                else {
                    continue;
                };
                let record = match decode::<AuthConnectionPresence>(&entry.value) {
                    Ok(record) => record,
                    Err(error) => {
                        tracing::warn!(key, error = %error, "skipping malformed connection presence");
                        continue;
                    }
                };
                if session_id.is_none_or(|id| record.session_id == id) {
                    records.push(record);
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
// Keep the test module beside this flat root module; moving the production
// module into a directory would churn its intentionally stable private paths.
#[path = "ephemeral_tests.rs"]
mod tests;
