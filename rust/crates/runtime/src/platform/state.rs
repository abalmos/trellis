//! Platform-owned Trellis State RPC runtime.
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;

use async_nats::jetstream::{self, consumer, kv};
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine as _;
use bytes::Bytes;
use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use time::format_description::well_known::Rfc3339;
use time::OffsetDateTime;
use trellis_protocol::{
    parse_api, AuthorizationPrincipalKind, ParticipantKind, ParticipantResourceKind,
    PermissionAction, PermissionAtom, PermissionTarget, StateKind,
};
use trellis_rs::service::{
    internal::run_builtin_authenticated_router, DeclaredRpcError, RequestContext, Router,
    ServerError, ValidationIssue,
};
use trellis_runtime_apis::state::rpc::{
    StateAdminDeleteRpc, StateAdminGetRpc, StateAdminListRpc, StateDeleteRpc, StateGetRpc,
    StateListRpc, StatePutRpc,
};

use super::auth::verifier::RuntimeAuthVerifier;
use super::auth::{
    AccountRepository, AuthorityEvidenceRepository, AuthorityRepository, ParticipantBindingRecord,
    ParticipantBindingState, PrincipalKind, SqliteAuthorizationStore,
};
use crate::shutdown::StopHandle;
use crate::supervisor::RuntimeError;

const API_ID: &str = "trellis.state@v1";
const BUCKET: &str = "trellis_state";
const STREAM: &str = "KV_trellis_state";
const SUBJECT_PREFIX: &str = "$KV.trellis_state.";
const SUBJECTS: &[&str] = &[
    "rpc.v1.State.Get",
    "rpc.v1.State.Put",
    "rpc.v1.State.Delete",
    "rpc.v1.State.List",
    "rpc.v1.State.Admin.Get",
    "rpc.v1.State.Admin.List",
    "rpc.v1.State.Admin.Delete",
];

#[derive(Clone)]
pub(crate) struct StateRuntime {
    repository: SqliteAuthorizationStore,
    verifier: RuntimeAuthVerifier,
    store: kv::Store,
    jetstream: jetstream::Context,
}

#[derive(Clone, Copy)]
enum Scope {
    UserApp,
    DeviceApp,
}

impl Scope {
    fn as_str(self) -> &'static str {
        match self {
            Self::UserApp => "userApp",
            Self::DeviceApp => "deviceApp",
        }
    }
}

#[derive(Clone)]
struct Declaration {
    scope: Scope,
    owner_id: String,
    contract_id: String,
    contract_digest: String,
    store: String,
    kind: StateKind,
    schema: Value,
    state_version: String,
    accepted_versions: BTreeMap<String, Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Request {
    store: String,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    value: Option<Value>,
    #[serde(default)]
    ttl_ms: Option<u64>,
    #[serde(default, deserialize_with = "optional_nullable")]
    expected_revision: Option<Option<String>>,
}

fn optional_nullable<'de, D, T>(deserializer: D) -> Result<Option<Option<T>>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<T>::deserialize(deserializer).map(Some)
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct AdminRequest {
    scope: String,
    contract_id: String,
    contract_digest: String,
    store: String,
    #[serde(default)]
    user_id: Option<String>,
    #[serde(default)]
    device_id: Option<String>,
    #[serde(default)]
    key: Option<String>,
    #[serde(default)]
    prefix: Option<String>,
    #[serde(default)]
    offset: Option<u64>,
    #[serde(default)]
    limit: Option<u64>,
    #[serde(default)]
    expected_revision: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredEnvelope {
    value: Value,
    state_version: String,
    writer_contract_digest: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_at: Option<String>,
}

struct PhysicalEntry {
    revision: u64,
    envelope: StoredEnvelope,
}

impl StateRuntime {
    pub(crate) async fn start(
        nats: async_nats::Client,
        repository: SqliteAuthorizationStore,
        verifier: RuntimeAuthVerifier,
    ) -> Result<Self, RuntimeError> {
        let jetstream = jetstream::new(nats);
        let config = kv::Config {
            bucket: BUCKET.to_owned(),
            history: 1,
            max_age: Duration::ZERO,
            storage: jetstream::stream::StorageType::File,
            ..Default::default()
        };
        let store = match jetstream.get_key_value(BUCKET).await {
            Ok(store) => store,
            Err(open_error) => match jetstream.create_key_value(config).await {
                Ok(store) => store,
                Err(create_error) => jetstream.get_key_value(BUCKET).await.map_err(|retry_error| {
                    RuntimeError::Platform(format!(
                        "failed to open {BUCKET} ({open_error}), create it ({create_error}), or reopen it after a possible concurrent create ({retry_error})"
                    ))
                })?,
            },
        };
        let status = store.status().await.map_err(|error| {
            RuntimeError::Platform(format!("failed to inspect {BUCKET}: {error}"))
        })?;
        if status.history() != 1
            || status.max_age() != Duration::ZERO
            || status.info.config.storage != jetstream::stream::StorageType::File
        {
            return Err(RuntimeError::Platform(format!(
                "{BUCKET} has incompatible history, TTL, or storage configuration"
            )));
        }
        Ok(Self {
            repository,
            verifier,
            store,
            jetstream,
        })
    }

    pub(crate) async fn run(self, stop: StopHandle) -> Result<(), RuntimeError> {
        let router = self.router();
        let loop_future = run_builtin_authenticated_router(
            self.jetstream.client().clone(),
            API_ID,
            SUBJECTS,
            router,
            self.verifier.clone(),
        );
        tokio::select! {
            result = loop_future => result.map_err(|error| RuntimeError::Platform(error.to_string())),
            () = stop.stopped() => Ok(()),
        }
    }

    fn router(&self) -> Router {
        let mut router = Router::new();
        let state = self.clone();
        router.register_rpc::<StateGetRpc, _, _>(move |context, input| {
            let state = state.clone();
            async move { output(state.get(context, serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StatePutRpc, _, _>(move |context, input| {
            let state = state.clone();
            async move { output(state.put(context, serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StateDeleteRpc, _, _>(move |context, input| {
            let state = state.clone();
            async move { output(state.delete(context, serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StateListRpc, _, _>(move |context, input| {
            let state = state.clone();
            async move { output(state.list(context, serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StateAdminGetRpc, _, _>(move |_context, input| {
            let state = state.clone();
            async move { output(state.admin_get(serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StateAdminListRpc, _, _>(move |_context, input| {
            let state = state.clone();
            async move { output(state.admin_list(serde_json::to_value(input)?).await?) }
        });
        let state = self.clone();
        router.register_rpc::<StateAdminDeleteRpc, _, _>(move |_context, input| {
            let state = state.clone();
            async move { output(state.admin_delete(serde_json::to_value(input)?).await?) }
        });
        router
    }

    async fn get(&self, context: RequestContext, value: Value) -> Result<Value, ServerError> {
        let request: Request = serde_json::from_value(value)?;
        let declaration = self.normal_declaration(&context, &request.store).await?;
        self.require_resource(&context, &declaration, PermissionAction::Read)?;
        self.validate_key(&declaration, request.key.as_deref(), false)?;
        self.get_at(&declaration, request.key.as_deref()).await
    }

    async fn put(&self, context: RequestContext, value: Value) -> Result<Value, ServerError> {
        let request: Request = serde_json::from_value(value)?;
        let declaration = self.normal_declaration(&context, &request.store).await?;
        self.require_resource(&context, &declaration, PermissionAction::Write)?;
        self.validate_key(&declaration, request.key.as_deref(), false)?;
        let value = request
            .value
            .ok_or_else(|| validation("/value", "value is required"))?;
        validate_schema(&declaration.schema, &value, "/value", false)?;
        let expected = request
            .expected_revision
            .map(|value| value.map(|value| parse_revision(&value)).transpose())
            .transpose()?;
        let now = OffsetDateTime::now_utc();
        let envelope = StoredEnvelope {
            value,
            state_version: declaration.state_version.clone(),
            writer_contract_digest: declaration.contract_digest.clone(),
            updated_at: timestamp(now)?,
            expires_at: request
                .ttl_ms
                .map(|ttl| {
                    i64::try_from(ttl)
                        .ok()
                        .and_then(|ttl| now.checked_add(time::Duration::milliseconds(ttl)))
                        .ok_or_else(|| validation("/ttlMs", "ttlMs exceeds the supported range"))
                })
                .transpose()?
                .map(timestamp)
                .transpose()?,
        };
        let key = physical_key(&declaration, request.key.as_deref())?;
        let current = if expected.is_some() {
            self.live_entry(&key).await?
        } else {
            None
        };
        match expected {
            None => {
                let revision = self
                    .store
                    .put(&key, encode(&envelope)?)
                    .await
                    .map_err(kv_error)?;
                Ok(
                    json!({"applied": true, "entry": public_entry(request.key.as_deref(), revision, &envelope)}),
                )
            }
            Some(None) if current.is_some() => {
                Ok(conflict(current, request.key.as_deref(), &declaration)?)
            }
            Some(None) => match self.store.create(&key, encode(&envelope)?).await {
                Ok(revision) => Ok(
                    json!({"applied": true, "entry": public_entry(request.key.as_deref(), revision, &envelope)}),
                ),
                Err(_error) => match self.live_entry(&key).await? {
                    Some(entry) => Ok(conflict(Some(entry), request.key.as_deref(), &declaration)?),
                    None => match self.store.create(&key, encode(&envelope)?).await {
                        Ok(revision) => Ok(
                            json!({"applied": true, "entry": public_entry(request.key.as_deref(), revision, &envelope)}),
                        ),
                        Err(error) => match self.live_entry(&key).await? {
                            Some(entry) => {
                                Ok(conflict(Some(entry), request.key.as_deref(), &declaration)?)
                            }
                            None => Err(kv_error(error)),
                        },
                    },
                },
            },
            Some(Some(expected_revision)) => match current {
                None => Ok(json!({"applied": false, "found": false})),
                Some(ref entry) if entry.revision != expected_revision => Ok(conflict(
                    Some(entry.clone_entry()),
                    request.key.as_deref(),
                    &declaration,
                )?),
                Some(_) => match self
                    .store
                    .update(&key, encode(&envelope)?, expected_revision)
                    .await
                {
                    Ok(revision) => Ok(
                        json!({"applied": true, "entry": public_entry(request.key.as_deref(), revision, &envelope)}),
                    ),
                    Err(error) => {
                        let current = self.live_entry(&key).await?;
                        if update_failure_was_race(
                            current.as_ref().map(|entry| entry.revision),
                            expected_revision,
                        ) {
                            Ok(conflict(current, request.key.as_deref(), &declaration)?)
                        } else {
                            Err(kv_error(error))
                        }
                    }
                },
            },
        }
    }

    async fn delete(&self, context: RequestContext, value: Value) -> Result<Value, ServerError> {
        let request: Request = serde_json::from_value(value)?;
        let declaration = self.normal_declaration(&context, &request.store).await?;
        self.require_resource(&context, &declaration, PermissionAction::Delete)?;
        self.validate_key(&declaration, request.key.as_deref(), false)?;
        let expected = request
            .expected_revision
            .flatten()
            .map(|value| parse_revision(&value))
            .transpose()?;
        self.delete_at(&declaration, request.key.as_deref(), expected)
            .await
    }

    async fn list(&self, context: RequestContext, value: Value) -> Result<Value, ServerError> {
        let request: Request = serde_json::from_value(value)?;
        let declaration = self.normal_declaration(&context, &request.store).await?;
        self.require_resource(&context, &declaration, PermissionAction::Read)?;
        self.validate_key(&declaration, request.prefix.as_deref(), true)?;
        self.list_at(
            &declaration,
            request.prefix.as_deref(),
            request.offset.unwrap_or(0),
            request.limit.unwrap_or(0),
        )
        .await
    }

    async fn admin_get(&self, value: Value) -> Result<Value, ServerError> {
        let request: AdminRequest = serde_json::from_value(value)?;
        let declaration = self.admin_declaration(&request).await?;
        self.validate_key(&declaration, request.key.as_deref(), false)?;
        self.get_at(&declaration, request.key.as_deref()).await
    }

    async fn admin_list(&self, value: Value) -> Result<Value, ServerError> {
        let request: AdminRequest = serde_json::from_value(value)?;
        let declaration = self.admin_declaration(&request).await?;
        self.validate_key(&declaration, request.prefix.as_deref(), true)?;
        self.list_at(
            &declaration,
            request.prefix.as_deref(),
            request.offset.unwrap_or(0),
            request.limit.unwrap_or(0),
        )
        .await
    }

    async fn admin_delete(&self, value: Value) -> Result<Value, ServerError> {
        let request: AdminRequest = serde_json::from_value(value)?;
        let declaration = self.admin_declaration(&request).await?;
        self.validate_key(&declaration, request.key.as_deref(), false)?;
        let expected = request
            .expected_revision
            .map(|value| parse_revision(&value))
            .transpose()?;
        self.admin_delete_at(&declaration, request.key.as_deref(), expected)
            .await
    }

    async fn normal_declaration(
        &self,
        context: &RequestContext,
        store: &str,
    ) -> Result<Declaration, ServerError> {
        let caller = context.caller.as_ref().ok_or_else(auth_denied)?;
        let scope = match (caller.principal.kind, caller.participant.kind) {
            (AuthorizationPrincipalKind::User, ParticipantKind::App) => Scope::UserApp,
            (AuthorizationPrincipalKind::Device, ParticipantKind::Device) => Scope::DeviceApp,
            _ => return Err(auth_denied()),
        };
        let binding = self
            .repository
            .get_participant_binding(&caller.participant.id, &caller.participant.artifact_digest)
            .await
            .map_err(unexpected)?
            .ok_or_else(|| validation("/store", "participant State declaration is unavailable"))?;
        if binding.participant_id != caller.participant.id
            || binding.participant_kind != caller.participant.kind
            || binding.artifact_digest != caller.participant.artifact_digest
            || binding.needs_digest != caller.participant.needs_digest
            || binding.state != ParticipantBindingState::Resolved
        {
            return Err(auth_denied());
        }
        declaration_from_binding(
            binding,
            scope,
            caller.principal.id.clone(),
            caller.participant.id.clone(),
            store,
        )
    }

    async fn admin_declaration(&self, request: &AdminRequest) -> Result<Declaration, ServerError> {
        match request.scope.as_str() {
            "userApp" => {
                let user_id = request
                    .user_id
                    .as_deref()
                    .ok_or_else(|| validation("/userId", "userId is required"))?;
                let principal = self
                    .repository
                    .get_principal(user_id)
                    .await
                    .map_err(unexpected)?
                    .ok_or_else(|| validation("/userId", "user target was not found"))?;
                if principal.kind != PrincipalKind::User {
                    return Err(validation("/userId", "target is not a user"));
                }
                let authority = self
                    .repository
                    .get_identity_authority(user_id, &request.contract_id)
                    .await
                    .map_err(unexpected)?
                    .ok_or_else(|| {
                        validation("/contractId", "user contract authority was not found")
                    })?;
                let binding = self
                    .repository
                    .get_participant_binding(
                        &authority.participant_id,
                        &authority.participant_artifact_digest,
                    )
                    .await
                    .map_err(unexpected)?
                    .ok_or_else(|| {
                        validation("/contractId", "user contract binding was not found")
                    })?;
                validate_admin_digest(
                    declaration_from_binding(
                        binding,
                        Scope::UserApp,
                        user_id.to_owned(),
                        request.contract_id.clone(),
                        &request.store,
                    )?,
                    &request.contract_digest,
                )
            }
            "deviceApp" => {
                let device_id = request
                    .device_id
                    .as_deref()
                    .ok_or_else(|| validation("/deviceId", "deviceId is required"))?;
                let mut matches = Vec::new();
                for device in self.repository.list_devices().await.map_err(unexpected)? {
                    if device.principal_id != device_id {
                        continue;
                    }
                    let Some(authority) = self
                        .repository
                        .get_deployment_authority(&device.deployment_id, &request.contract_id)
                        .await
                        .map_err(unexpected)?
                    else {
                        continue;
                    };
                    let Some(binding) = self
                        .repository
                        .get_participant_binding(
                            &authority.participant_id,
                            &authority.participant_artifact_digest,
                        )
                        .await
                        .map_err(unexpected)?
                    else {
                        continue;
                    };
                    matches.push(declaration_from_binding(
                        binding,
                        Scope::DeviceApp,
                        device_id.to_owned(),
                        request.contract_id.clone(),
                        &request.store,
                    )?);
                }
                select_admin_declaration(matches, &request.contract_digest)
            }
            _ => Err(validation("/scope", "scope is invalid")),
        }
    }

    fn require_resource(
        &self,
        context: &RequestContext,
        declaration: &Declaration,
        action: PermissionAction,
    ) -> Result<(), ServerError> {
        let caller = context.caller.as_ref().ok_or_else(auth_denied)?;
        let target = PermissionTarget::participant_resource(
            caller.participant.id.clone(),
            ParticipantResourceKind::State,
            declaration.store.clone(),
        )
        .map_err(unexpected)?;
        let atom = PermissionAtom::new(target, action).map_err(unexpected)?;
        self.verifier
            .require_cached_permission(&caller.context_digest, &atom)
            .map_err(|_| auth_denied())
    }

    fn validate_key(
        &self,
        declaration: &Declaration,
        key: Option<&str>,
        list: bool,
    ) -> Result<(), ServerError> {
        match declaration.kind {
            StateKind::Value if list => Err(validation("/store", "value stores cannot be listed")),
            StateKind::Value if key.is_some() => {
                Err(validation("/key", "value stores do not use keys"))
            }
            StateKind::Map if !list && key.is_none() => {
                Err(validation("/key", "map key is required"))
            }
            StateKind::Map => key.map_or(Ok(()), |path| {
                validate_path(path, if list { "/prefix" } else { "/key" })
            }),
            StateKind::Value => Ok(()),
        }
    }

    async fn get_at(
        &self,
        declaration: &Declaration,
        logical_key: Option<&str>,
    ) -> Result<Value, ServerError> {
        let key = physical_key(declaration, logical_key)?;
        let Some(entry) = self.live_entry(&key).await? else {
            return Ok(json!({"found": false}));
        };
        project_entry(declaration, logical_key, &entry, true)
    }

    async fn delete_at(
        &self,
        declaration: &Declaration,
        logical_key: Option<&str>,
        expected: Option<u64>,
    ) -> Result<Value, ServerError> {
        let key = physical_key(declaration, logical_key)?;
        let Some(entry) = self.live_entry(&key).await? else {
            return Ok(json!({"deleted": false}));
        };
        if expected.is_some_and(|expected| expected != entry.revision) {
            return Ok(json!({"deleted": false}));
        }
        match self
            .store
            .delete_expect_revision(&key, Some(entry.revision))
            .await
        {
            Ok(()) => Ok(json!({"deleted": true})),
            Err(error) => match self.live_entry(&key).await? {
                Some(current) if current.revision == entry.revision => Err(unexpected(error)),
                Some(_) | None => Ok(json!({"deleted": false})),
            },
        }
    }

    async fn admin_delete_at(
        &self,
        declaration: &Declaration,
        logical_key: Option<&str>,
        expected: Option<u64>,
    ) -> Result<Value, ServerError> {
        let key = physical_key(declaration, logical_key)?;
        let Some(entry) = self.store.entry(&key).await.map_err(kv_error)? else {
            return Ok(json!({"deleted": false}));
        };
        if entry.operation != kv::Operation::Put
            || expected.is_some_and(|expected| expected != entry.revision)
        {
            return Ok(json!({"deleted": false}));
        }
        let expired = valid_entry_is_expired(&entry.value, declaration, OffsetDateTime::now_utc());
        match self
            .store
            .delete_expect_revision(&key, Some(entry.revision))
            .await
        {
            Ok(()) => Ok(json!({"deleted": !expired})),
            Err(error) => {
                let current = self
                    .store
                    .entry(&key)
                    .await
                    .map_err(kv_error)?
                    .map(|entry| (entry.operation, entry.revision));
                if delete_failure_was_race(current, entry.revision) {
                    Ok(json!({"deleted": false}))
                } else {
                    Err(kv_error(error))
                }
            }
        }
    }

    async fn list_at(
        &self,
        declaration: &Declaration,
        prefix: Option<&str>,
        offset: u64,
        limit: u64,
    ) -> Result<Value, ServerError> {
        if declaration.kind != StateKind::Map {
            return Err(validation("/store", "only map stores can be listed"));
        }
        let physical_prefix = map_prefix(declaration, prefix)?;
        let keys = self.matching_keys(&physical_prefix).await?;
        let mut entries = Vec::new();
        for key in keys {
            let Some(entry) = self.live_entry(&key).await? else {
                continue;
            };
            let logical_key = decode_map_key(&physical_prefix_base(declaration)?, &key)?;
            entries.push((
                logical_key.clone(),
                project_entry(declaration, Some(&logical_key), &entry, false)?,
            ));
        }
        entries.sort_by(|left, right| left.0.cmp(&right.0));
        let count = u64::try_from(entries.len()).map_err(unexpected)?;
        let start = usize::try_from(offset)
            .unwrap_or(usize::MAX)
            .min(entries.len());
        let take = usize::try_from(limit).unwrap_or(usize::MAX);
        let page = entries
            .into_iter()
            .skip(start)
            .take(take)
            .map(|(_, value)| value)
            .collect::<Vec<_>>();
        let next = offset.saturating_add(u64::try_from(page.len()).map_err(unexpected)?);
        let mut response =
            json!({"entries": page, "count": count, "offset": offset, "limit": limit});
        if limit > 0 && next < count {
            response["nextOffset"] = json!(next);
        }
        Ok(response)
    }

    async fn live_entry(&self, key: &str) -> Result<Option<PhysicalEntry>, ServerError> {
        let Some(entry) = self.store.entry(key).await.map_err(kv_error)? else {
            return Ok(None);
        };
        if entry.operation != kv::Operation::Put {
            return Ok(None);
        }
        let envelope: StoredEnvelope = serde_json::from_slice(&entry.value).map_err(unexpected)?;
        if !is_canonical_digest(&envelope.writer_contract_digest) {
            return Err(unexpected("stored State writer contract digest is invalid"));
        }
        let expires_at = envelope
            .expires_at
            .as_deref()
            .map(parse_timestamp)
            .transpose()?;
        parse_timestamp(&envelope.updated_at)?;
        if expires_at.is_some_and(|expires_at| expires_at <= OffsetDateTime::now_utc()) {
            let _ = self
                .store
                .delete_expect_revision(key, Some(entry.revision))
                .await;
            return Ok(None);
        }
        Ok(Some(PhysicalEntry {
            revision: entry.revision,
            envelope,
        }))
    }

    async fn matching_keys(&self, prefix: &str) -> Result<Vec<String>, ServerError> {
        let stream = self
            .jetstream
            .get_stream(STREAM)
            .await
            .map_err(unexpected)?;
        let subject = format!("{SUBJECT_PREFIX}{prefix}>");
        let mut consumer = stream
            .create_consumer(consumer::push::OrderedConfig {
                deliver_subject: self.jetstream.client().new_inbox(),
                filter_subject: subject,
                headers_only: true,
                deliver_policy: consumer::DeliverPolicy::LastPerSubject,
                ..Default::default()
            })
            .await
            .map_err(unexpected)?;
        let snapshot_count = consumer.info().await.map_err(unexpected)?.num_pending;
        if snapshot_count == 0 {
            return Ok(Vec::new());
        }
        let mut messages = consumer.messages().await.map_err(unexpected)?;
        let mut keys = BTreeSet::new();
        for _ in 0..snapshot_count {
            let message = messages
                .next()
                .await
                .ok_or_else(|| unexpected("State key enumeration ended early"))?
                .map_err(unexpected)?;
            if let Some(key) = message.subject.strip_prefix(SUBJECT_PREFIX) {
                keys.insert(key.to_owned());
            }
        }
        Ok(keys.into_iter().collect())
    }
}

impl PhysicalEntry {
    fn clone_entry(&self) -> Self {
        Self {
            revision: self.revision,
            envelope: StoredEnvelope {
                value: self.envelope.value.clone(),
                state_version: self.envelope.state_version.clone(),
                writer_contract_digest: self.envelope.writer_contract_digest.clone(),
                updated_at: self.envelope.updated_at.clone(),
                expires_at: self.envelope.expires_at.clone(),
            },
        }
    }
}

fn declaration_from_binding(
    binding: ParticipantBindingRecord,
    scope: Scope,
    owner_id: String,
    contract_id: String,
    store: &str,
) -> Result<Declaration, ServerError> {
    let resolved = binding.resolve().map_err(unexpected)?;
    let owned_api_id = resolved
        .implemented_apis()
        .iter()
        .find(|implemented| implemented.alias() == "self")
        .ok_or_else(|| {
            validation(
                "/store",
                "participant does not implement its owned API under alias 'self'",
            )
        })?
        .provided()
        .api();
    let api_values: BTreeMap<String, Value> =
        serde_json::from_str(&binding.api_artifacts_json).map_err(unexpected)?;
    let api = api_values
        .get(owned_api_id)
        .ok_or_else(|| validation("/store", "contract API artifact was not found"))?;
    let api = parse_api(api).map_err(unexpected)?;
    let definition = api
        .state_definition(store)
        .ok_or_else(|| validation("/store", "State store is not declared"))?;
    let schema = api
        .schema(definition.schema_name())
        .cloned()
        .ok_or_else(|| unexpected("current State schema is missing"))?;
    let mut accepted_versions = BTreeMap::new();
    for (version, schema_name) in definition.accepted_versions() {
        accepted_versions.insert(
            version.to_owned(),
            api.schema(schema_name)
                .cloned()
                .ok_or_else(|| unexpected("accepted State schema is missing"))?,
        );
    }
    Ok(Declaration {
        scope,
        owner_id,
        contract_id,
        contract_digest: binding.artifact_digest,
        store: store.to_owned(),
        kind: definition.kind(),
        schema,
        state_version: definition.state_version().to_owned(),
        accepted_versions,
    })
}

fn validate_admin_digest(
    declaration: Declaration,
    expected: &str,
) -> Result<Declaration, ServerError> {
    if declaration.contract_digest == expected {
        Ok(declaration)
    } else {
        Err(validation(
            "/contractDigest",
            "contractDigest does not identify the current contract artifact",
        ))
    }
}

fn same_declaration(left: &Declaration, right: &Declaration) -> bool {
    left.contract_digest == right.contract_digest
        && left.kind == right.kind
        && left.schema == right.schema
        && left.state_version == right.state_version
        && left.accepted_versions == right.accepted_versions
}

fn select_admin_declaration(
    matches: Vec<Declaration>,
    expected_digest: &str,
) -> Result<Declaration, ServerError> {
    let mut matches = matches.into_iter();
    let first = matches
        .next()
        .ok_or_else(|| validation("/contractId", "device contract authority was not found"))?;
    if matches.any(|candidate| !same_declaration(&first, &candidate)) {
        return Err(declared(
            "UnexpectedError",
            "device contract authority is incoherent",
        ));
    }
    validate_admin_digest(first, expected_digest)
}

fn delete_failure_was_race(current: Option<(kv::Operation, u64)>, revision: u64) -> bool {
    !matches!(current, Some((kv::Operation::Put, current)) if current == revision)
}

fn update_failure_was_race(current_revision: Option<u64>, expected_revision: u64) -> bool {
    current_revision != Some(expected_revision)
}

fn valid_entry_is_expired(value: &[u8], declaration: &Declaration, now: OffsetDateTime) -> bool {
    let Ok(envelope) = serde_json::from_slice::<StoredEnvelope>(value) else {
        return false;
    };
    if !is_canonical_digest(&envelope.writer_contract_digest)
        || parse_timestamp(&envelope.updated_at).is_err()
    {
        return false;
    }
    let schema = if envelope.state_version == declaration.state_version {
        &declaration.schema
    } else if let Some(schema) = declaration.accepted_versions.get(&envelope.state_version) {
        schema
    } else {
        return false;
    };
    if !jsonschema::validator_for(schema).is_ok_and(|validator| validator.is_valid(&envelope.value))
    {
        return false;
    }
    envelope
        .expires_at
        .as_deref()
        .and_then(|value| parse_timestamp(value).ok())
        .is_some_and(|expires_at| expires_at <= now)
}

fn is_canonical_digest(value: &str) -> bool {
    URL_SAFE_NO_PAD
        .decode(value)
        .is_ok_and(|decoded| decoded.len() == 32 && URL_SAFE_NO_PAD.encode(decoded) == value)
}

fn physical_key(
    declaration: &Declaration,
    logical_key: Option<&str>,
) -> Result<String, ServerError> {
    let namespace = namespace_digest(declaration)?;
    let store = URL_SAFE_NO_PAD.encode(declaration.store.as_bytes());
    match declaration.kind {
        StateKind::Value => Ok(format!("value.{namespace}.{store}")),
        StateKind::Map => Ok(format!(
            "map.{namespace}.{store}.{}",
            encode_path(logical_key.expect("validated map key"))
        )),
    }
}

fn physical_prefix_base(declaration: &Declaration) -> Result<String, ServerError> {
    Ok(format!(
        "map.{}.{}.",
        namespace_digest(declaration)?,
        URL_SAFE_NO_PAD.encode(declaration.store.as_bytes())
    ))
}

fn map_prefix(declaration: &Declaration, prefix: Option<&str>) -> Result<String, ServerError> {
    let base = physical_prefix_base(declaration)?;
    Ok(prefix.map_or(base.clone(), |prefix| {
        format!("{base}{}.", encode_path(prefix))
    }))
}

fn namespace_digest(declaration: &Declaration) -> Result<String, ServerError> {
    trellis_protocol::digest_json(&json!({
        "scope": declaration.scope.as_str(),
        "ownerId": declaration.owner_id,
        "contractId": declaration.contract_id,
    }))
    .map_err(unexpected)
}

fn encode_path(path: &str) -> String {
    path.split('/')
        .map(|segment| URL_SAFE_NO_PAD.encode(segment.as_bytes()))
        .collect::<Vec<_>>()
        .join(".")
}

fn decode_map_key(base: &str, physical: &str) -> Result<String, ServerError> {
    let encoded = physical
        .strip_prefix(base)
        .ok_or_else(|| unexpected("State map key is outside its namespace"))?;
    encoded
        .split('.')
        .map(|segment| {
            URL_SAFE_NO_PAD
                .decode(segment)
                .map_err(unexpected)
                .and_then(|bytes| String::from_utf8(bytes).map_err(unexpected))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(|segments| segments.join("/"))
}

fn validate_path(path: &str, pointer: &str) -> Result<(), ServerError> {
    if path.is_empty()
        || path.starts_with('/')
        || path.ends_with('/')
        || path.split('/').any(str::is_empty)
    {
        Err(validation(pointer, "key must be a canonical slash path"))
    } else {
        Ok(())
    }
}

fn parse_revision(value: &str) -> Result<u64, ServerError> {
    if value.is_empty()
        || value == "0"
        || value.starts_with('0')
        || !value.bytes().all(|byte| byte.is_ascii_digit())
    {
        return Err(validation(
            "/expectedRevision",
            "expectedRevision must be a canonical positive integer",
        ));
    }
    value.parse().map_err(|_| {
        validation(
            "/expectedRevision",
            "expectedRevision exceeds the supported range",
        )
    })
}

fn timestamp(value: OffsetDateTime) -> Result<String, ServerError> {
    let milliseconds = value.unix_timestamp_nanos().div_euclid(1_000_000);
    let seconds = milliseconds.div_euclid(1_000);
    let millis = milliseconds.rem_euclid(1_000);
    let value = OffsetDateTime::from_unix_timestamp(i64::try_from(seconds).map_err(unexpected)?)
        .map_err(unexpected)?;
    let base = value.format(&Rfc3339).map_err(unexpected)?;
    let base = base
        .strip_suffix('Z')
        .ok_or_else(|| unexpected("UTC timestamp formatting failed"))?;
    Ok(format!("{base}.{millis:03}Z"))
}

fn parse_timestamp(value: &str) -> Result<OffsetDateTime, ServerError> {
    if value.len() != 24 || value.as_bytes().get(19) != Some(&b'.') || !value.ends_with('Z') {
        return Err(unexpected("stored State timestamp is not canonical"));
    }
    OffsetDateTime::parse(value, &Rfc3339).map_err(unexpected)
}

fn encode(envelope: &StoredEnvelope) -> Result<Bytes, ServerError> {
    let value = serde_json::to_value(envelope)?;
    trellis_protocol::canonicalize_json(&value)
        .map(Bytes::from)
        .map_err(unexpected)
}

fn public_entry(key: Option<&str>, revision: u64, envelope: &StoredEnvelope) -> Value {
    let mut entry = json!({
        "value": envelope.value,
        "revision": revision.to_string(),
        "updatedAt": envelope.updated_at,
    });
    if let Some(key) = key {
        entry["key"] = json!(key);
    }
    if let Some(expires_at) = envelope.expires_at.as_deref() {
        entry["expiresAt"] = json!(expires_at);
    }
    entry
}

fn project_entry(
    declaration: &Declaration,
    key: Option<&str>,
    entry: &PhysicalEntry,
    found_wrapper: bool,
) -> Result<Value, ServerError> {
    let public = public_entry(key, entry.revision, &entry.envelope);
    if entry.envelope.state_version == declaration.state_version {
        validate_schema(&declaration.schema, &entry.envelope.value, "", true)?;
        return Ok(if found_wrapper {
            json!({"found": true, "entry": public})
        } else {
            public
        });
    }
    let schema = declaration
        .accepted_versions
        .get(&entry.envelope.state_version)
        .ok_or_else(|| unexpected("stored State version is not accepted"))?;
    validate_schema(schema, &entry.envelope.value, "", true)?;
    Ok(json!({
        "migrationRequired": true,
        "entry": public,
        "stateVersion": entry.envelope.state_version,
        "currentStateVersion": declaration.state_version,
        "writerContractDigest": entry.envelope.writer_contract_digest,
    }))
}

fn conflict(
    entry: Option<PhysicalEntry>,
    key: Option<&str>,
    declaration: &Declaration,
) -> Result<Value, ServerError> {
    match entry {
        Some(entry) => Ok(json!({
            "applied": false,
            "found": true,
            "entry": project_entry(declaration, key, &entry, false)?,
        })),
        None => Ok(json!({"applied": false, "found": false})),
    }
}

fn validate_schema(
    schema: &Value,
    value: &Value,
    path: &str,
    stored: bool,
) -> Result<(), ServerError> {
    let validator = jsonschema::validator_for(schema).map_err(unexpected)?;
    if validator.is_valid(value) {
        Ok(())
    } else if stored {
        Err(unexpected("stored State value fails its declared schema"))
    } else {
        Err(validation(path, "value fails the declared State schema"))
    }
}

fn output<T: for<'de> Deserialize<'de>>(value: Value) -> Result<T, ServerError> {
    serde_json::from_value(value).map_err(ServerError::from)
}

fn validation(path: &str, message: &str) -> ServerError {
    ServerError::Validation {
        issues: Box::new(vec![ValidationIssue {
            path: path.to_owned(),
            message: message.to_owned(),
        }]),
    }
}

fn auth_denied() -> ServerError {
    ServerError::DeclaredRpc(DeclaredRpcError::new(
        "AuthError",
        "request is not granted by the active authority",
        [("reason", json!("not_authorized"))],
    ))
}

fn declared(error_type: &str, message: &str) -> ServerError {
    ServerError::DeclaredRpc(DeclaredRpcError::new(
        error_type,
        message,
        std::iter::empty::<(&str, Value)>(),
    ))
}

fn unexpected(error: impl std::fmt::Display) -> ServerError {
    tracing::error!(error = %error, "State runtime failure");
    declared("UnexpectedError", "State is temporarily unavailable")
}

fn kv_error(error: impl std::fmt::Display) -> ServerError {
    let message = error.to_string();
    if message.to_ascii_lowercase().contains("key")
        && (message.to_ascii_lowercase().contains("invalid")
            || message.to_ascii_lowercase().contains("maximum"))
    {
        validation("/key", "encoded key exceeds NATS KV key limits")
    } else {
        unexpected(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_protocol::{parse_participant, resolve_participant};

    fn test_declaration(digest: &str) -> Declaration {
        Declaration {
            scope: Scope::DeviceApp,
            owner_id: "device-1".into(),
            contract_id: "example.device@v1".into(),
            contract_digest: digest.into(),
            store: "preferences".into(),
            kind: StateKind::Value,
            schema: json!({"type": "string"}),
            state_version: "v1".into(),
            accepted_versions: BTreeMap::new(),
        }
    }

    #[test]
    fn state_key_revision_and_timestamp_codecs_are_canonical() {
        assert!(validate_path("inspection/active/open", "/key").is_ok());
        for invalid in ["", "/open", "open/", "open//draft"] {
            assert!(
                validate_path(invalid, "/key").is_err(),
                "accepted {invalid:?}"
            );
        }
        assert_eq!(parse_revision("42").expect("canonical revision"), 42);
        for invalid in ["", "0", "01", "+1", "-1", "x"] {
            assert!(parse_revision(invalid).is_err(), "accepted {invalid:?}");
        }
        let value = OffsetDateTime::from_unix_timestamp_nanos(1_754_741_696_789_123_456)
            .expect("test timestamp");
        let encoded = timestamp(value).expect("format timestamp");
        assert_eq!(encoded, "2025-08-09T12:14:56.789Z");
        assert_eq!(
            parse_timestamp(&encoded)
                .expect("parse timestamp")
                .unix_timestamp_nanos(),
            1_754_741_696_789_000_000
        );
        let digest =
            trellis_protocol::digest_json(&json!({"contract": "device"})).expect("contract digest");
        assert!(is_canonical_digest(&digest));
        assert!(!is_canonical_digest("not-a-contract-digest"));
    }

    #[test]
    fn failed_delete_classifies_only_absent_or_changed_entries_as_races() {
        assert!(delete_failure_was_race(None, 7));
        assert!(delete_failure_was_race(Some((kv::Operation::Delete, 7)), 7));
        assert!(delete_failure_was_race(Some((kv::Operation::Put, 8)), 7));
        assert!(!delete_failure_was_race(Some((kv::Operation::Put, 7)), 7));
    }

    #[test]
    fn failed_update_classifies_only_absent_or_changed_entries_as_races() {
        assert!(update_failure_was_race(None, 7));
        assert!(update_failure_was_race(Some(8), 7));
        assert!(!update_failure_was_race(Some(7), 7));
    }

    #[test]
    fn device_admin_selection_establishes_coherence_before_digest_validation() {
        assert!(select_admin_declaration(vec![test_declaration("digest-a")], "digest-a").is_ok());

        let wrong = select_admin_declaration(vec![test_declaration("digest-a")], "digest-b")
            .err()
            .expect("wrong digest must fail");
        assert!(format!("{wrong:?}").contains("/contractDigest"));

        assert!(select_admin_declaration(
            vec![test_declaration("digest-a"), test_declaration("digest-a")],
            "digest-a",
        )
        .is_ok());

        let incoherent = select_admin_declaration(
            vec![test_declaration("digest-a"), test_declaration("digest-b")],
            "digest-a",
        )
        .err()
        .expect("conflicting candidates must fail");
        assert!(format!("{incoherent:?}").contains("UnexpectedError"));
    }

    #[test]
    fn state_declaration_admin_and_writer_digests_match_participant_binding() {
        let api = json!({
            "format": "trellis.api.v1",
            "id": "example.device@v1",
            "version": "1.0.0",
            "displayName": "Example Device",
            "description": "State test API",
            "schemas": {"State": {"type": "string"}},
            "state": {"preferences": {"kind": "value", "schema": {"schema": "State"}}}
        });
        let parsed_api = parse_api(&api).expect("API");
        let api_digest = parsed_api.digest().expect("API digest");
        let shared_api = json!({
            "format": "trellis.api.v1",
            "id": "example.shared@v1",
            "version": "1.0.0",
            "displayName": "Shared",
            "description": "State test dependency",
            "schemas": {"State": {"type": "boolean"}},
            "state": {"preferences": {"kind": "value", "schema": {"schema": "State"}}}
        });
        let parsed_shared_api = parse_api(&shared_api).expect("shared API");
        let shared_api_digest = parsed_shared_api.digest().expect("shared API digest");
        let participant = json!({
            "format": "trellis.participant.v1",
            "id": "example.device-participant@v1",
            "displayName": "Example Device",
            "description": "State test participant",
            "kind": "device",
            "implements": {
                "shared": {"api": "example.shared@v1", "apiDigest": shared_api_digest},
                "self": {"api": "example.device@v1", "apiDigest": api_digest}
            }
        });
        let parsed_participant = parse_participant(&participant).expect("participant");
        let participant_digest = parsed_participant.digest().expect("participant digest");
        let resolved = resolve_participant(
            &parsed_participant,
            &BTreeMap::from([
                ("example.device@v1".to_owned(), parsed_api),
                ("example.shared@v1".to_owned(), parsed_shared_api),
            ]),
        )
        .expect("resolved participant");
        let binding = ParticipantBindingRecord {
            participant_id: "example.device-participant@v1".to_owned(),
            participant_kind: ParticipantKind::Device,
            artifact_digest: participant_digest.clone(),
            needs_digest: resolved.needs().digest().expect("needs digest"),
            participant_json: serde_json::to_string(&participant).expect("participant JSON"),
            api_artifacts_json: serde_json::to_string(&json!({
                "example.device@v1": api,
                "example.shared@v1": shared_api
            }))
            .expect("API map JSON"),
            resolved_at: 0,
            state: ParticipantBindingState::Resolved,
            error: None,
        };
        let declaration = declaration_from_binding(
            binding,
            Scope::DeviceApp,
            "device-1".to_owned(),
            "example.device-participant@v1".to_owned(),
            "preferences",
        )
        .expect("declaration");
        assert_eq!(declaration.contract_digest, participant_digest);
        assert_eq!(declaration.schema, json!({"type": "string"}));
        assert!(validate_admin_digest(declaration.clone(), &participant_digest).is_ok());
        let writer = StoredEnvelope {
            value: json!("value"),
            state_version: declaration.state_version.clone(),
            writer_contract_digest: declaration.contract_digest.clone(),
            updated_at: "2025-01-01T00:00:00Z".to_owned(),
            expires_at: None,
        };
        assert_eq!(writer.writer_contract_digest, participant_digest);
    }

    #[test]
    fn compatible_participant_digest_changes_keep_state_namespace() {
        assert_eq!(
            namespace_digest(&test_declaration("digest-a")).expect("namespace"),
            namespace_digest(&test_declaration("digest-b")).expect("namespace"),
        );
    }
}
