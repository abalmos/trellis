use std::future::Future;
use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::{Map, Value};
use trellis_contracts::PageResponse;

use crate::client::{TrellisClient, TrellisClientError};

const GET_SUBJECT: &str = "rpc.v1.State.Get";
const PUT_SUBJECT: &str = "rpc.v1.State.Put";
const DELETE_SUBJECT: &str = "rpc.v1.State.Delete";
const LIST_SUBJECT: &str = "rpc.v1.State.List";

/// Transport used by typed state store helpers.
pub trait StateTransport {
    /// Send one JSON request to a Trellis-owned state RPC subject.
    fn request_state_json<'a>(
        &'a self,
        subject: &'static str,
        body: Value,
    ) -> impl Future<Output = Result<Value, TrellisClientError>> + Send + 'a;
}

impl StateTransport for TrellisClient {
    async fn request_state_json(
        &self,
        subject: &'static str,
        body: Value,
    ) -> Result<Value, TrellisClientError> {
        self.request_json_value(subject, &body).await
    }
}

/// Expected revision behavior for state puts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[doc = concat!("Public Trellis value set `", stringify!(ExpectedPutRevision), "`.")]
pub enum ExpectedPutRevision {
    /// Omit `expectedRevision` for unconditional create-or-overwrite.
    #[default]
    Unconditional,
    /// Send `expectedRevision: null` for create-if-absent.
    CreateIfAbsent,
    /// Send a concrete revision for update-if-current-revision-matches.
    Revision(String),
}

/// Options for state put requests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(PutStateOptions), "`.")]
pub struct PutStateOptions {
    #[doc = concat!("The `", stringify!(ttl_ms), "` value.")]
    pub ttl_ms: Option<u64>,
    #[doc = concat!("The `", stringify!(expected_revision), "` value.")]
    pub expected_revision: ExpectedPutRevision,
}

/// Options for state delete requests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(DeleteStateOptions), "`.")]
pub struct DeleteStateOptions {
    #[doc = concat!("The `", stringify!(expected_revision), "` value.")]
    pub expected_revision: Option<String>,
}

/// Options for map state list requests.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(ListStateOptions), "`.")]
pub struct ListStateOptions {
    #[doc = concat!("The `", stringify!(offset), "` value.")]
    pub offset: Option<u64>,
    #[doc = concat!("The `", stringify!(limit), "` value.")]
    pub limit: Option<u64>,
}

/// One current state entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StateEntry<TValue = Value> {
    #[doc = concat!("The `", stringify!(value), "` value.")]
    pub value: TValue,
    #[doc = concat!("The `", stringify!(revision), "` value.")]
    pub revision: String,
    #[doc = concat!("The `", stringify!(updated_at), "` value.")]
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: Option<String>,
}

/// One current map state entry.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MapStateEntry<TValue = Value> {
    #[doc = concat!("The `", stringify!(key), "` value.")]
    pub key: String,
    #[doc = concat!("The `", stringify!(value), "` value.")]
    pub value: TValue,
    #[doc = concat!("The `", stringify!(revision), "` value.")]
    pub revision: String,
    #[doc = concat!("The `", stringify!(updated_at), "` value.")]
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(expires_at), "` value.")]
    pub expires_at: Option<String>,
}

/// Current or migration-required state value.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateValue<TEntry, TMigrationEntry = StateEntry<Value>> {
    Current(TEntry),
    MigrationRequired(StateMigrationRequired<TMigrationEntry>),
}

/// State entry that must be migrated by the caller before it is current.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StateMigrationRequired<TEntry = StateEntry<Value>> {
    #[doc = concat!("The `", stringify!(migration_required), "` value.")]
    pub migration_required: bool,
    #[doc = concat!("The `", stringify!(entry), "` value.")]
    pub entry: TEntry,
    #[doc = concat!("The `", stringify!(state_version), "` value.")]
    pub state_version: String,
    #[doc = concat!("The `", stringify!(current_state_version), "` value.")]
    pub current_state_version: String,
    #[doc = concat!("The `", stringify!(writer_contract_digest), "` value.")]
    pub writer_contract_digest: String,
}

/// Result returned by state get requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum StateGetResult<TEntry, TMigrationEntry = StateEntry<Value>> {
    Found {
        #[serde(deserialize_with = "deserialize_true")]
        found: bool,
        entry: TEntry,
    },
    Missing {
        #[serde(deserialize_with = "deserialize_false")]
        found: bool,
    },
    MigrationRequired(StateMigrationRequired<TMigrationEntry>),
}

/// Result returned by state put requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StatePutResult<TEntry, TMigrationEntry = StateEntry<Value>> {
    #[doc = concat!("The `", stringify!(applied), "` value.")]
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(found), "` value.")]
    pub found: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[doc = concat!("The `", stringify!(entry), "` value.")]
    pub entry: Option<StateValue<TEntry, TMigrationEntry>>,
}

/// Result returned by state delete requests.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StateDeleteResult {
    #[doc = concat!("The `", stringify!(deleted), "` value.")]
    pub deleted: bool,
}

/// Result returned by map state list requests.
pub type MapStateListResult<TValue> =
    PageResponse<StateValue<MapStateEntry<TValue>, MapStateEntry<Value>>>;

/// Typed client for a contract-declared value state store.
#[derive(Debug)]
pub struct ValueStateStore<'a, TTransport, TValue> {
    transport: &'a TTransport,
    store: &'static str,
    _value: PhantomData<TValue>,
}

impl<'a, TTransport, TValue> ValueStateStore<'a, TTransport, TValue> {
    /// Create a typed value state store helper.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(transport: &'a TTransport, store: &'static str) -> Self {
        Self {
            transport,
            store,
            _value: PhantomData,
        }
    }
}

impl<TTransport, TValue> ValueStateStore<'_, TTransport, TValue>
where
    TTransport: StateTransport,
    TValue: Serialize + DeserializeOwned,
{
    /// Read the current value state entry.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(get), "`.")]
    pub async fn get(&self) -> Result<StateGetResult<StateEntry<TValue>>, TrellisClientError> {
        let response = self
            .transport
            .request_state_json(GET_SUBJECT, request_with_store(self.store))
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Write the value state entry unconditionally.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(put), "`.")]
    pub async fn put(
        &self,
        value: &TValue,
    ) -> Result<StatePutResult<StateEntry<TValue>>, TrellisClientError> {
        self.put_with_options(value, &PutStateOptions::default())
            .await
    }

    /// Write the value state entry with TTL and revision options.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(put_with_options), "`.")]
    pub async fn put_with_options(
        &self,
        value: &TValue,
        options: &PutStateOptions,
    ) -> Result<StatePutResult<StateEntry<TValue>>, TrellisClientError> {
        let response = self
            .transport
            .request_state_json(PUT_SUBJECT, put_request(self.store, None, value, options)?)
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Delete the value state entry.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete), "`.")]
    pub async fn delete(&self) -> Result<StateDeleteResult, TrellisClientError> {
        self.delete_with_options(&DeleteStateOptions::default())
            .await
    }

    /// Delete the value state entry with revision options.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete_with_options), "`.")]
    pub async fn delete_with_options(
        &self,
        options: &DeleteStateOptions,
    ) -> Result<StateDeleteResult, TrellisClientError> {
        let response = self
            .transport
            .request_state_json(DELETE_SUBJECT, delete_request(self.store, None, options))
            .await?;
        Ok(serde_json::from_value(response)?)
    }
}

/// Typed client for a contract-declared map state store.
#[derive(Debug)]
pub struct MapStateStore<'a, TTransport, TValue> {
    transport: &'a TTransport,
    store: &'static str,
    prefix: String,
    _value: PhantomData<TValue>,
}

impl<'a, TTransport, TValue> MapStateStore<'a, TTransport, TValue> {
    /// Create a typed map state store helper.
    #[doc = concat!("Trellis API operation `", stringify!(new), "`.")]
    pub fn new(transport: &'a TTransport, store: &'static str) -> Self {
        Self {
            transport,
            store,
            prefix: String::new(),
            _value: PhantomData,
        }
    }

    /// Return a view of this map store rooted at a composed path prefix.
    #[doc = concat!("Trellis API operation `", stringify!(prefix), "`.")]
    pub fn prefix(&self, path: &str) -> Self {
        Self {
            transport: self.transport,
            store: self.store,
            prefix: join_state_path(&self.prefix, path),
            _value: PhantomData,
        }
    }
}

impl<TTransport, TValue> MapStateStore<'_, TTransport, TValue>
where
    TTransport: StateTransport,
    TValue: Serialize + DeserializeOwned,
{
    /// Read one map state entry by key.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(get), "`.")]
    pub async fn get(
        &self,
        key: &str,
    ) -> Result<StateGetResult<MapStateEntry<TValue>, MapStateEntry<Value>>, TrellisClientError>
    {
        let response = self
            .transport
            .request_state_json(GET_SUBJECT, key_request(self.store, &self.prefix, key))
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Write one map state entry unconditionally.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(put), "`.")]
    pub async fn put(
        &self,
        key: &str,
        value: &TValue,
    ) -> Result<StatePutResult<MapStateEntry<TValue>, MapStateEntry<Value>>, TrellisClientError>
    {
        self.put_with_options(key, value, &PutStateOptions::default())
            .await
    }

    /// Write one map state entry with TTL and revision options.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(put_with_options), "`.")]
    pub async fn put_with_options(
        &self,
        key: &str,
        value: &TValue,
        options: &PutStateOptions,
    ) -> Result<StatePutResult<MapStateEntry<TValue>, MapStateEntry<Value>>, TrellisClientError>
    {
        let composed_key = join_state_path(&self.prefix, key);
        let response = self
            .transport
            .request_state_json(
                PUT_SUBJECT,
                put_request(self.store, Some(&composed_key), value, options)?,
            )
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// Delete one map state entry.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete), "`.")]
    pub async fn delete(&self, key: &str) -> Result<StateDeleteResult, TrellisClientError> {
        self.delete_with_options(key, &DeleteStateOptions::default())
            .await
    }

    /// Delete one map state entry with revision options.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(delete_with_options), "`.")]
    pub async fn delete_with_options(
        &self,
        key: &str,
        options: &DeleteStateOptions,
    ) -> Result<StateDeleteResult, TrellisClientError> {
        let composed_key = join_state_path(&self.prefix, key);
        let response = self
            .transport
            .request_state_json(
                DELETE_SUBJECT,
                delete_request(self.store, Some(&composed_key), options),
            )
            .await?;
        Ok(serde_json::from_value(response)?)
    }

    /// List map entries under the current prefix.
    #[doc = concat!("Asynchronous Trellis API operation `", stringify!(list), "`.")]
    pub async fn list(
        &self,
        options: &ListStateOptions,
    ) -> Result<MapStateListResult<TValue>, TrellisClientError> {
        let response = self
            .transport
            .request_state_json(
                LIST_SUBJECT,
                list_request(self.store, &self.prefix, options),
            )
            .await?;
        Ok(serde_json::from_value(response)?)
    }
}

fn request_with_store(store: &'static str) -> Value {
    let mut request = Map::new();
    request.insert("store".to_string(), Value::String(store.to_string()));
    Value::Object(request)
}

fn key_request(store: &'static str, prefix: &str, key: &str) -> Value {
    let mut request = Map::new();
    request.insert("store".to_string(), Value::String(store.to_string()));
    request.insert(
        "key".to_string(),
        Value::String(join_state_path(prefix, key)),
    );
    Value::Object(request)
}

fn put_request<TValue: Serialize>(
    store: &'static str,
    key: Option<&str>,
    value: &TValue,
    options: &PutStateOptions,
) -> Result<Value, TrellisClientError> {
    let mut request = Map::new();
    request.insert("store".to_string(), Value::String(store.to_string()));
    if let Some(key) = key {
        request.insert("key".to_string(), Value::String(key.to_string()));
    }
    request.insert("value".to_string(), serde_json::to_value(value)?);
    if let Some(ttl_ms) = options.ttl_ms {
        request.insert("ttlMs".to_string(), Value::from(ttl_ms));
    }
    match &options.expected_revision {
        ExpectedPutRevision::Unconditional => {}
        ExpectedPutRevision::CreateIfAbsent => {
            request.insert("expectedRevision".to_string(), Value::Null);
        }
        ExpectedPutRevision::Revision(revision) => {
            request.insert(
                "expectedRevision".to_string(),
                Value::String(revision.clone()),
            );
        }
    }
    Ok(Value::Object(request))
}

fn delete_request(store: &'static str, key: Option<&str>, options: &DeleteStateOptions) -> Value {
    let mut request = Map::new();
    request.insert("store".to_string(), Value::String(store.to_string()));
    if let Some(key) = key {
        request.insert("key".to_string(), Value::String(key.to_string()));
    }
    if let Some(revision) = &options.expected_revision {
        request.insert(
            "expectedRevision".to_string(),
            Value::String(revision.clone()),
        );
    }
    Value::Object(request)
}

fn list_request(store: &'static str, prefix: &str, options: &ListStateOptions) -> Value {
    let mut request = Map::new();
    request.insert("store".to_string(), Value::String(store.to_string()));
    if !prefix.is_empty() {
        request.insert("prefix".to_string(), Value::String(prefix.to_string()));
    }
    request.insert(
        "offset".to_string(),
        Value::from(options.offset.unwrap_or(0)),
    );
    request.insert(
        "limit".to_string(),
        Value::from(options.limit.unwrap_or(100)),
    );
    Value::Object(request)
}

fn join_state_path(left: &str, right: &str) -> String {
    if left.is_empty() {
        right.to_owned()
    } else {
        format!("{left}/{right}")
    }
}

fn deserialize_true<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = bool::deserialize(deserializer)?;
    if value {
        Ok(value)
    } else {
        Err(serde::de::Error::custom("expected true"))
    }
}

fn deserialize_false<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = bool::deserialize(deserializer)?;
    if value {
        Err(serde::de::Error::custom("expected false"))
    } else {
        Ok(value)
    }
}
