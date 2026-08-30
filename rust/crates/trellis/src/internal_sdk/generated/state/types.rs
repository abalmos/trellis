//! Shared request and response types for `trellis.state@v1`.
use serde::{Deserialize, Serialize};
use serde_json::Value;
fn deserialize_optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: serde::Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}
/// Generated schema type `StateAdminDeleteRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "scope")]
pub enum StateAdminDeleteRequest {
    /// The `userApp` variant.
    #[serde(rename = "userApp")]
    UserApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `expectedRevision` wire field.
        #[serde(rename = "expectedRevision")]
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_revision: Option<String>,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `store` wire field.
        store: String,
        /// The `userId` wire field.
        #[serde(rename = "userId")]
        user_id: String,
    },
    /// The `deviceApp` variant.
    #[serde(rename = "deviceApp")]
    DeviceApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `deviceId` wire field.
        #[serde(rename = "deviceId")]
        device_id: String,
        /// The `expectedRevision` wire field.
        #[serde(rename = "expectedRevision")]
        #[serde(skip_serializing_if = "Option::is_none")]
        expected_revision: Option<String>,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `store` wire field.
        store: String,
    },
}
/// Generated schema type `StateAdminDeleteResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateAdminDeleteResponse {
    /// The `deleted` wire field.
    pub deleted: bool,
}
/// Generated schema type `StateAdminGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "scope")]
pub enum StateAdminGetRequest {
    /// The `userApp` variant.
    #[serde(rename = "userApp")]
    UserApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `store` wire field.
        store: String,
        /// The `userId` wire field.
        #[serde(rename = "userId")]
        user_id: String,
    },
    /// The `deviceApp` variant.
    #[serde(rename = "deviceApp")]
    DeviceApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `deviceId` wire field.
        #[serde(rename = "deviceId")]
        device_id: String,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `store` wire field.
        store: String,
    },
}
/// Generated schema type `StateAdminGetResponseVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateAdminGetResponseVariant2Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateAdminGetResponseVariant3Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateAdminGetResponseVariant3Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateAdminGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateAdminGetResponse {
    /// The `Variant3` variant.
    Variant3 {
        /// The `currentStateVersion` wire field.
        #[serde(rename = "currentStateVersion")]
        current_state_version: String,
        /// The `entry` wire field.
        entry: StateAdminGetResponseVariant3Entry,
        /// The `migrationRequired` wire field.
        #[serde(rename = "migrationRequired")]
        migration_required: bool,
        /// The `stateVersion` wire field.
        #[serde(rename = "stateVersion")]
        state_version: String,
        /// The `writerContractDigest` wire field.
        #[serde(rename = "writerContractDigest")]
        writer_contract_digest: String,
    },
    /// The `Variant2` variant.
    Variant2 {
        /// The `entry` wire field.
        entry: StateAdminGetResponseVariant2Entry,
        /// The `found` wire field.
        found: bool,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `found` wire field.
        found: bool,
    },
}
/// Generated schema type `StateAdminListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "scope")]
pub enum StateAdminListRequest {
    /// The `userApp` variant.
    #[serde(rename = "userApp")]
    UserApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `limit` wire field.
        limit: i64,
        /// The `offset` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<i64>,
        /// The `prefix` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        /// The `store` wire field.
        store: String,
        /// The `userId` wire field.
        #[serde(rename = "userId")]
        user_id: String,
    },
    /// The `deviceApp` variant.
    #[serde(rename = "deviceApp")]
    DeviceApp {
        /// The `contractDigest` wire field.
        #[serde(rename = "contractDigest")]
        contract_digest: String,
        /// The `contractId` wire field.
        #[serde(rename = "contractId")]
        contract_id: String,
        /// The `deviceId` wire field.
        #[serde(rename = "deviceId")]
        device_id: String,
        /// The `limit` wire field.
        limit: i64,
        /// The `offset` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        offset: Option<i64>,
        /// The `prefix` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        prefix: Option<String>,
        /// The `store` wire field.
        store: String,
    },
}
/// Generated schema type `StateAdminListResponseEntriesItemVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateAdminListResponseEntriesItemVariant2Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateAdminListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateAdminListResponseEntriesItem {
    /// The `Variant2` variant.
    Variant2 {
        /// The `currentStateVersion` wire field.
        #[serde(rename = "currentStateVersion")]
        current_state_version: String,
        /// The `entry` wire field.
        entry: StateAdminListResponseEntriesItemVariant2Entry,
        /// The `migrationRequired` wire field.
        #[serde(rename = "migrationRequired")]
        migration_required: bool,
        /// The `stateVersion` wire field.
        #[serde(rename = "stateVersion")]
        state_version: String,
        /// The `writerContractDigest` wire field.
        #[serde(rename = "writerContractDigest")]
        writer_contract_digest: String,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `expiresAt` wire field.
        #[serde(rename = "expiresAt")]
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<String>,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `revision` wire field.
        revision: String,
        /// The `updatedAt` wire field.
        #[serde(rename = "updatedAt")]
        updated_at: String,
        /// The `value` wire field.
        value: Value,
    },
}
/// Generated schema type `StateAdminListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateAdminListResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<StateAdminListResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `StateDeleteRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateDeleteRequest {
    /// The `expectedRevision` wire field.
    #[serde(rename = "expectedRevision")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expected_revision: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `store` wire field.
    pub store: String,
}
/// Generated schema type `StateDeleteResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateDeleteResponse {
    /// The `deleted` wire field.
    pub deleted: bool,
}
/// Generated schema type `StateGetRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateGetRequest {
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `store` wire field.
    pub store: String,
}
/// Generated schema type `StateGetResponseVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateGetResponseVariant2Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateGetResponseVariant3Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateGetResponseVariant3Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateGetResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateGetResponse {
    /// The `Variant3` variant.
    Variant3 {
        /// The `currentStateVersion` wire field.
        #[serde(rename = "currentStateVersion")]
        current_state_version: String,
        /// The `entry` wire field.
        entry: StateGetResponseVariant3Entry,
        /// The `migrationRequired` wire field.
        #[serde(rename = "migrationRequired")]
        migration_required: bool,
        /// The `stateVersion` wire field.
        #[serde(rename = "stateVersion")]
        state_version: String,
        /// The `writerContractDigest` wire field.
        #[serde(rename = "writerContractDigest")]
        writer_contract_digest: String,
    },
    /// The `Variant2` variant.
    Variant2 {
        /// The `entry` wire field.
        entry: StateGetResponseVariant2Entry,
        /// The `found` wire field.
        found: bool,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `found` wire field.
        found: bool,
    },
}
/// Generated schema type `StateListRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateListRequest {
    /// The `limit` wire field.
    pub limit: i64,
    /// The `offset` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<i64>,
    /// The `prefix` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prefix: Option<String>,
    /// The `store` wire field.
    pub store: String,
}
/// Generated schema type `StateListResponseEntriesItemVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateListResponseEntriesItemVariant2Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StateListResponseEntriesItem`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateListResponseEntriesItem {
    /// The `Variant2` variant.
    Variant2 {
        /// The `currentStateVersion` wire field.
        #[serde(rename = "currentStateVersion")]
        current_state_version: String,
        /// The `entry` wire field.
        entry: StateListResponseEntriesItemVariant2Entry,
        /// The `migrationRequired` wire field.
        #[serde(rename = "migrationRequired")]
        migration_required: bool,
        /// The `stateVersion` wire field.
        #[serde(rename = "stateVersion")]
        state_version: String,
        /// The `writerContractDigest` wire field.
        #[serde(rename = "writerContractDigest")]
        writer_contract_digest: String,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `expiresAt` wire field.
        #[serde(rename = "expiresAt")]
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<String>,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `revision` wire field.
        revision: String,
        /// The `updatedAt` wire field.
        #[serde(rename = "updatedAt")]
        updated_at: String,
        /// The `value` wire field.
        value: Value,
    },
}
/// Generated schema type `StateListResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StateListResponse {
    /// The `count` wire field.
    pub count: i64,
    /// The `entries` wire field.
    pub entries: Vec<StateListResponseEntriesItem>,
    /// The `limit` wire field.
    pub limit: i64,
    /// The `nextOffset` wire field.
    #[serde(rename = "nextOffset")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next_offset: Option<i64>,
    /// The `offset` wire field.
    pub offset: i64,
}
/// Generated schema type `StatePutRequest`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatePutRequest {
    /// The `expectedRevision` wire field.
    #[serde(rename = "expectedRevision")]
    #[serde(
        default,
        deserialize_with = "deserialize_optional_nullable",
        skip_serializing_if = "Option::is_none"
    )]
    pub expected_revision: Option<Option<String>>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `store` wire field.
    pub store: String,
    /// The `ttlMs` wire field.
    #[serde(rename = "ttlMs")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ttl_ms: Option<i64>,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StatePutResponseVariant1Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatePutResponseVariant1Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StatePutResponseVariant2EntryVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct StatePutResponseVariant2EntryVariant2Entry {
    /// The `expiresAt` wire field.
    #[serde(rename = "expiresAt")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    /// The `key` wire field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    /// The `revision` wire field.
    pub revision: String,
    /// The `updatedAt` wire field.
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
    /// The `value` wire field.
    pub value: Value,
}
/// Generated schema type `StatePutResponseVariant2Entry`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StatePutResponseVariant2Entry {
    /// The `Variant2` variant.
    Variant2 {
        /// The `currentStateVersion` wire field.
        #[serde(rename = "currentStateVersion")]
        current_state_version: String,
        /// The `entry` wire field.
        entry: StatePutResponseVariant2EntryVariant2Entry,
        /// The `migrationRequired` wire field.
        #[serde(rename = "migrationRequired")]
        migration_required: bool,
        /// The `stateVersion` wire field.
        #[serde(rename = "stateVersion")]
        state_version: String,
        /// The `writerContractDigest` wire field.
        #[serde(rename = "writerContractDigest")]
        writer_contract_digest: String,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `expiresAt` wire field.
        #[serde(rename = "expiresAt")]
        #[serde(skip_serializing_if = "Option::is_none")]
        expires_at: Option<String>,
        /// The `key` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        key: Option<String>,
        /// The `revision` wire field.
        revision: String,
        /// The `updatedAt` wire field.
        #[serde(rename = "updatedAt")]
        updated_at: String,
        /// The `value` wire field.
        value: Value,
    },
}
/// Generated schema type `StatePutResponse`.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StatePutResponse {
    /// The `Variant2` variant.
    Variant2 {
        /// The `applied` wire field.
        applied: bool,
        /// The `entry` wire field.
        #[serde(skip_serializing_if = "Option::is_none")]
        entry: Option<StatePutResponseVariant2Entry>,
        /// The `found` wire field.
        found: bool,
    },
    /// The `Variant1` variant.
    Variant1 {
        /// The `applied` wire field.
        applied: bool,
        /// The `entry` wire field.
        entry: StatePutResponseVariant1Entry,
    },
}
