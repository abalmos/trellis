//! Shared request and response types for `trellis.auth@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
/// Generated schema type `AuthCapabilitiesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilitiesListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `sourceApi` wire field.
    #[serde(rename = "sourceApi")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_api: Option<String>,
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItemAllowsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthCapabilitiesListResponseEntriesItemAllowsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthCapabilitiesListResponseEntriesItemAllowsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str> for AuthCapabilitiesListResponseEntriesItemAllowsItemAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthCapabilitiesListResponseEntriesItemAllowsItemAction {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthCapabilitiesListResponseEntriesItemAllowsItemAction {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthCapabilitiesListResponseEntriesItemAllowsItemAction> for &str {
    fn eq(&self, other: &AuthCapabilitiesListResponseEntriesItemAllowsItemAction) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface {
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str> for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface> for &str {
    fn eq(
        &self,
        other: &AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource {
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
    for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource>
    for &str
{
    fn eq(
        &self,
        other: &AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItemAllowsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthCapabilitiesListResponseEntriesItemAllowsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthCapabilitiesListResponseEntriesItemAllowsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource:
            AuthCapabilitiesListResponseEntriesItemAllowsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItemAllowsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilitiesListResponseEntriesItemAllowsItem {
    /// The `action` wire field.
    pub action: AuthCapabilitiesListResponseEntriesItemAllowsItemAction,
    /// The `target` wire field.
    pub target: AuthCapabilitiesListResponseEntriesItemAllowsItemTarget,
}
/// Generated schema type `AuthCapabilitiesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilitiesListResponseEntriesItem {
    /// The `allows` wire field.
    pub allows: Vec<AuthCapabilitiesListResponseEntriesItemAllowsItem>,
    /// The `capability` wire field.
    pub capability: String,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `sourceApi` wire field.
    #[serde(rename = "sourceApi")]
    pub source_api: Option<String>,
}
/// Generated schema type `AuthCapabilitiesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilitiesListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthCapabilitiesListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthCapabilityGroupsDeleteRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsDeleteRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}
/// Generated schema type `AuthCapabilityGroupsDeleteResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsDeleteResponse {
    /// The `success` wire field.
    pub success: bool,
}
/// Generated schema type `AuthCapabilityGroupsGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsGetRequest {
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
}
/// Generated schema type `AuthCapabilityGroupsGetResponseGroup`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsGetResponseGroup {
    /// The `capabilities` wire field.
    pub capabilities: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
    /// The `includedGroups` wire field.
    #[serde(rename = "includedGroups")]
    pub included_groups: Vec<String>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthCapabilityGroupsGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsGetResponse {
    /// The `group` wire field.
    pub group: AuthCapabilityGroupsGetResponseGroup,
}
/// Generated schema type `AuthCapabilityGroupsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsListRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
}
/// Generated schema type `AuthCapabilityGroupsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsListResponseEntriesItem {
    /// The `capabilities` wire field.
    pub capabilities: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
    /// The `includedGroups` wire field.
    #[serde(rename = "includedGroups")]
    pub included_groups: Vec<String>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthCapabilityGroupsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsListResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<AuthCapabilityGroupsListResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `AuthCapabilityGroupsPutRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsPutRequest {
    /// The `capabilities` wire field.
    pub capabilities: Vec<String>,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `includedGroups` wire field.
    #[serde(rename = "includedGroups")]
    pub included_groups: Vec<String>,
}
/// Generated schema type `AuthCapabilityGroupsPutResponseGroup`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsPutResponseGroup {
    /// The `capabilities` wire field.
    pub capabilities: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `groupKey` wire field.
    #[serde(rename = "groupKey")]
    pub group_key: String,
    /// The `includedGroups` wire field.
    #[serde(rename = "includedGroups")]
    pub included_groups: Vec<String>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthCapabilityGroupsPutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthCapabilityGroupsPutResponse {
    /// The `group` wire field.
    pub group: AuthCapabilityGroupsPutResponseGroup,
}
/// Generated schema type `AuthConnectionsKickRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsKickRequest {
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthConnectionsKickResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsKickResponse {
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `kicked` wire field.
    pub kicked: bool,
}
/// Generated schema type `AuthConnectionsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub session_id: Option<String>,
}
/// Generated schema type `AuthConnectionsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsListResponseEntriesItem {
    /// The `clientId` wire field.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// The `connectedAt` wire field.
    #[serde(rename = "connectedAt")]
    pub connected_at: i64,
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `remoteAddress` wire field.
    #[serde(rename = "remoteAddress")]
    pub remote_address: Option<String>,
    /// The `serverId` wire field.
    #[serde(rename = "serverId")]
    pub server_id: String,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The `userNkey` wire field.
    #[serde(rename = "userNkey")]
    pub user_nkey: String,
}
/// Generated schema type `AuthConnectionsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthConnectionsListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationRequest {
    /// The `expectedBaseAuthorityVersion` wire field.
    #[serde(rename = "expectedBaseAuthorityVersion")]
    pub expected_base_authority_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<
        AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction,
    > for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTarget
{
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind {
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityParticipantKind,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityAcceptMigrationResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction
{
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<
        AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction,
    > for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTarget
{
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposalState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptMigrationResponseProposalState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityAcceptMigrationResponseProposalState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptMigrationResponseProposalState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptMigrationResponseProposalState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptMigrationResponseProposalState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptMigrationResponseProposalState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponseProposal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponseProposal {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityAcceptMigrationResponseProposalAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityAcceptMigrationResponseProposalClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityAcceptMigrationResponseProposalProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityAcceptMigrationResponseProposalState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptMigrationResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptMigrationResponse {
    /// The `authority` wire field.
    pub authority: AuthDeploymentAuthorityAcceptMigrationResponseAuthority,
    /// The `proposal` wire field.
    pub proposal: AuthDeploymentAuthorityAcceptMigrationResponseProposal,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateRequest {
    /// The `expectedBaseAuthorityVersion` wire field.
    #[serde(rename = "expectedBaseAuthorityVersion")]
    pub expected_base_authority_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<
        AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction,
    > for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind {
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityParticipantKind,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityAcceptUpdateResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<
        AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction,
    > for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposalState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityAcceptUpdateResponseProposalState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityAcceptUpdateResponseProposalState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityAcceptUpdateResponseProposalState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityAcceptUpdateResponseProposalState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityAcceptUpdateResponseProposalState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityAcceptUpdateResponseProposalState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponseProposal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponseProposal {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityAcceptUpdateResponseProposalAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityAcceptUpdateResponseProposalClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityAcceptUpdateResponseProposalProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityAcceptUpdateResponseProposalState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityAcceptUpdateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityAcceptUpdateResponse {
    /// The `authority` wire field.
    pub authority: AuthDeploymentAuthorityAcceptUpdateResponseAuthority,
    /// The `proposal` wire field.
    pub proposal: AuthDeploymentAuthorityAcceptUpdateResponseProposal,
}
/// Generated schema type `AuthDeploymentAuthorityGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityGetRequest {
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl
    AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions: Vec<AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityKind {
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityGetResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityGetResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityGetResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityGetResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityGetResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityGetResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentAuthorityGetResponseAuthorityParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityGetResponseAuthorityParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityGetResponseAuthorityParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityGetResponseAuthorityParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityGetResponseAuthorityParticipantKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityGetResponseAuthorityParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityGetResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityGetResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityGetResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityGetResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityGetResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityGetResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityGetResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityGetResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityGetResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthDeploymentAuthorityGetResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentAuthorityGetResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthDeploymentAuthorityGetResponseAuthorityParticipantKind,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityGetResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentAuthorityGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityGetResponse {
    /// The `authority` wire field.
    pub authority: AuthDeploymentAuthorityGetResponseAuthority,
}
/// Generated schema type `AuthDeploymentAuthorityListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListRequestState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListRequestState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthDeploymentAuthorityListRequestState>,
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemKind {
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityListResponseEntriesItemKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityListResponseEntriesItemKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityListResponseEntriesItemKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityListResponseEntriesItemKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListResponseEntriesItemKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityListResponseEntriesItemKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentAuthorityListResponseEntriesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityListResponseEntriesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityListResponseEntriesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityListResponseEntriesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListResponseEntriesItemParticipantKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityListResponseEntriesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityListResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityListResponseEntriesItem {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthDeploymentAuthorityListResponseEntriesItemDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentAuthorityListResponseEntriesItemKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthDeploymentAuthorityListResponseEntriesItemParticipantKind,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentAuthorityListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDeploymentAuthorityListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityPlanRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlanRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `participantArtifact` wire field.
    #[serde(rename = "participantArtifact")]
    pub participant_artifact: BTreeMap<String, Value>,
    /// The `referencedApiArtifacts` wire field.
    #[serde(rename = "referencedApiArtifacts")]
    pub referenced_api_artifacts: Vec<BTreeMap<String, Value>>,
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityPlanResponseProposalAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlanResponseProposalAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlanResponseProposalAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlanResponseProposalAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlanResponseProposalAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlanResponseProposalAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityPlanResponseProposalClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlanResponseProposalClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlanResponseProposalClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlanResponseProposalClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlanResponseProposalClassification> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlanResponseProposalClassification) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlanResponseProposalProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityPlanResponseProposalProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposalState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlanResponseProposalState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityPlanResponseProposalState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlanResponseProposalState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlanResponseProposalState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlanResponseProposalState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlanResponseProposalState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlanResponseProposalState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponseProposal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlanResponseProposal {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityPlanResponseProposalAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityPlanResponseProposalClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityPlanResponseProposalProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityPlanResponseProposalState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityPlanResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlanResponse {
    /// The `proposal` wire field.
    pub proposal: AuthDeploymentAuthorityPlanResponseProposal,
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansGetRequest {
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansGetResponseProposalClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansGetResponseProposalClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansGetResponseProposalClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansGetResponseProposalClassification> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansGetResponseProposalClassification) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposalState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansGetResponseProposalState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityPlansGetResponseProposalState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansGetResponseProposalState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansGetResponseProposalState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansGetResponseProposalState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansGetResponseProposalState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansGetResponseProposalState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponseProposal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansGetResponseProposal {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityPlansGetResponseProposalAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityPlansGetResponseProposalClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityPlansGetResponseProposalProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityPlansGetResponseProposalState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityPlansGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansGetResponse {
    /// The `proposal` wire field.
    pub proposal: AuthDeploymentAuthorityPlansGetResponseProposal,
}
/// Generated schema type `AuthDeploymentAuthorityPlansListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListRequestState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityPlansListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansListRequestState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthDeploymentAuthorityPlansListRequestState>,
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansListResponseEntriesItemClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansListResponseEntriesItemClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansListResponseEntriesItemClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansListResponseEntriesItemClassification> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansListResponseEntriesItemClassification,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<
        AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction,
    > for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityPlansListResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityPlansListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityPlansListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityPlansListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityPlansListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityPlansListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityPlansListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansListResponseEntriesItem {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityPlansListResponseEntriesItemAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityPlansListResponseEntriesItemClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityPlansListResponseEntriesItemProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityPlansListResponseEntriesItemState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityPlansListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityPlansListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDeploymentAuthorityPlansListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityReconcileRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityReconcileRequest {
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl
    PartialEq<AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action:
        AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target:
        AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityKind {
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityReconcileResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityReconcileResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityReconcileResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityReconcileResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityReconcileResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityReconcileResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthDeploymentAuthorityReconcileResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityReconcileResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityReconcileResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityReconcileResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityReconcileResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityReconcileResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityReconcileResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthDeploymentAuthorityReconcileResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentAuthorityReconcileResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthDeploymentAuthorityReconcileResponseAuthorityParticipantKind,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityReconcileResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentAuthorityReconcileResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityReconcileResponse {
    /// The `authority` wire field.
    pub authority: AuthDeploymentAuthorityReconcileResponseAuthority,
}
/// Generated schema type `AuthDeploymentAuthorityRejectRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityRejectRequest {
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
    /// The `deployment` wire value.
    #[serde(rename = "deployment")]
    Deployment,
}
impl AuthDeploymentAuthorityRejectResponseProposalAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
            Self::Deployment => "deployment",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityRejectResponseProposalAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityRejectResponseProposalAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityRejectResponseProposalAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityRejectResponseProposalAuthorityKind> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityRejectResponseProposalAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalClassification`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalClassification {
    /// The `initial` wire value.
    #[serde(rename = "initial")]
    Initial,
    /// The `update` wire value.
    #[serde(rename = "update")]
    Update,
    /// The `migration` wire value.
    #[serde(rename = "migration")]
    Migration,
}
impl AuthDeploymentAuthorityRejectResponseProposalClassification {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Initial => "initial",
            Self::Update => "update",
            Self::Migration => "migration",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityRejectResponseProposalClassification {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityRejectResponseProposalClassification {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityRejectResponseProposalClassification {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityRejectResponseProposalClassification> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityRejectResponseProposalClassification) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalProposedGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityRejectResponseProposalProposedGrantSet {
    /// The `format` wire field.
    pub format: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthDeploymentAuthorityRejectResponseProposalProposedGrantSetPermissionsItem>,
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposalState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentAuthorityRejectResponseProposalState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `superseded` wire value.
    #[serde(rename = "superseded")]
    Superseded,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
}
impl AuthDeploymentAuthorityRejectResponseProposalState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Superseded => "superseded",
            Self::Expired => "expired",
        }
    }
}
impl AsRef<str> for AuthDeploymentAuthorityRejectResponseProposalState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentAuthorityRejectResponseProposalState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentAuthorityRejectResponseProposalState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentAuthorityRejectResponseProposalState> for &str {
    fn eq(&self, other: &AuthDeploymentAuthorityRejectResponseProposalState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponseProposal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityRejectResponseProposal {
    /// The `authorityKind` wire field.
    #[serde(rename = "authorityKind")]
    pub authority_kind: AuthDeploymentAuthorityRejectResponseProposalAuthorityKind,
    /// The `baseAuthorityVersion` wire field.
    #[serde(rename = "baseAuthorityVersion")]
    pub base_authority_version: Option<i64>,
    /// The `classification` wire field.
    pub classification: AuthDeploymentAuthorityRejectResponseProposalClassification,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decisionAt` wire field.
    #[serde(rename = "decisionAt")]
    pub decision_at: Option<i64>,
    /// The `decisionBy` wire field.
    #[serde(rename = "decisionBy")]
    pub decision_by: Option<String>,
    /// The `decisionReason` wire field.
    #[serde(rename = "decisionReason")]
    pub decision_reason: Option<String>,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `proposalId` wire field.
    #[serde(rename = "proposalId")]
    pub proposal_id: String,
    /// The `proposedCapabilities` wire field.
    #[serde(rename = "proposedCapabilities")]
    pub proposed_capabilities: Vec<String>,
    /// The `proposedGrantSet` wire field.
    #[serde(rename = "proposedGrantSet")]
    pub proposed_grant_set: AuthDeploymentAuthorityRejectResponseProposalProposedGrantSet,
    /// The `reasons` wire field.
    pub reasons: Vec<String>,
    /// The `state` wire field.
    pub state: AuthDeploymentAuthorityRejectResponseProposalState,
    /// The `subjectId` wire field.
    #[serde(rename = "subjectId")]
    pub subject_id: String,
}
/// Generated schema type `AuthDeploymentAuthorityRejectResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentAuthorityRejectResponse {
    /// The `proposal` wire field.
    pub proposal: AuthDeploymentAuthorityRejectResponseProposal,
}
/// Generated schema type `AuthDeploymentsCreateRequestKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsCreateRequestKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsCreateRequestKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsCreateRequestKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsCreateRequestKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsCreateRequestKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsCreateRequestKind> for &str {
    fn eq(&self, other: &AuthDeploymentsCreateRequestKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsCreateRequestReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsCreateRequestReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsCreateRequestReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsCreateRequestReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsCreateRequestReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsCreateRequestReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsCreateRequestReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsCreateRequestReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsCreateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsCreateRequest {
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsCreateRequestKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsCreateRequestReviewMode>,
}
/// Generated schema type `AuthDeploymentsCreateResponseDeploymentKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsCreateResponseDeploymentKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsCreateResponseDeploymentKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsCreateResponseDeploymentKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsCreateResponseDeploymentKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsCreateResponseDeploymentKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsCreateResponseDeploymentKind> for &str {
    fn eq(&self, other: &AuthDeploymentsCreateResponseDeploymentKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsCreateResponseDeploymentReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsCreateResponseDeploymentReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsCreateResponseDeploymentReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsCreateResponseDeploymentReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsCreateResponseDeploymentReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsCreateResponseDeploymentReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsCreateResponseDeploymentReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsCreateResponseDeploymentReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsCreateResponseDeploymentState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsCreateResponseDeploymentState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsCreateResponseDeploymentState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsCreateResponseDeploymentState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsCreateResponseDeploymentState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsCreateResponseDeploymentState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsCreateResponseDeploymentState> for &str {
    fn eq(&self, other: &AuthDeploymentsCreateResponseDeploymentState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsCreateResponseDeployment`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsCreateResponseDeployment {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsCreateResponseDeploymentKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsCreateResponseDeploymentReviewMode>,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthDeploymentsCreateResponseDeploymentState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsCreateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsCreateResponse {
    /// The `deployment` wire field.
    pub deployment: AuthDeploymentsCreateResponseDeployment,
}
/// Generated schema type `AuthDeploymentsDisableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsDisableRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentsDisableResponseDeploymentKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsDisableResponseDeploymentKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsDisableResponseDeploymentKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsDisableResponseDeploymentKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsDisableResponseDeploymentKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsDisableResponseDeploymentKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsDisableResponseDeploymentKind> for &str {
    fn eq(&self, other: &AuthDeploymentsDisableResponseDeploymentKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsDisableResponseDeploymentReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsDisableResponseDeploymentReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsDisableResponseDeploymentReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsDisableResponseDeploymentReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsDisableResponseDeploymentReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsDisableResponseDeploymentReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsDisableResponseDeploymentReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsDisableResponseDeploymentReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsDisableResponseDeploymentState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsDisableResponseDeploymentState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsDisableResponseDeploymentState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsDisableResponseDeploymentState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsDisableResponseDeploymentState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsDisableResponseDeploymentState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsDisableResponseDeploymentState> for &str {
    fn eq(&self, other: &AuthDeploymentsDisableResponseDeploymentState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsDisableResponseDeployment`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsDisableResponseDeployment {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsDisableResponseDeploymentKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsDisableResponseDeploymentReviewMode>,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthDeploymentsDisableResponseDeploymentState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsDisableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsDisableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsDisableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsDisableResponse {
    /// The `deployment` wire field.
    pub deployment: AuthDeploymentsDisableResponseDeployment,
    /// The `mutation` wire field.
    pub mutation: AuthDeploymentsDisableResponseMutation,
}
/// Generated schema type `AuthDeploymentsEnableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsEnableRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentsEnableResponseDeploymentKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsEnableResponseDeploymentKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsEnableResponseDeploymentKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsEnableResponseDeploymentKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsEnableResponseDeploymentKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsEnableResponseDeploymentKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsEnableResponseDeploymentKind> for &str {
    fn eq(&self, other: &AuthDeploymentsEnableResponseDeploymentKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsEnableResponseDeploymentReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsEnableResponseDeploymentReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsEnableResponseDeploymentReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsEnableResponseDeploymentReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsEnableResponseDeploymentReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsEnableResponseDeploymentReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsEnableResponseDeploymentReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsEnableResponseDeploymentReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsEnableResponseDeploymentState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsEnableResponseDeploymentState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsEnableResponseDeploymentState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsEnableResponseDeploymentState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsEnableResponseDeploymentState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsEnableResponseDeploymentState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsEnableResponseDeploymentState> for &str {
    fn eq(&self, other: &AuthDeploymentsEnableResponseDeploymentState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsEnableResponseDeployment`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsEnableResponseDeployment {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsEnableResponseDeploymentKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsEnableResponseDeploymentReviewMode>,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthDeploymentsEnableResponseDeploymentState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsEnableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsEnableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsEnableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsEnableResponse {
    /// The `deployment` wire field.
    pub deployment: AuthDeploymentsEnableResponseDeployment,
    /// The `mutation` wire field.
    pub mutation: AuthDeploymentsEnableResponseMutation,
}
/// Generated schema type `AuthDeploymentsListRequestKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsListRequestKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsListRequestKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsListRequestKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsListRequestKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsListRequestKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsListRequestKind> for &str {
    fn eq(&self, other: &AuthDeploymentsListRequestKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsListRequestState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsListRequestState> for &str {
    fn eq(&self, other: &AuthDeploymentsListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `kind` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<AuthDeploymentsListRequestKind>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthDeploymentsListRequestState>,
}
/// Generated schema type `AuthDeploymentsListResponseEntriesItemKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsListResponseEntriesItemKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsListResponseEntriesItemKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsListResponseEntriesItemKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsListResponseEntriesItemKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsListResponseEntriesItemKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsListResponseEntriesItemKind> for &str {
    fn eq(&self, other: &AuthDeploymentsListResponseEntriesItemKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsListResponseEntriesItemReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsListResponseEntriesItemReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsListResponseEntriesItemReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsListResponseEntriesItemReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsListResponseEntriesItemReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsListResponseEntriesItemReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsListResponseEntriesItemReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsListResponseEntriesItemReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsListResponseEntriesItemState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthDeploymentsListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsListResponseEntriesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsListResponseEntriesItemKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsListResponseEntriesItemReviewMode>,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthDeploymentsListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDeploymentsListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeploymentsRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsRemoveRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeploymentsRemoveResponseDeploymentKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsRemoveResponseDeploymentKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthDeploymentsRemoveResponseDeploymentKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthDeploymentsRemoveResponseDeploymentKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsRemoveResponseDeploymentKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsRemoveResponseDeploymentKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsRemoveResponseDeploymentKind> for &str {
    fn eq(&self, other: &AuthDeploymentsRemoveResponseDeploymentKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsRemoveResponseDeploymentReviewMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsRemoveResponseDeploymentReviewMode {
    /// The `none` wire value.
    #[serde(rename = "none")]
    None,
    /// The `required` wire value.
    #[serde(rename = "required")]
    Required,
}
impl AuthDeploymentsRemoveResponseDeploymentReviewMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Required => "required",
        }
    }
}
impl AsRef<str> for AuthDeploymentsRemoveResponseDeploymentReviewMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsRemoveResponseDeploymentReviewMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsRemoveResponseDeploymentReviewMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsRemoveResponseDeploymentReviewMode> for &str {
    fn eq(&self, other: &AuthDeploymentsRemoveResponseDeploymentReviewMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsRemoveResponseDeploymentState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeploymentsRemoveResponseDeploymentState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeploymentsRemoveResponseDeploymentState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeploymentsRemoveResponseDeploymentState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeploymentsRemoveResponseDeploymentState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeploymentsRemoveResponseDeploymentState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeploymentsRemoveResponseDeploymentState> for &str {
    fn eq(&self, other: &AuthDeploymentsRemoveResponseDeploymentState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeploymentsRemoveResponseDeployment`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsRemoveResponseDeployment {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthDeploymentsRemoveResponseDeploymentKind,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: Option<String>,
    /// The `requiresDeviceDelegation` wire field.
    #[serde(rename = "requiresDeviceDelegation")]
    pub requires_device_delegation: bool,
    /// The `reviewMode` wire field.
    #[serde(rename = "reviewMode")]
    pub review_mode: Option<AuthDeploymentsRemoveResponseDeploymentReviewMode>,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthDeploymentsRemoveResponseDeploymentState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsRemoveResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsRemoveResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeploymentsRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeploymentsRemoveResponse {
    /// The `deployment` wire field.
    pub deployment: AuthDeploymentsRemoveResponseDeployment,
    /// The `mutation` wire field.
    pub mutation: AuthDeploymentsRemoveResponseMutation,
}
/// Generated schema type `AuthDeviceUserAuthoritiesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval>
    for &str
{
    fn eq(
        &self,
        other: &AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState> for &str {
    fn eq(
        &self,
        other: &AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponseEntriesItemDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesListResponseEntriesItemDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval:
        AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesListResponseEntriesItemDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesListResponseEntriesItem {
    /// The `authority` wire field.
    pub authority: Option<BTreeMap<String, Value>>,
    /// The `device` wire field.
    pub device: AuthDeviceUserAuthoritiesListResponseEntriesItemDevice,
}
/// Generated schema type `AuthDeviceUserAuthoritiesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDeviceUserAuthoritiesListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsDecideRequestDecision`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesReviewsDecideRequestDecision {
    /// The `approve` wire value.
    #[serde(rename = "approve")]
    Approve,
    /// The `reject` wire value.
    #[serde(rename = "reject")]
    Reject,
}
impl AuthDeviceUserAuthoritiesReviewsDecideRequestDecision {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Reject => "reject",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesReviewsDecideRequestDecision {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesReviewsDecideRequestDecision {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesReviewsDecideRequestDecision {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesReviewsDecideRequestDecision> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesReviewsDecideRequestDecision) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsDecideRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsDecideRequest {
    /// The `decision` wire field.
    pub decision: AuthDeviceUserAuthoritiesReviewsDecideRequestDecision,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `reviewId` wire field.
    #[serde(rename = "reviewId")]
    pub review_id: String,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsDecideResponseReview`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsDecideResponseReview {
    /// The `activatedByUserPrincipalId` wire field.
    #[serde(rename = "activatedByUserPrincipalId")]
    pub activated_by_user_principal_id: Option<String>,
    /// The `confirmationCode` wire field.
    #[serde(rename = "confirmationCode")]
    pub confirmation_code: String,
    /// The `decidedAt` wire field.
    #[serde(rename = "decidedAt")]
    pub decided_at: Option<i64>,
    /// The `decidedBy` wire field.
    #[serde(rename = "decidedBy")]
    pub decided_by: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `devicePrincipalId` wire field.
    #[serde(rename = "devicePrincipalId")]
    pub device_principal_id: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `requestedAt` wire field.
    #[serde(rename = "requestedAt")]
    pub requested_at: i64,
    /// The `reviewId` wire field.
    #[serde(rename = "reviewId")]
    pub review_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesReviewsDecideResponseReviewState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsDecideResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsDecideResponse {
    /// The `review` wire field.
    pub review: AuthDeviceUserAuthoritiesReviewsDecideResponseReview,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesReviewsListRequestState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesReviewsListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesReviewsListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesReviewsListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesReviewsListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesReviewsListRequestState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesReviewsListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthDeviceUserAuthoritiesReviewsListRequestState>,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsListResponseEntriesItem {
    /// The `activatedByUserPrincipalId` wire field.
    #[serde(rename = "activatedByUserPrincipalId")]
    pub activated_by_user_principal_id: Option<String>,
    /// The `confirmationCode` wire field.
    #[serde(rename = "confirmationCode")]
    pub confirmation_code: String,
    /// The `decidedAt` wire field.
    #[serde(rename = "decidedAt")]
    pub decided_at: Option<i64>,
    /// The `decidedBy` wire field.
    #[serde(rename = "decidedBy")]
    pub decided_by: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `devicePrincipalId` wire field.
    #[serde(rename = "devicePrincipalId")]
    pub device_principal_id: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `requestedAt` wire field.
    #[serde(rename = "requestedAt")]
    pub requested_at: i64,
    /// The `reviewId` wire field.
    #[serde(rename = "reviewId")]
    pub review_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesReviewsListResponseEntriesItemState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewsListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDeviceUserAuthoritiesReviewsListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesRevokeRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `devicePrincipalId` wire field.
    #[serde(rename = "devicePrincipalId")]
    pub device_principal_id: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval> for &str {
    fn eq(
        &self,
        other: &AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeResponseDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesRevokeResponseDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesRevokeResponseDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesRevokeResponseDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesRevokeResponseDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesRevokeResponseDeviceState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesRevokeResponseDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeResponseDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesRevokeResponseDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval:
        AuthDeviceUserAuthoritiesRevokeResponseDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDeviceUserAuthoritiesRevokeResponseDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesRevokeResponseDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesRevokeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesRevokeResponse {
    /// The `device` wire field.
    pub device: AuthDeviceUserAuthoritiesRevokeResponseDevice,
    /// The `kickedSessionCount` wire field.
    #[serde(rename = "kickedSessionCount")]
    pub kicked_session_count: i64,
}
/// Generated schema type `AuthDevicesConnectInfoGetRequestProofFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesConnectInfoGetRequestProofFormat {
    /// The `trellis.session-proof.v1` wire value.
    #[serde(rename = "trellis.session-proof.v1")]
    TrellisSessionProofV1,
}
impl AuthDevicesConnectInfoGetRequestProofFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisSessionProofV1 => "trellis.session-proof.v1",
        }
    }
}
impl AsRef<str> for AuthDevicesConnectInfoGetRequestProofFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesConnectInfoGetRequestProofFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesConnectInfoGetRequestProofFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesConnectInfoGetRequestProofFormat> for &str {
    fn eq(&self, other: &AuthDevicesConnectInfoGetRequestProofFormat) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesConnectInfoGetRequestProof`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesConnectInfoGetRequestProof {
    /// The `format` wire field.
    pub format: AuthDevicesConnectInfoGetRequestProofFormat,
    /// The `signature` wire field.
    pub signature: String,
}
/// Generated schema type `AuthDevicesConnectInfoGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesConnectInfoGetRequest {
    /// The `challengeDigest` wire field.
    #[serde(rename = "challengeDigest")]
    pub challenge_digest: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `deviceIdentityKeyId` wire field.
    #[serde(rename = "deviceIdentityKeyId")]
    pub device_identity_key_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `issuedAt` wire field.
    #[serde(rename = "issuedAt")]
    pub issued_at: i64,
    /// The `newSessionNkey` wire field.
    #[serde(rename = "newSessionNkey")]
    pub new_session_nkey: String,
    /// The `newSessionPublicKey` wire field.
    #[serde(rename = "newSessionPublicKey")]
    pub new_session_public_key: String,
    /// The `participantDigest` wire field.
    #[serde(rename = "participantDigest")]
    pub participant_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `proof` wire field.
    pub proof: AuthDevicesConnectInfoGetRequestProof,
    /// The `requestId` wire field.
    #[serde(rename = "requestId")]
    pub request_id: String,
}
/// Generated schema type `AuthDevicesConnectInfoGetResponseEndpointsAuthMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesConnectInfoGetResponseEndpointsAuthMode {
    /// The `session_nkey` wire value.
    #[serde(rename = "session_nkey")]
    SessionNkey,
}
impl AuthDevicesConnectInfoGetResponseEndpointsAuthMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::SessionNkey => "session_nkey",
        }
    }
}
impl AsRef<str> for AuthDevicesConnectInfoGetResponseEndpointsAuthMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesConnectInfoGetResponseEndpointsAuthMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesConnectInfoGetResponseEndpointsAuthMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesConnectInfoGetResponseEndpointsAuthMode> for &str {
    fn eq(&self, other: &AuthDevicesConnectInfoGetResponseEndpointsAuthMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode {
    /// The `server_issued` wire value.
    #[serde(rename = "server_issued")]
    ServerIssued,
}
impl AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::ServerIssued => "server_issued",
        }
    }
}
impl AsRef<str> for AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode> for &str {
    fn eq(&self, other: &AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesConnectInfoGetResponseEndpoints`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesConnectInfoGetResponseEndpoints {
    /// The `authMode` wire field.
    #[serde(rename = "authMode")]
    pub auth_mode: AuthDevicesConnectInfoGetResponseEndpointsAuthMode,
    /// The `authorityMode` wire field.
    #[serde(rename = "authorityMode")]
    pub authority_mode: AuthDevicesConnectInfoGetResponseEndpointsAuthorityMode,
    /// The `maximumClockSkewMs` wire field.
    #[serde(rename = "maximumClockSkewMs")]
    pub maximum_clock_skew_ms: i64,
    /// The `native` wire field.
    pub native: Vec<String>,
    /// The `websocket` wire field.
    pub websocket: Vec<String>,
}
/// Generated schema type `AuthDevicesConnectInfoGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesConnectInfoGetResponse {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `endpoints` wire field.
    pub endpoints: AuthDevicesConnectInfoGetResponseEndpoints,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
}
/// Generated schema type `AuthDevicesDisableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesDisableRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDevicesDisableResponseDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesDisableResponseDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesDisableResponseDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesDisableResponseDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesDisableResponseDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesDisableResponseDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesDisableResponseDeviceAdministrativeApproval> for &str {
    fn eq(&self, other: &AuthDevicesDisableResponseDeviceAdministrativeApproval) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesDisableResponseDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesDisableResponseDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesDisableResponseDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesDisableResponseDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesDisableResponseDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesDisableResponseDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesDisableResponseDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDevicesDisableResponseDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesDisableResponseDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesDisableResponseDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesDisableResponseDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesDisableResponseDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesDisableResponseDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesDisableResponseDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesDisableResponseDeviceState> for &str {
    fn eq(&self, other: &AuthDevicesDisableResponseDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesDisableResponseDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesDisableResponseDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDevicesDisableResponseDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDevicesDisableResponseDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDevicesDisableResponseDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesDisableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesDisableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesDisableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesDisableResponse {
    /// The `device` wire field.
    pub device: AuthDevicesDisableResponseDevice,
    /// The `mutation` wire field.
    pub mutation: AuthDevicesDisableResponseMutation,
}
/// Generated schema type `AuthDevicesEnableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesEnableRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDevicesEnableResponseDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesEnableResponseDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesEnableResponseDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesEnableResponseDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesEnableResponseDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesEnableResponseDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesEnableResponseDeviceAdministrativeApproval> for &str {
    fn eq(&self, other: &AuthDevicesEnableResponseDeviceAdministrativeApproval) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesEnableResponseDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesEnableResponseDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesEnableResponseDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesEnableResponseDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesEnableResponseDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesEnableResponseDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesEnableResponseDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDevicesEnableResponseDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesEnableResponseDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesEnableResponseDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesEnableResponseDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesEnableResponseDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesEnableResponseDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesEnableResponseDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesEnableResponseDeviceState> for &str {
    fn eq(&self, other: &AuthDevicesEnableResponseDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesEnableResponseDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesEnableResponseDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDevicesEnableResponseDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDevicesEnableResponseDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDevicesEnableResponseDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesEnableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesEnableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesEnableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesEnableResponse {
    /// The `device` wire field.
    pub device: AuthDevicesEnableResponseDevice,
    /// The `mutation` wire field.
    pub mutation: AuthDevicesEnableResponseMutation,
}
/// Generated schema type `AuthDevicesListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesListRequestState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesListRequestState> for &str {
    fn eq(&self, other: &AuthDevicesListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthDevicesListRequestState>,
}
/// Generated schema type `AuthDevicesListResponseEntriesItemAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesListResponseEntriesItemAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesListResponseEntriesItemAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesListResponseEntriesItemAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesListResponseEntriesItemAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesListResponseEntriesItemAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesListResponseEntriesItemAdministrativeApproval> for &str {
    fn eq(&self, other: &AuthDevicesListResponseEntriesItemAdministrativeApproval) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesListResponseEntriesItemDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesListResponseEntriesItemDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesListResponseEntriesItemDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesListResponseEntriesItemDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesListResponseEntriesItemDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesListResponseEntriesItemDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesListResponseEntriesItemDelegationState> for &str {
    fn eq(&self, other: &AuthDevicesListResponseEntriesItemDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesListResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthDevicesListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesListResponseEntriesItem {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDevicesListResponseEntriesItemAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDevicesListResponseEntriesItemDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDevicesListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthDevicesListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthDevicesProvisionRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesProvisionRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: Option<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
}
/// Generated schema type `AuthDevicesProvisionResponseDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesProvisionResponseDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesProvisionResponseDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesProvisionResponseDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesProvisionResponseDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesProvisionResponseDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesProvisionResponseDeviceAdministrativeApproval> for &str {
    fn eq(&self, other: &AuthDevicesProvisionResponseDeviceAdministrativeApproval) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesProvisionResponseDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesProvisionResponseDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesProvisionResponseDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesProvisionResponseDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesProvisionResponseDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesProvisionResponseDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesProvisionResponseDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDevicesProvisionResponseDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesProvisionResponseDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesProvisionResponseDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesProvisionResponseDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesProvisionResponseDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesProvisionResponseDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesProvisionResponseDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesProvisionResponseDeviceState> for &str {
    fn eq(&self, other: &AuthDevicesProvisionResponseDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesProvisionResponseDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesProvisionResponseDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDevicesProvisionResponseDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDevicesProvisionResponseDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDevicesProvisionResponseDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesProvisionResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesProvisionResponse {
    /// The `device` wire field.
    pub device: AuthDevicesProvisionResponseDevice,
    /// The `provisioningSecret` wire field.
    #[serde(rename = "provisioningSecret")]
    pub provisioning_secret: Option<String>,
}
/// Generated schema type `AuthDevicesRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesRemoveRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthDevicesRemoveResponseDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesRemoveResponseDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesRemoveResponseDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesRemoveResponseDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesRemoveResponseDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesRemoveResponseDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesRemoveResponseDeviceAdministrativeApproval> for &str {
    fn eq(&self, other: &AuthDevicesRemoveResponseDeviceAdministrativeApproval) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesRemoveResponseDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesRemoveResponseDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesRemoveResponseDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesRemoveResponseDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesRemoveResponseDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesRemoveResponseDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesRemoveResponseDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDevicesRemoveResponseDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesRemoveResponseDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDevicesRemoveResponseDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDevicesRemoveResponseDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDevicesRemoveResponseDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDevicesRemoveResponseDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDevicesRemoveResponseDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDevicesRemoveResponseDeviceState> for &str {
    fn eq(&self, other: &AuthDevicesRemoveResponseDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDevicesRemoveResponseDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesRemoveResponseDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDevicesRemoveResponseDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDevicesRemoveResponseDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDevicesRemoveResponseDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesRemoveResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesRemoveResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDevicesRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDevicesRemoveResponse {
    /// The `device` wire field.
    pub device: AuthDevicesRemoveResponseDevice,
    /// The `mutation` wire field.
    pub mutation: AuthDevicesRemoveResponseMutation,
}
/// Generated schema type `AuthIdentityAuthorityGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityGetRequest {
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl
    AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions: Vec<AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
}
impl AuthIdentityAuthorityGetResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityGetResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityGetResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityGetResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityGetResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityGetResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityGetResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthIdentityAuthorityGetResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityGetResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityGetResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityGetResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityGetResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityGetResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityGetResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityGetResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthIdentityAuthorityGetResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthIdentityAuthorityGetResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthIdentityAuthorityGetResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthIdentityAuthorityGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityGetResponse {
    /// The `authority` wire field.
    pub authority: AuthIdentityAuthorityGetResponseAuthority,
}
/// Generated schema type `AuthIdentityAuthorityListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListRequestState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthIdentityAuthorityListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityListRequestState> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthIdentityAuthorityListRequestState>,
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
}
impl AuthIdentityAuthorityListResponseEntriesItemKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityListResponseEntriesItemKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityListResponseEntriesItemKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityListResponseEntriesItemKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityListResponseEntriesItemKind> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityListResponseEntriesItemKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityListResponseEntriesItemState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthIdentityAuthorityListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityListResponseEntriesItem {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthIdentityAuthorityListResponseEntriesItemDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthIdentityAuthorityListResponseEntriesItemKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthIdentityAuthorityListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthIdentityAuthorityListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthIdentityAuthorityListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthIdentityAuthorityRevokeRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityRevokeRequest {
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat {
    /// The `trellis.grant-set.v1` wire value.
    #[serde(rename = "trellis.grant-set.v1")]
    TrellisGrantSetV1,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::TrellisGrantSetV1 => "trellis.grant-set.v1",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// The `call` wire value.
    #[serde(rename = "call")]
    Call,
    /// The `invoke` wire value.
    #[serde(rename = "invoke")]
    Invoke,
    /// The `observe` wire value.
    #[serde(rename = "observe")]
    Observe,
    /// The `cancel` wire value.
    #[serde(rename = "cancel")]
    Cancel,
    /// The `control` wire value.
    #[serde(rename = "control")]
    Control,
    /// The `publish` wire value.
    #[serde(rename = "publish")]
    Publish,
    /// The `subscribe` wire value.
    #[serde(rename = "subscribe")]
    Subscribe,
    /// The `read` wire value.
    #[serde(rename = "read")]
    Read,
    /// The `write` wire value.
    #[serde(rename = "write")]
    Write,
    /// The `delete` wire value.
    #[serde(rename = "delete")]
    Delete,
    /// The `submit` wire value.
    #[serde(rename = "submit")]
    Submit,
    /// The `process` wire value.
    #[serde(rename = "process")]
    Process,
    /// The `consume` wire value.
    #[serde(rename = "consume")]
    Consume,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Call => "call",
            Self::Invoke => "invoke",
            Self::Observe => "observe",
            Self::Cancel => "cancel",
            Self::Control => "control",
            Self::Publish => "publish",
            Self::Subscribe => "subscribe",
            Self::Read => "read",
            Self::Write => "write",
            Self::Delete => "delete",
            Self::Submit => "submit",
            Self::Process => "process",
            Self::Consume => "consume",
        }
    }
}
impl AsRef<str>
    for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
    for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
    for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction
{
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction>
    for &str
{
    fn eq(
        &self,
        other: &AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface
{
    /// The `rpc` wire value.
    #[serde(rename = "rpc")]
    Rpc,
    /// The `operation` wire value.
    #[serde(rename = "operation")]
    Operation,
    /// The `event` wire value.
    #[serde(rename = "event")]
    Event,
    /// The `feed` wire value.
    #[serde(rename = "feed")]
    Feed,
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rpc => "rpc",
            Self::Operation => "operation",
            Self::Event => "event",
            Self::Feed => "feed",
            Self::State => "state",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource
{
    /// The `state` wire value.
    #[serde(rename = "state")]
    State,
    /// The `jobQueue` wire value.
    #[serde(rename = "jobQueue")]
    JobQueue,
    /// The `eventConsumer` wire value.
    #[serde(rename = "eventConsumer")]
    EventConsumer,
    /// The `kv` wire value.
    #[serde(rename = "kv")]
    Kv,
    /// The `store` wire value.
    #[serde(rename = "store")]
    Store,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::State => "state",
            Self::JobQueue => "jobQueue",
            Self::EventConsumer => "eventConsumer",
            Self::Kv => "kv",
            Self::Store => "store",
        }
    }
}
impl AsRef<str>
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str>
for AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<
    AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
> for &str {
    fn eq(
        &self,
        other: &AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTarget`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTarget {
    /// The `apiSurface` variant.
    #[serde(rename = "apiSurface")]
    ApiSurface {
        /// The `api` wire field.
        api: String,
        /// The `name` wire field.
        name: String,
        /// The `surface` wire field.
        surface: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetApiSurfaceSurface,
    },
    /// The `operationSignal` variant.
    #[serde(rename = "operationSignal")]
    OperationSignal {
        /// The `api` wire field.
        api: String,
        /// The `operation` wire field.
        operation: String,
        /// The `signal` wire field.
        signal: String,
    },
    /// The `participantResource` variant.
    #[serde(rename = "participantResource")]
    ParticipantResource {
        /// The `name` wire field.
        name: String,
        /// The `participant` wire field.
        participant: String,
        /// The `resource` wire field.
        resource: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTargetParticipantResourceResource,
    },
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItem {
    /// The `action` wire field.
    pub action: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemAction,
    /// The `target` wire field.
    pub target: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItemTarget,
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSet`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSet {
    /// The `format` wire field.
    pub format: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetFormat,
    /// The `permissions` wire field.
    pub permissions:
        Vec<AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSetPermissionsItem>,
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityKind {
    /// The `identity` wire value.
    #[serde(rename = "identity")]
    Identity,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Identity => "identity",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityRevokeResponseAuthorityKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityRevokeResponseAuthorityKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityRevokeResponseAuthorityKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityRevokeResponseAuthorityKind> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityRevokeResponseAuthorityKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthorityState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityAuthorityRevokeResponseAuthorityState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `accepted` wire value.
    #[serde(rename = "accepted")]
    Accepted,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthIdentityAuthorityRevokeResponseAuthorityState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthIdentityAuthorityRevokeResponseAuthorityState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityAuthorityRevokeResponseAuthorityState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityAuthorityRevokeResponseAuthorityState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityAuthorityRevokeResponseAuthorityState> for &str {
    fn eq(&self, other: &AuthIdentityAuthorityRevokeResponseAuthorityState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponseAuthority`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityRevokeResponseAuthority {
    /// The `acceptedNeedsDigest` wire field.
    #[serde(rename = "acceptedNeedsDigest")]
    pub accepted_needs_digest: String,
    /// The `authorityId` wire field.
    #[serde(rename = "authorityId")]
    pub authority_id: String,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `decision` wire field.
    pub decision: Option<BTreeMap<String, Value>>,
    /// The `desiredCapabilities` wire field.
    #[serde(rename = "desiredCapabilities")]
    pub desired_capabilities: Vec<String>,
    /// The `desiredGrantSet` wire field.
    #[serde(rename = "desiredGrantSet")]
    pub desired_grant_set: AuthIdentityAuthorityRevokeResponseAuthorityDesiredGrantSet,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `kind` wire field.
    pub kind: AuthIdentityAuthorityRevokeResponseAuthorityKind,
    /// The `materialization` wire field.
    pub materialization: Option<BTreeMap<String, Value>>,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthIdentityAuthorityRevokeResponseAuthorityState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthIdentityAuthorityRevokeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityAuthorityRevokeResponse {
    /// The `authority` wire field.
    pub authority: AuthIdentityAuthorityRevokeResponseAuthority,
}
/// Generated schema type `AuthIdentityGrantsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsListRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `user` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
/// Generated schema type `AuthIdentityGrantsListResponseEntriesItemContractEvidence`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsListResponseEntriesItemContractEvidence {
    /// The `contractDigest` wire field.
    #[serde(rename = "contractDigest")]
    pub contract_digest: String,
    /// The `contractId` wire field.
    #[serde(rename = "contractId")]
    pub contract_id: String,
}
/// Generated schema type `AuthIdentityGrantsListResponseEntriesItemIdentityAnchor`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthIdentityGrantsListResponseEntriesItemIdentityAnchor {
    /// The `web` variant.
    #[serde(rename = "web")]
    Web {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `origin` wire field.
        origin: String,
    },
    /// The `cli` variant.
    #[serde(rename = "cli")]
    Cli {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `sessionPublicKey` wire field.
        #[serde(rename = "sessionPublicKey")]
        session_public_key: String,
    },
    /// The `native` variant.
    #[serde(rename = "native")]
    Native {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `sessionPublicKey` wire field.
        #[serde(rename = "sessionPublicKey")]
        session_public_key: String,
    },
    /// The `device-user` variant.
    #[serde(rename = "device-user")]
    DeviceUser {
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `devicePublicKey` wire field.
        #[serde(rename = "devicePublicKey")]
        device_public_key: String,
    },
}
/// Generated schema type `AuthIdentityGrantsListResponseEntriesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthIdentityGrantsListResponseEntriesItemParticipantKind {
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl AuthIdentityGrantsListResponseEntriesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::App => "app",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for AuthIdentityGrantsListResponseEntriesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthIdentityGrantsListResponseEntriesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthIdentityGrantsListResponseEntriesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthIdentityGrantsListResponseEntriesItemParticipantKind> for &str {
    fn eq(&self, other: &AuthIdentityGrantsListResponseEntriesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthIdentityGrantsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsListResponseEntriesItem {
    /// The `capabilities` wire field.
    pub capabilities: Vec<String>,
    /// The `contractEvidence` wire field.
    #[serde(rename = "contractEvidence")]
    pub contract_evidence: AuthIdentityGrantsListResponseEntriesItemContractEvidence,
    /// The `description` wire field.
    pub description: String,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `grantedAt` wire field.
    #[serde(rename = "grantedAt")]
    pub granted_at: String,
    /// The `identityAnchor` wire field.
    #[serde(rename = "identityAnchor")]
    pub identity_anchor: AuthIdentityGrantsListResponseEntriesItemIdentityAnchor,
    /// The `identityGrantId` wire field.
    #[serde(rename = "identityGrantId")]
    pub identity_grant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthIdentityGrantsListResponseEntriesItemParticipantKind,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}
/// Generated schema type `AuthIdentityGrantsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsListResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<AuthIdentityGrantsListResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `AuthIdentityGrantsRevokeRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsRevokeRequest {
    /// The `identityGrantId` wire field.
    #[serde(rename = "identityGrantId")]
    pub identity_grant_id: String,
    /// The `user` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user: Option<String>,
}
/// Generated schema type `AuthIdentityGrantsRevokeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthIdentityGrantsRevokeResponse {
    /// The `success` wire field.
    pub success: bool,
}
/// Generated schema type `AuthPortalsGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGetRequest {
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
}
/// Generated schema type `AuthPortalsGetResponsePortalLoginSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGetResponsePortalLoginSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsGetResponsePortal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGetResponsePortal {
    /// The `builtIn` wire field.
    #[serde(rename = "builtIn")]
    pub built_in: bool,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabled` wire field.
    pub disabled: bool,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `entryUrl` wire field.
    #[serde(rename = "entryUrl")]
    pub entry_url: Option<String>,
    /// The `loginSettings` wire field.
    #[serde(rename = "loginSettings")]
    pub login_settings: AuthPortalsGetResponsePortalLoginSettings,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsGetResponseRoutesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGetResponseRoutesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: Option<String>,
    /// The `origin` wire field.
    pub origin: Option<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `priority` wire field.
    pub priority: i64,
    /// The `routeId` wire field.
    #[serde(rename = "routeId")]
    pub route_id: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGetResponse {
    /// The `portal` wire field.
    pub portal: AuthPortalsGetResponsePortal,
    /// The `routes` wire field.
    pub routes: Vec<AuthPortalsGetResponseRoutesItem>,
}
/// Generated schema type `AuthPortalsGrantOverridesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesListRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub portal_id: Option<String>,
}
/// Generated schema type `AuthPortalsGrantOverridesListResponseEntriesItemRoleMappingsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesListResponseEntriesItemRoleMappingsItem {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `role` wire field.
    pub role: String,
}
/// Generated schema type `AuthPortalsGrantOverridesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesListResponseEntriesItem {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `roleMappings` wire field.
    #[serde(rename = "roleMappings")]
    pub role_mappings: Vec<AuthPortalsGrantOverridesListResponseEntriesItemRoleMappingsItem>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsGrantOverridesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesListResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<AuthPortalsGrantOverridesListResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `AuthPortalsGrantOverridesPutRequestRoleMappingsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesPutRequestRoleMappingsItem {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `role` wire field.
    pub role: String,
}
/// Generated schema type `AuthPortalsGrantOverridesPutRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesPutRequest {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `roleMappings` wire field.
    #[serde(rename = "roleMappings")]
    pub role_mappings: Vec<AuthPortalsGrantOverridesPutRequestRoleMappingsItem>,
}
/// Generated schema type `AuthPortalsGrantOverridesPutResponsePolicyRoleMappingsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesPutResponsePolicyRoleMappingsItem {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `role` wire field.
    pub role: String,
}
/// Generated schema type `AuthPortalsGrantOverridesPutResponsePolicy`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesPutResponsePolicy {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `roleMappings` wire field.
    #[serde(rename = "roleMappings")]
    pub role_mappings: Vec<AuthPortalsGrantOverridesPutResponsePolicyRoleMappingsItem>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsGrantOverridesPutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesPutResponse {
    /// The `policy` wire field.
    pub policy: AuthPortalsGrantOverridesPutResponsePolicy,
}
/// Generated schema type `AuthPortalsGrantOverridesRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesRemoveRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
}
/// Generated schema type `AuthPortalsGrantOverridesRemoveResponseRemovedRoleMappingsItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesRemoveResponseRemovedRoleMappingsItem {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `role` wire field.
    pub role: String,
}
/// Generated schema type `AuthPortalsGrantOverridesRemoveResponseRemoved`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesRemoveResponseRemoved {
    /// The `capabilityGroupKeys` wire field.
    #[serde(rename = "capabilityGroupKeys")]
    pub capability_group_keys: Vec<String>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `directCapabilities` wire field.
    #[serde(rename = "directCapabilities")]
    pub direct_capabilities: Vec<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `roleMappings` wire field.
    #[serde(rename = "roleMappings")]
    pub role_mappings: Vec<AuthPortalsGrantOverridesRemoveResponseRemovedRoleMappingsItem>,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsGrantOverridesRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsGrantOverridesRemoveResponse {
    /// The `removed` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub removed: Option<AuthPortalsGrantOverridesRemoveResponseRemoved>,
}
/// Generated schema type `AuthPortalsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `disabled` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub disabled: Option<bool>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
}
/// Generated schema type `AuthPortalsListResponseEntriesItemLoginSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsListResponseEntriesItemLoginSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsListResponseEntriesItem {
    /// The `builtIn` wire field.
    #[serde(rename = "builtIn")]
    pub built_in: bool,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabled` wire field.
    pub disabled: bool,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `entryUrl` wire field.
    #[serde(rename = "entryUrl")]
    pub entry_url: Option<String>,
    /// The `loginSettings` wire field.
    #[serde(rename = "loginSettings")]
    pub login_settings: AuthPortalsListResponseEntriesItemLoginSettings,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthPortalsListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthPortalsLoginSettingsGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsGetRequest {
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
}
/// Generated schema type `AuthPortalsLoginSettingsGetResponseSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsGetResponseSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsLoginSettingsGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsGetResponse {
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `settings` wire field.
    pub settings: AuthPortalsLoginSettingsGetResponseSettings,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsLoginSettingsUpdateRequestSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsUpdateRequestSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsLoginSettingsUpdateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsUpdateRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `settings` wire field.
    pub settings: AuthPortalsLoginSettingsUpdateRequestSettings,
}
/// Generated schema type `AuthPortalsLoginSettingsUpdateResponseSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsUpdateResponseSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsLoginSettingsUpdateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsLoginSettingsUpdateResponse {
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `settings` wire field.
    pub settings: AuthPortalsLoginSettingsUpdateResponseSettings,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsPutRequestLoginSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsPutRequestLoginSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsPutRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsPutRequest {
    /// The `disabled` wire field.
    pub disabled: bool,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `entryUrl` wire field.
    #[serde(rename = "entryUrl")]
    pub entry_url: Option<String>,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `loginSettings` wire field.
    #[serde(rename = "loginSettings")]
    pub login_settings: AuthPortalsPutRequestLoginSettings,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
}
/// Generated schema type `AuthPortalsPutResponsePortalLoginSettings`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsPutResponsePortalLoginSettings {
    /// The `federatedRegistration` wire field.
    #[serde(rename = "federatedRegistration")]
    pub federated_registration: bool,
    /// The `localLogin` wire field.
    #[serde(rename = "localLogin")]
    pub local_login: bool,
    /// The `localRegistration` wire field.
    #[serde(rename = "localRegistration")]
    pub local_registration: bool,
    /// The `providers` wire field.
    pub providers: Option<Vec<String>>,
}
/// Generated schema type `AuthPortalsPutResponsePortal`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsPutResponsePortal {
    /// The `builtIn` wire field.
    #[serde(rename = "builtIn")]
    pub built_in: bool,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabled` wire field.
    pub disabled: bool,
    /// The `displayName` wire field.
    #[serde(rename = "displayName")]
    pub display_name: String,
    /// The `entryUrl` wire field.
    #[serde(rename = "entryUrl")]
    pub entry_url: Option<String>,
    /// The `loginSettings` wire field.
    #[serde(rename = "loginSettings")]
    pub login_settings: AuthPortalsPutResponsePortalLoginSettings,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsPutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsPutResponse {
    /// The `portal` wire field.
    pub portal: AuthPortalsPutResponsePortal,
}
/// Generated schema type `AuthPortalsRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRemoveRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
}
/// Generated schema type `AuthPortalsRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRemoveResponse {
    /// The `removed` wire field.
    pub removed: bool,
}
/// Generated schema type `AuthPortalsRoutesPutRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRoutesPutRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: Option<String>,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `origin` wire field.
    pub origin: Option<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `priority` wire field.
    pub priority: i64,
    /// The `routeId` wire field.
    #[serde(rename = "routeId")]
    pub route_id: Option<String>,
}
/// Generated schema type `AuthPortalsRoutesPutResponseRoute`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRoutesPutResponseRoute {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: Option<String>,
    /// The `origin` wire field.
    pub origin: Option<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `portalId` wire field.
    #[serde(rename = "portalId")]
    pub portal_id: String,
    /// The `priority` wire field.
    pub priority: i64,
    /// The `routeId` wire field.
    #[serde(rename = "routeId")]
    pub route_id: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthPortalsRoutesPutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRoutesPutResponse {
    /// The `route` wire field.
    pub route: AuthPortalsRoutesPutResponseRoute,
}
/// Generated schema type `AuthPortalsRoutesRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRoutesRemoveRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `routeId` wire field.
    #[serde(rename = "routeId")]
    pub route_id: String,
}
/// Generated schema type `AuthPortalsRoutesRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthPortalsRoutesRemoveResponse {
    /// The `removed` wire field.
    pub removed: bool,
}
/// Generated schema type `AuthServiceInstancesDisableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesDisableRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthServiceInstancesDisableResponseInstanceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesDisableResponseInstanceState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesDisableResponseInstanceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesDisableResponseInstanceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesDisableResponseInstanceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesDisableResponseInstanceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesDisableResponseInstanceState> for &str {
    fn eq(&self, other: &AuthServiceInstancesDisableResponseInstanceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesDisableResponseInstance`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesDisableResponseInstance {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthServiceInstancesDisableResponseInstanceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesDisableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesDisableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesDisableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesDisableResponse {
    /// The `instance` wire field.
    pub instance: AuthServiceInstancesDisableResponseInstance,
    /// The `mutation` wire field.
    pub mutation: AuthServiceInstancesDisableResponseMutation,
}
/// Generated schema type `AuthServiceInstancesEnableRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesEnableRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthServiceInstancesEnableResponseInstanceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesEnableResponseInstanceState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesEnableResponseInstanceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesEnableResponseInstanceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesEnableResponseInstanceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesEnableResponseInstanceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesEnableResponseInstanceState> for &str {
    fn eq(&self, other: &AuthServiceInstancesEnableResponseInstanceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesEnableResponseInstance`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesEnableResponseInstance {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthServiceInstancesEnableResponseInstanceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesEnableResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesEnableResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesEnableResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesEnableResponse {
    /// The `instance` wire field.
    pub instance: AuthServiceInstancesEnableResponseInstance,
    /// The `mutation` wire field.
    pub mutation: AuthServiceInstancesEnableResponseMutation,
}
/// Generated schema type `AuthServiceInstancesListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesListRequestState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesListRequestState> for &str {
    fn eq(&self, other: &AuthServiceInstancesListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthServiceInstancesListRequestState>,
}
/// Generated schema type `AuthServiceInstancesListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesListResponseEntriesItemState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthServiceInstancesListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesListResponseEntriesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthServiceInstancesListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthServiceInstancesListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthServiceInstancesProvisionRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesProvisionRequest {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: Option<String>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
}
/// Generated schema type `AuthServiceInstancesProvisionResponseInstanceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesProvisionResponseInstanceState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesProvisionResponseInstanceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesProvisionResponseInstanceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesProvisionResponseInstanceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesProvisionResponseInstanceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesProvisionResponseInstanceState> for &str {
    fn eq(&self, other: &AuthServiceInstancesProvisionResponseInstanceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesProvisionResponseInstance`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesProvisionResponseInstance {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthServiceInstancesProvisionResponseInstanceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesProvisionResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesProvisionResponse {
    /// The `instance` wire field.
    pub instance: AuthServiceInstancesProvisionResponseInstance,
}
/// Generated schema type `AuthServiceInstancesRemoveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesRemoveRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
}
/// Generated schema type `AuthServiceInstancesRemoveResponseInstanceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthServiceInstancesRemoveResponseInstanceState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
    /// The `stale` wire value.
    #[serde(rename = "stale")]
    Stale,
}
impl AuthServiceInstancesRemoveResponseInstanceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
            Self::Stale => "stale",
        }
    }
}
impl AsRef<str> for AuthServiceInstancesRemoveResponseInstanceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthServiceInstancesRemoveResponseInstanceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthServiceInstancesRemoveResponseInstanceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthServiceInstancesRemoveResponseInstanceState> for &str {
    fn eq(&self, other: &AuthServiceInstancesRemoveResponseInstanceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthServiceInstancesRemoveResponseInstance`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesRemoveResponseInstance {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: String,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthServiceInstancesRemoveResponseInstanceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesRemoveResponseMutation`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesRemoveResponseMutation {
    /// The `changed` wire field.
    pub changed: bool,
    /// The `resourceId` wire field.
    #[serde(rename = "resourceId")]
    pub resource_id: String,
    /// The `state` wire field.
    pub state: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthServiceInstancesRemoveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthServiceInstancesRemoveResponse {
    /// The `instance` wire field.
    pub instance: AuthServiceInstancesRemoveResponseInstance,
    /// The `mutation` wire field.
    pub mutation: AuthServiceInstancesRemoveResponseMutation,
}
/// Generated schema type `AuthSessionsListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsListRequestState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthSessionsListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthSessionsListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsListRequestState> for &str {
    fn eq(&self, other: &AuthSessionsListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub deployment_id: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub principal_id: Option<String>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthSessionsListRequestState>,
}
/// Generated schema type `AuthSessionsListResponseEntriesItemParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsListResponseEntriesItemParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl AuthSessionsListResponseEntriesItemParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::App => "app",
            Self::Device => "device",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for AuthSessionsListResponseEntriesItemParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsListResponseEntriesItemParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsListResponseEntriesItemParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsListResponseEntriesItemParticipantKind> for &str {
    fn eq(&self, other: &AuthSessionsListResponseEntriesItemParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsListResponseEntriesItemPrincipalKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsListResponseEntriesItemPrincipalKind {
    /// The `user` wire value.
    #[serde(rename = "user")]
    User,
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthSessionsListResponseEntriesItemPrincipalKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthSessionsListResponseEntriesItemPrincipalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsListResponseEntriesItemPrincipalKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsListResponseEntriesItemPrincipalKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsListResponseEntriesItemPrincipalKind> for &str {
    fn eq(&self, other: &AuthSessionsListResponseEntriesItemPrincipalKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsListResponseEntriesItemState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthSessionsListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthSessionsListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthSessionsListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsListResponseEntriesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `inboxPrefix` wire field.
    #[serde(rename = "inboxPrefix")]
    pub inbox_prefix: String,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthSessionsListResponseEntriesItemParticipantKind,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `principalKind` wire field.
    #[serde(rename = "principalKind")]
    pub principal_kind: AuthSessionsListResponseEntriesItemPrincipalKind,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The `sessionKeyId` wire field.
    #[serde(rename = "sessionKeyId")]
    pub session_key_id: String,
    /// The `sessionPublicKey` wire field.
    #[serde(rename = "sessionPublicKey")]
    pub session_public_key: String,
    /// The `state` wire field.
    pub state: AuthSessionsListResponseEntriesItemState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthSessionsListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthSessionsListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthSessionsLogoutResponseSessionParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsLogoutResponseSessionParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl AuthSessionsLogoutResponseSessionParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::App => "app",
            Self::Device => "device",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for AuthSessionsLogoutResponseSessionParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsLogoutResponseSessionParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsLogoutResponseSessionParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsLogoutResponseSessionParticipantKind> for &str {
    fn eq(&self, other: &AuthSessionsLogoutResponseSessionParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsLogoutResponseSessionPrincipalKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsLogoutResponseSessionPrincipalKind {
    /// The `user` wire value.
    #[serde(rename = "user")]
    User,
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthSessionsLogoutResponseSessionPrincipalKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthSessionsLogoutResponseSessionPrincipalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsLogoutResponseSessionPrincipalKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsLogoutResponseSessionPrincipalKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsLogoutResponseSessionPrincipalKind> for &str {
    fn eq(&self, other: &AuthSessionsLogoutResponseSessionPrincipalKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsLogoutResponseSessionState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsLogoutResponseSessionState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthSessionsLogoutResponseSessionState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthSessionsLogoutResponseSessionState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsLogoutResponseSessionState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsLogoutResponseSessionState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsLogoutResponseSessionState> for &str {
    fn eq(&self, other: &AuthSessionsLogoutResponseSessionState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsLogoutResponseSession`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsLogoutResponseSession {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `inboxPrefix` wire field.
    #[serde(rename = "inboxPrefix")]
    pub inbox_prefix: String,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthSessionsLogoutResponseSessionParticipantKind,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `principalKind` wire field.
    #[serde(rename = "principalKind")]
    pub principal_kind: AuthSessionsLogoutResponseSessionPrincipalKind,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The `sessionKeyId` wire field.
    #[serde(rename = "sessionKeyId")]
    pub session_key_id: String,
    /// The `sessionPublicKey` wire field.
    #[serde(rename = "sessionPublicKey")]
    pub session_public_key: String,
    /// The `state` wire field.
    pub state: AuthSessionsLogoutResponseSessionState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthSessionsLogoutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsLogoutResponse {
    /// The `kickedConnections` wire field.
    #[serde(rename = "kickedConnections")]
    pub kicked_connections: i64,
    /// The `session` wire field.
    pub session: AuthSessionsLogoutResponseSession,
}
/// Generated schema type `AuthSessionsMeResponseSessionParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsMeResponseSessionParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl AuthSessionsMeResponseSessionParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::App => "app",
            Self::Device => "device",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for AuthSessionsMeResponseSessionParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsMeResponseSessionParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsMeResponseSessionParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsMeResponseSessionParticipantKind> for &str {
    fn eq(&self, other: &AuthSessionsMeResponseSessionParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsMeResponseSessionPrincipalKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsMeResponseSessionPrincipalKind {
    /// The `user` wire value.
    #[serde(rename = "user")]
    User,
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthSessionsMeResponseSessionPrincipalKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthSessionsMeResponseSessionPrincipalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsMeResponseSessionPrincipalKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsMeResponseSessionPrincipalKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsMeResponseSessionPrincipalKind> for &str {
    fn eq(&self, other: &AuthSessionsMeResponseSessionPrincipalKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsMeResponseSessionState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsMeResponseSessionState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthSessionsMeResponseSessionState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthSessionsMeResponseSessionState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsMeResponseSessionState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsMeResponseSessionState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsMeResponseSessionState> for &str {
    fn eq(&self, other: &AuthSessionsMeResponseSessionState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsMeResponseSession`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsMeResponseSession {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `inboxPrefix` wire field.
    #[serde(rename = "inboxPrefix")]
    pub inbox_prefix: String,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthSessionsMeResponseSessionParticipantKind,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `principalKind` wire field.
    #[serde(rename = "principalKind")]
    pub principal_kind: AuthSessionsMeResponseSessionPrincipalKind,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The `sessionKeyId` wire field.
    #[serde(rename = "sessionKeyId")]
    pub session_key_id: String,
    /// The `sessionPublicKey` wire field.
    #[serde(rename = "sessionPublicKey")]
    pub session_public_key: String,
    /// The `state` wire field.
    pub state: AuthSessionsMeResponseSessionState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthSessionsMeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsMeResponse {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: Option<String>,
    /// The `session` wire field.
    pub session: AuthSessionsMeResponseSession,
    /// The `user` wire field.
    pub user: Option<BTreeMap<String, Value>>,
}
/// Generated schema type `AuthSessionsRevokeRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsRevokeRequest {
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: Option<i64>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
/// Generated schema type `AuthSessionsRevokeResponseSessionParticipantKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsRevokeResponseSessionParticipantKind {
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `app` wire value.
    #[serde(rename = "app")]
    App,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
    /// The `agent` wire value.
    #[serde(rename = "agent")]
    Agent,
}
impl AuthSessionsRevokeResponseSessionParticipantKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Service => "service",
            Self::App => "app",
            Self::Device => "device",
            Self::Agent => "agent",
        }
    }
}
impl AsRef<str> for AuthSessionsRevokeResponseSessionParticipantKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsRevokeResponseSessionParticipantKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsRevokeResponseSessionParticipantKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsRevokeResponseSessionParticipantKind> for &str {
    fn eq(&self, other: &AuthSessionsRevokeResponseSessionParticipantKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsRevokeResponseSessionPrincipalKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsRevokeResponseSessionPrincipalKind {
    /// The `user` wire value.
    #[serde(rename = "user")]
    User,
    /// The `service` wire value.
    #[serde(rename = "service")]
    Service,
    /// The `device` wire value.
    #[serde(rename = "device")]
    Device,
}
impl AuthSessionsRevokeResponseSessionPrincipalKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::User => "user",
            Self::Service => "service",
            Self::Device => "device",
        }
    }
}
impl AsRef<str> for AuthSessionsRevokeResponseSessionPrincipalKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsRevokeResponseSessionPrincipalKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsRevokeResponseSessionPrincipalKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsRevokeResponseSessionPrincipalKind> for &str {
    fn eq(&self, other: &AuthSessionsRevokeResponseSessionPrincipalKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsRevokeResponseSessionState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthSessionsRevokeResponseSessionState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthSessionsRevokeResponseSessionState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthSessionsRevokeResponseSessionState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthSessionsRevokeResponseSessionState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthSessionsRevokeResponseSessionState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthSessionsRevokeResponseSessionState> for &str {
    fn eq(&self, other: &AuthSessionsRevokeResponseSessionState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthSessionsRevokeResponseSession`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsRevokeResponseSession {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: Option<i64>,
    /// The `inboxPrefix` wire field.
    #[serde(rename = "inboxPrefix")]
    pub inbox_prefix: String,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `participantArtifactDigest` wire field.
    #[serde(rename = "participantArtifactDigest")]
    pub participant_artifact_digest: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `participantKind` wire field.
    #[serde(rename = "participantKind")]
    pub participant_kind: AuthSessionsRevokeResponseSessionParticipantKind,
    /// The `participantNeedsDigest` wire field.
    #[serde(rename = "participantNeedsDigest")]
    pub participant_needs_digest: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `principalKind` wire field.
    #[serde(rename = "principalKind")]
    pub principal_kind: AuthSessionsRevokeResponseSessionPrincipalKind,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
    /// The `sessionKeyId` wire field.
    #[serde(rename = "sessionKeyId")]
    pub session_key_id: String,
    /// The `sessionPublicKey` wire field.
    #[serde(rename = "sessionPublicKey")]
    pub session_public_key: String,
    /// The `state` wire field.
    pub state: AuthSessionsRevokeResponseSessionState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthSessionsRevokeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsRevokeResponse {
    /// The `kickedConnections` wire field.
    #[serde(rename = "kickedConnections")]
    pub kicked_connections: i64,
    /// The `session` wire field.
    pub session: AuthSessionsRevokeResponseSession,
}
/// Generated schema type `AuthUserIdentitiesListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUserIdentitiesListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
}
/// Generated schema type `AuthUserIdentitiesListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUserIdentitiesListResponseEntriesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `lastSeenAt` wire field.
    #[serde(rename = "lastSeenAt")]
    pub last_seen_at: i64,
    /// The `observedEmail` wire field.
    #[serde(rename = "observedEmail")]
    pub observed_email: Option<String>,
    /// The `observedName` wire field.
    #[serde(rename = "observedName")]
    pub observed_name: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `subject` wire field.
    pub subject: String,
    /// The `username` wire field.
    pub username: Option<String>,
}
/// Generated schema type `AuthUserIdentitiesListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUserIdentitiesListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthUserIdentitiesListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthUserIdentitiesUnlinkRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUserIdentitiesUnlinkRequest {
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `providerId` wire field.
    #[serde(rename = "providerId")]
    pub provider_id: String,
    /// The `subject` wire field.
    pub subject: String,
}
/// Generated schema type `AuthUserIdentitiesUnlinkResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUserIdentitiesUnlinkResponse {
    /// The `unlinked` wire field.
    pub unlinked: bool,
}
/// Generated schema type `AuthUsersCreateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersCreateRequest {
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
}
/// Generated schema type `AuthUsersCreateResponseUserState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersCreateResponseUserState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthUsersCreateResponseUserState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthUsersCreateResponseUserState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersCreateResponseUserState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersCreateResponseUserState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersCreateResponseUserState> for &str {
    fn eq(&self, other: &AuthUsersCreateResponseUserState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersCreateResponseUser`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersCreateResponseUser {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthUsersCreateResponseUserState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersCreateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersCreateResponse {
    /// The `user` wire field.
    pub user: AuthUsersCreateResponseUser,
}
/// Generated schema type `AuthUsersGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersGetRequest {
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
}
/// Generated schema type `AuthUsersGetResponseUserState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersGetResponseUserState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthUsersGetResponseUserState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthUsersGetResponseUserState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersGetResponseUserState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersGetResponseUserState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersGetResponseUserState> for &str {
    fn eq(&self, other: &AuthUsersGetResponseUserState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersGetResponseUser`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersGetResponseUser {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthUsersGetResponseUserState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersGetResponse {
    /// The `user` wire field.
    pub user: AuthUsersGetResponseUser,
}
/// Generated schema type `AuthUsersIdentityLinkCreateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersIdentityLinkCreateRequest {
    /// The `allowedProviders` wire field.
    #[serde(rename = "allowedProviders")]
    pub allowed_providers: Vec<String>,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `returnTarget` wire field.
    #[serde(rename = "returnTarget")]
    pub return_target: Option<String>,
}
/// Generated schema type `AuthUsersIdentityLinkCreateResponseFlowKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersIdentityLinkCreateResponseFlowKind {
    /// The `identity_link` wire value.
    #[serde(rename = "identity_link")]
    IdentityLink,
}
impl AuthUsersIdentityLinkCreateResponseFlowKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::IdentityLink => "identity_link",
        }
    }
}
impl AsRef<str> for AuthUsersIdentityLinkCreateResponseFlowKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersIdentityLinkCreateResponseFlowKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersIdentityLinkCreateResponseFlowKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersIdentityLinkCreateResponseFlowKind> for &str {
    fn eq(&self, other: &AuthUsersIdentityLinkCreateResponseFlowKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersIdentityLinkCreateResponseFlow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersIdentityLinkCreateResponseFlow {
    /// The `allowedProviders` wire field.
    #[serde(rename = "allowedProviders")]
    pub allowed_providers: Vec<String>,
    /// The `completionUrl` wire field.
    #[serde(rename = "completionUrl")]
    pub completion_url: String,
    /// The `consumedAt` wire field.
    #[serde(rename = "consumedAt")]
    pub consumed_at: Option<i64>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    /// The `flowId` wire field.
    #[serde(rename = "flowId")]
    pub flow_id: String,
    /// The `kind` wire field.
    pub kind: AuthUsersIdentityLinkCreateResponseFlowKind,
    /// The `returnTarget` wire field.
    #[serde(rename = "returnTarget")]
    pub return_target: Option<String>,
    /// The `targetPrincipalId` wire field.
    #[serde(rename = "targetPrincipalId")]
    pub target_principal_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersIdentityLinkCreateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersIdentityLinkCreateResponse {
    /// The `flow` wire field.
    pub flow: AuthUsersIdentityLinkCreateResponseFlow,
}
/// Generated schema type `AuthUsersListRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersListRequestState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthUsersListRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthUsersListRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersListRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersListRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersListRequestState> for &str {
    fn eq(&self, other: &AuthUsersListRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersListRequest {
    /// The `cursor` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cursor: Option<String>,
    /// The `limit` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<i64>,
    /// The `state` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state: Option<AuthUsersListRequestState>,
}
/// Generated schema type `AuthUsersListResponseEntriesItemState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersListResponseEntriesItemState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthUsersListResponseEntriesItemState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthUsersListResponseEntriesItemState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersListResponseEntriesItemState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersListResponseEntriesItemState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersListResponseEntriesItemState> for &str {
    fn eq(&self, other: &AuthUsersListResponseEntriesItemState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersListResponseEntriesItem {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthUsersListResponseEntriesItemState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersListResponse {
    /// The `entries` wire field.
    pub entries: Vec<AuthUsersListResponseEntriesItem>,
    /// The `nextCursor` wire field.
    #[serde(rename = "nextCursor")]
    pub next_cursor: Option<String>,
}
/// Generated schema type `AuthUsersPasswordChangeRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersPasswordChangeRequest {
    /// The `currentPassword` wire field.
    #[serde(rename = "currentPassword")]
    pub current_password: String,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `newPassword` wire field.
    #[serde(rename = "newPassword")]
    pub new_password: String,
}
/// Generated schema type `AuthUsersPasswordChangeResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersPasswordChangeResponse {
    /// The `changedAt` wire field.
    #[serde(rename = "changedAt")]
    pub changed_at: i64,
    /// The `revokedSessionCount` wire field.
    #[serde(rename = "revokedSessionCount")]
    pub revoked_session_count: i64,
}
/// Generated schema type `AuthUsersPasswordResetCreateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersPasswordResetCreateRequest {
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `returnTarget` wire field.
    #[serde(rename = "returnTarget")]
    pub return_target: Option<String>,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
}
/// Generated schema type `AuthUsersPasswordResetCreateResponseFlowKind`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersPasswordResetCreateResponseFlowKind {
    /// The `password_reset` wire value.
    #[serde(rename = "password_reset")]
    PasswordReset,
}
impl AuthUsersPasswordResetCreateResponseFlowKind {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::PasswordReset => "password_reset",
        }
    }
}
impl AsRef<str> for AuthUsersPasswordResetCreateResponseFlowKind {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersPasswordResetCreateResponseFlowKind {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersPasswordResetCreateResponseFlowKind {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersPasswordResetCreateResponseFlowKind> for &str {
    fn eq(&self, other: &AuthUsersPasswordResetCreateResponseFlowKind) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersPasswordResetCreateResponseFlow`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersPasswordResetCreateResponseFlow {
    /// The `allowedProviders` wire field.
    #[serde(rename = "allowedProviders")]
    pub allowed_providers: Vec<String>,
    /// The `completionUrl` wire field.
    #[serde(rename = "completionUrl")]
    pub completion_url: String,
    /// The `consumedAt` wire field.
    #[serde(rename = "consumedAt")]
    pub consumed_at: Option<i64>,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    /// The `flowId` wire field.
    #[serde(rename = "flowId")]
    pub flow_id: String,
    /// The `kind` wire field.
    pub kind: AuthUsersPasswordResetCreateResponseFlowKind,
    /// The `returnTarget` wire field.
    #[serde(rename = "returnTarget")]
    pub return_target: Option<String>,
    /// The `targetPrincipalId` wire field.
    #[serde(rename = "targetPrincipalId")]
    pub target_principal_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersPasswordResetCreateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersPasswordResetCreateResponse {
    /// The `flow` wire field.
    pub flow: AuthUsersPasswordResetCreateResponseFlow,
}
/// Generated schema type `AuthUsersResolveRequestSelector`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "kind")]
pub enum AuthUsersResolveRequestSelector {
    /// The `user` variant.
    #[serde(rename = "user")]
    User {
        /// The `userId` wire field.
        #[serde(rename = "userId")]
        user_id: String,
    },
    /// The `provider` variant.
    #[serde(rename = "provider")]
    Provider {
        /// The `providerId` wire field.
        #[serde(rename = "providerId")]
        provider_id: String,
        /// The `providerSubject` wire field.
        #[serde(rename = "providerSubject")]
        provider_subject: String,
    },
}
/// Generated schema type `AuthUsersResolveRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersResolveRequest {
    /// The `selector` wire field.
    pub selector: AuthUsersResolveRequestSelector,
}
/// Generated schema type `AuthUsersResolveResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersResolveResponse {
    /// The `user` wire field.
    pub user: Option<BTreeMap<String, Value>>,
}
/// Generated schema type `AuthUsersUpdateRequestState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersUpdateRequestState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
}
impl AuthUsersUpdateRequestState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
        }
    }
}
impl AsRef<str> for AuthUsersUpdateRequestState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersUpdateRequestState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersUpdateRequestState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersUpdateRequestState> for &str {
    fn eq(&self, other: &AuthUsersUpdateRequestState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersUpdateRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersUpdateRequest {
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `expectedVersion` wire field.
    #[serde(rename = "expectedVersion")]
    pub expected_version: i64,
    /// The `idempotencyKey` wire field.
    #[serde(rename = "idempotencyKey")]
    pub idempotency_key: String,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
    /// The `state` wire field.
    pub state: AuthUsersUpdateRequestState,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
}
/// Generated schema type `AuthUsersUpdateResponseUserState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthUsersUpdateResponseUserState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthUsersUpdateResponseUserState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthUsersUpdateResponseUserState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthUsersUpdateResponseUserState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthUsersUpdateResponseUserState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthUsersUpdateResponseUserState> for &str {
    fn eq(&self, other: &AuthUsersUpdateResponseUserState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthUsersUpdateResponseUser`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersUpdateResponseUser {
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `disabledAt` wire field.
    #[serde(rename = "disabledAt")]
    pub disabled_at: Option<i64>,
    /// The `email` wire field.
    pub email: Option<String>,
    /// The `image` wire field.
    pub image: Option<String>,
    /// The `name` wire field.
    pub name: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `revokedAt` wire field.
    #[serde(rename = "revokedAt")]
    pub revoked_at: Option<i64>,
    /// The `state` wire field.
    pub state: AuthUsersUpdateResponseUserState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `userId` wire field.
    #[serde(rename = "userId")]
    pub user_id: String,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthUsersUpdateResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthUsersUpdateResponse {
    /// The `user` wire field.
    pub user: AuthUsersUpdateResponseUser,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveInput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolveInput {
    /// The `confirmationCode` wire field.
    #[serde(rename = "confirmationCode")]
    pub confirmation_code: String,
    /// The `flowId` wire field.
    #[serde(rename = "flowId")]
    pub flow_id: String,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveProgressState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesResolveProgressState {
    /// The `waiting` wire value.
    #[serde(rename = "waiting")]
    Waiting,
    /// The `review_pending` wire value.
    #[serde(rename = "review_pending")]
    ReviewPending,
    /// The `delegation_pending` wire value.
    #[serde(rename = "delegation_pending")]
    DelegationPending,
}
impl AuthDeviceUserAuthoritiesResolveProgressState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Waiting => "waiting",
            Self::ReviewPending => "review_pending",
            Self::DelegationPending => "delegation_pending",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesResolveProgressState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesResolveProgressState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesResolveProgressState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesResolveProgressState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesResolveProgressState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveProgress`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolveProgress {
    /// The `retryAfterMs` wire field.
    #[serde(rename = "retryAfterMs")]
    pub retry_after_ms: i64,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesResolveProgressState,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval> for &str {
    fn eq(
        &self,
        other: &AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval,
    ) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState {
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `missing` wire value.
    #[serde(rename = "missing")]
    Missing,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Active => "active",
            Self::Missing => "missing",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputDeviceState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesResolveOutputDeviceState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `active` wire value.
    #[serde(rename = "active")]
    Active,
    /// The `disabled` wire value.
    #[serde(rename = "disabled")]
    Disabled,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesResolveOutputDeviceState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Active => "active",
            Self::Disabled => "disabled",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesResolveOutputDeviceState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesResolveOutputDeviceState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesResolveOutputDeviceState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesResolveOutputDeviceState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesResolveOutputDeviceState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputDevice`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolveOutputDevice {
    /// The `administrativeApproval` wire field.
    #[serde(rename = "administrativeApproval")]
    pub administrative_approval: AuthDeviceUserAuthoritiesResolveOutputDeviceAdministrativeApproval,
    /// The `createdAt` wire field.
    #[serde(rename = "createdAt")]
    pub created_at: i64,
    /// The `delegationExpiresAt` wire field.
    #[serde(rename = "delegationExpiresAt")]
    pub delegation_expires_at: Option<i64>,
    /// The `delegationRequired` wire field.
    #[serde(rename = "delegationRequired")]
    pub delegation_required: bool,
    /// The `delegationState` wire field.
    #[serde(rename = "delegationState")]
    pub delegation_state: AuthDeviceUserAuthoritiesResolveOutputDeviceDelegationState,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `identityKeyId` wire field.
    #[serde(rename = "identityKeyId")]
    pub identity_key_id: Option<String>,
    /// The `identityPublicKey` wire field.
    #[serde(rename = "identityPublicKey")]
    pub identity_public_key: Option<String>,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: Option<String>,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesResolveOutputDeviceState,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: i64,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputReviewState`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuthDeviceUserAuthoritiesResolveOutputReviewState {
    /// The `pending` wire value.
    #[serde(rename = "pending")]
    Pending,
    /// The `approved` wire value.
    #[serde(rename = "approved")]
    Approved,
    /// The `rejected` wire value.
    #[serde(rename = "rejected")]
    Rejected,
    /// The `expired` wire value.
    #[serde(rename = "expired")]
    Expired,
    /// The `revoked` wire value.
    #[serde(rename = "revoked")]
    Revoked,
}
impl AuthDeviceUserAuthoritiesResolveOutputReviewState {
    /// Return the contract wire value.
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Approved => "approved",
            Self::Rejected => "rejected",
            Self::Expired => "expired",
            Self::Revoked => "revoked",
        }
    }
}
impl AsRef<str> for AuthDeviceUserAuthoritiesResolveOutputReviewState {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl std::fmt::Display for AuthDeviceUserAuthoritiesResolveOutputReviewState {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl PartialEq<&str> for AuthDeviceUserAuthoritiesResolveOutputReviewState {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}
impl PartialEq<AuthDeviceUserAuthoritiesResolveOutputReviewState> for &str {
    fn eq(&self, other: &AuthDeviceUserAuthoritiesResolveOutputReviewState) -> bool {
        *self == other.as_str()
    }
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutputReview`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolveOutputReview {
    /// The `activatedByUserPrincipalId` wire field.
    #[serde(rename = "activatedByUserPrincipalId")]
    pub activated_by_user_principal_id: Option<String>,
    /// The `decidedAt` wire field.
    #[serde(rename = "decidedAt")]
    pub decided_at: Option<i64>,
    /// The `decidedBy` wire field.
    #[serde(rename = "decidedBy")]
    pub decided_by: Option<String>,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `devicePrincipalId` wire field.
    #[serde(rename = "devicePrincipalId")]
    pub device_principal_id: String,
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    pub expires_at: i64,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `requestedAt` wire field.
    #[serde(rename = "requestedAt")]
    pub requested_at: i64,
    /// The `reviewId` wire field.
    #[serde(rename = "reviewId")]
    pub review_id: String,
    /// The `state` wire field.
    pub state: AuthDeviceUserAuthoritiesResolveOutputReviewState,
    /// The `version` wire field.
    pub version: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolveOutput`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolveOutput {
    /// The `authority` wire field.
    pub authority: Option<BTreeMap<String, Value>>,
    /// The `device` wire field.
    pub device: AuthDeviceUserAuthoritiesResolveOutputDevice,
    /// The `review` wire field.
    pub review: AuthDeviceUserAuthoritiesResolveOutputReview,
}
/// Generated schema type `AuthConnectionsClosedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsClosedEvent {
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
/// Generated schema type `AuthConnectionsKickedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsKickedEvent {
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
/// Generated schema type `AuthConnectionsOpenedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthConnectionsOpenedEvent {
    /// The `clientId` wire field.
    #[serde(rename = "clientId")]
    pub client_id: String,
    /// The `connectionId` wire field.
    #[serde(rename = "connectionId")]
    pub connection_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `serverId` wire field.
    #[serde(rename = "serverId")]
    pub server_id: String,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
/// Generated schema type `AuthDeviceUserAuthoritiesApprovedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesApprovedEvent {
    /// The `approvedBy` wire field.
    #[serde(rename = "approvedBy")]
    pub approved_by: String,
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
}
/// Generated schema type `AuthDeviceUserAuthoritiesRequestedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesRequestedEvent {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `userPrincipalId` wire field.
    #[serde(rename = "userPrincipalId")]
    pub user_principal_id: String,
}
/// Generated schema type `AuthDeviceUserAuthoritiesResolvedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesResolvedEvent {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `state` wire field.
    pub state: String,
}
/// Generated schema type `AuthDeviceUserAuthoritiesReviewRequestedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthDeviceUserAuthoritiesReviewRequestedEvent {
    /// The `deploymentId` wire field.
    #[serde(rename = "deploymentId")]
    pub deployment_id: String,
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `instanceId` wire field.
    #[serde(rename = "instanceId")]
    pub instance_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `reviewId` wire field.
    #[serde(rename = "reviewId")]
    pub review_id: String,
}
/// Generated schema type `AuthSessionsRevokedEvent`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthSessionsRevokedEvent {
    /// The `eventId` wire field.
    #[serde(rename = "eventId")]
    pub event_id: String,
    /// The `occurredAt` wire field.
    #[serde(rename = "occurredAt")]
    pub occurred_at: i64,
    /// The `participantId` wire field.
    #[serde(rename = "participantId")]
    pub participant_id: String,
    /// The `principalId` wire field.
    #[serde(rename = "principalId")]
    pub principal_id: String,
    /// The `reason` wire field.
    pub reason: Option<String>,
    /// The `revokedBy` wire field.
    #[serde(rename = "revokedBy")]
    pub revoked_by: Option<String>,
    /// The `sessionId` wire field.
    #[serde(rename = "sessionId")]
    pub session_id: String,
}
/// Generated schema type `AuthErrorDetails`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AuthErrorDetails {
    /// The `code` wire field.
    pub code: String,
    /// The `field` wire field.
    pub field: Option<String>,
    /// The `message` wire field.
    pub message: String,
    /// The `retryable` wire field.
    pub retryable: bool,
}
