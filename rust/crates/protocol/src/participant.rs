use std::collections::{BTreeMap, BTreeSet};

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use jsonptr::PointerBuf;
use serde::{de::Error as _, Deserialize, Deserializer, Serialize};
use serde_json::{Map, Value};

use crate::{
    canonicalize_json, digest_json,
    identifiers::{
        participant_error, sort_deduplicate, validate_api_id, validate_logical_name,
        validate_nonempty_text, validate_protocol_identifier,
    },
    schema_profile::{
        lint_participant_authoring, validate_embedded_schema,
        validate_participant_runtime_structure, validate_wire_schema_additive,
    },
    ProtocolError,
};

/// The first canonical Trellis participant artifact format.
pub const PARTICIPANT_FORMAT_V1: &str = "trellis.participant.v1";

/// Strict Draft 2020-12 schema for authoring `trellis.participant.v1` artifacts.
pub const PARTICIPANT_AUTHORING_SCHEMA_V1_JSON: &str =
    include_str!("../schemas/trellis.participant.v1.schema.json");

/// Apply the strict, closed authoring lint to a participant artifact.
///
/// Runtime parsing is intentionally tolerant of unknown object members; use
/// this lint in authoring tools when extensions should be reported.
///
/// # Errors
///
/// Returns [`ProtocolError::ParticipantValidation`] when the closed authoring
/// schema or the participant's intrinsic invariants are violated.
pub fn lint_participant_v1_authoring(value: &Value) -> Result<(), ProtocolError> {
    lint_participant_authoring(value)?;
    parse_participant_v1(value).map(|_| ())
}

/// A user-authored participant kind.
///
/// Services provide hosted APIs, apps provide interactive application behavior,
/// devices represent deployed hardware participants, and agents run delegated
/// automated behavior.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ParticipantKindV1 {
    /// A service participant.
    Service,
    /// An application participant.
    App,
    /// A device participant.
    Device,
    /// An agent participant.
    Agent,
}

/// A validated, normalized `trellis.participant.v1` artifact.
///
/// `implements` records APIs provided by this participant. Required and optional
/// `uses` record APIs and surfaces it consumes. The artifact also owns its local
/// schemas and private state, Jobs, consumer, KV, store, and transfer declarations.
/// Validation here is intrinsic; referenced API surfaces are not contextually
/// resolved by this type.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ParticipantArtifactV1 {
    format: String,
    id: String,
    display_name: String,
    description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
    kind: ParticipantKindV1,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    schemas: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    implements: BTreeMap<String, ImplementedApiV1>,
    #[serde(skip_serializing_if = "UsesV1::is_empty")]
    uses: UsesV1,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    state: BTreeMap<String, ParticipantStateV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    job_queues: BTreeMap<String, JobQueueV1>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    event_consumers: BTreeMap<String, EventConsumerV1>,
    #[serde(skip_serializing_if = "ResourcesV1::is_empty")]
    resources: ResourcesV1,
}

/// Validate and parse one raw `trellis.participant.v1` JSON value.
///
/// Unknown object members are ignored and do not affect normalization or the
/// semantic digest. Use [`lint_participant_v1_authoring`] in strict authoring
/// tools.
///
/// # Errors
///
/// Returns [`ProtocolError::ParticipantValidation`] with an RFC 6901 path for an
/// invalid artifact, local reference, resource, or selection, or
/// [`ProtocolError::Json`] when decoding fails.
pub fn parse_participant_v1(value: &Value) -> Result<ParticipantArtifactV1, ProtocolError> {
    validate_participant_runtime_structure(value)?;
    let wire: WireParticipantArtifactV1 = serde_json::from_value(value.clone())
        .map_err(|error| participant_error("", error.to_string()))?;
    ParticipantArtifactV1::from_wire(wire)
}

impl ParticipantArtifactV1 {
    /// Return the stable software-definition identifier.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Return the participant kind.
    pub fn kind(&self) -> ParticipantKindV1 {
        self.kind
    }

    /// Serialize the normalized participant artifact.
    ///
    /// This includes supported human-facing fields as well as identity-bearing
    /// machine fields.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if the validated artifact cannot be
    /// represented as JSON.
    pub fn normalized_value(&self) -> Result<Value, ProtocolError> {
        Ok(serde_json::to_value(self)?)
    }

    /// Render the full normalized artifact as RFC 8785 canonical JSON.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn canonical_json(&self) -> Result<String, ProtocolError> {
        canonicalize_json(&self.normalized_value()?)
    }

    /// Build the machine-visible participant identity projection.
    ///
    /// Display names, descriptions, docs, resource purposes, and resource docs
    /// are excluded. Kind, schemas, pinned API references and selections,
    /// participant-local state, private Jobs queues, event consumers, resources,
    /// and transfer mappings are included.
    ///
    /// # Errors
    ///
    /// Returns [`ProtocolError::Json`] if a supported machine field cannot be
    /// represented as JSON.
    pub fn digest_projection(&self) -> Result<Value, ProtocolError> {
        let mut projection = Map::new();
        projection.insert("format".to_string(), Value::String(self.format.clone()));
        projection.insert("id".to_string(), Value::String(self.id.clone()));
        projection.insert("kind".to_string(), serde_json::to_value(self.kind)?);
        insert_nonempty(&mut projection, "schemas", &self.schemas)?;
        insert_nonempty(&mut projection, "implements", &self.implements)?;
        if !self.uses.is_empty() {
            projection.insert("uses".to_string(), serde_json::to_value(&self.uses)?);
        }
        insert_map_without_human_fields(&mut projection, "state", &self.state, false)?;
        insert_map_without_human_fields(&mut projection, "jobQueues", &self.job_queues, false)?;
        insert_map_without_human_fields(
            &mut projection,
            "eventConsumers",
            &self.event_consumers,
            false,
        )?;
        if !self.resources.is_empty() {
            let mut resources = serde_json::to_value(&self.resources)?;
            if let Some(resources) = resources.as_object_mut() {
                for definitions in resources.values_mut().filter_map(Value::as_object_mut) {
                    for definition in definitions.values_mut().filter_map(Value::as_object_mut) {
                        definition.remove("purpose");
                        definition.remove("docs");
                    }
                }
            }
            projection.insert("resources".to_string(), resources);
        }
        Ok(Value::Object(projection))
    }

    /// Return the SHA-256 base64url digest of the machine-visible projection.
    ///
    /// # Errors
    ///
    /// Returns a serialization or canonicalization [`ProtocolError`].
    pub fn digest(&self) -> Result<String, ProtocolError> {
        digest_json(&self.digest_projection()?)
    }

    fn from_wire(mut wire: WireParticipantArtifactV1) -> Result<Self, ProtocolError> {
        if wire.format != PARTICIPANT_FORMAT_V1 {
            return Err(participant_error(
                "/format",
                format!("must equal '{PARTICIPANT_FORMAT_V1}'"),
            ));
        }
        validate_protocol_identifier("/id", &wire.id, participant_error)?;
        validate_nonempty_text("/displayName", &wire.display_name, participant_error)?;
        validate_nonempty_text("/description", &wire.description, participant_error)?;
        if let Some(docs) = &wire.docs {
            validate_docs("/docs", docs)?;
        }

        for (name, schema) in &wire.schemas {
            validate_protocol_identifier(&member_path("schemas", name), name, participant_error)?;
            validate_embedded_schema(name, schema)?;
        }

        let mut aliases = BTreeMap::new();
        let mut api_ids = BTreeMap::new();
        for (alias, implemented) in &wire.implements {
            validate_api_reference(
                alias,
                "implements",
                &implemented.api,
                &implemented.api_digest,
                &mut aliases,
                &mut api_ids,
            )?;
        }
        for (group, uses) in [
            ("uses/required", &wire.uses.required),
            ("uses/optional", &wire.uses.optional),
        ] {
            for (alias, used) in uses {
                validate_api_reference(
                    alias,
                    group,
                    &used.api,
                    &used.api_digest,
                    &mut aliases,
                    &mut api_ids,
                )?;
            }
        }

        for (alias, used) in &mut wire.uses.required {
            normalize_used_api("required", alias, used)?;
        }
        for (alias, used) in &mut wire.uses.optional {
            normalize_used_api("optional", alias, used)?;
        }

        for (name, state) in &wire.state {
            let path = member_path("state", name);
            validate_protocol_identifier(&path, name, participant_error)?;
            require_schema(&wire.schemas, &state.schema, &format!("{path}/schema"))?;
            if let Some(version) = &state.state_version {
                validate_protocol_identifier(
                    &format!("{path}/stateVersion"),
                    version,
                    participant_error,
                )?;
            }
            for (version, schema) in &state.accepted_versions {
                let version_path = pointer(["state", name, "acceptedVersions", version]);
                validate_protocol_identifier(&version_path, version, participant_error)?;
                require_schema(&wire.schemas, schema, &version_path)?;
            }
            if let Some(docs) = &state.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        for (name, queue) in &wire.job_queues {
            validate_job_queue(&wire.schemas, name, queue)?;
        }

        for (name, resource) in &wire.resources.kv {
            let path = member_path("resources/kv", name);
            validate_protocol_identifier(&path, name, participant_error)?;
            validate_nonempty_text(
                &format!("{path}/purpose"),
                &resource.purpose,
                participant_error,
            )?;
            require_schema(&wire.schemas, &resource.schema, &format!("{path}/schema"))?;
            if resource.history == 0 {
                return Err(participant_error(
                    format!("{path}/history"),
                    "must be at least 1",
                ));
            }
            validate_optional_positive(&format!("{path}/maxValueBytes"), resource.max_value_bytes)?;
            if let Some(docs) = &resource.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }
        for (name, resource) in &wire.resources.store {
            let path = member_path("resources/store", name);
            validate_protocol_identifier(&path, name, participant_error)?;
            validate_nonempty_text(
                &format!("{path}/purpose"),
                &resource.purpose,
                participant_error,
            )?;
            validate_optional_positive(
                &format!("{path}/maxObjectBytes"),
                resource.max_object_bytes,
            )?;
            validate_optional_positive(&format!("{path}/maxTotalBytes"), resource.max_total_bytes)?;
            if let Some(docs) = &resource.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        let mut wire_schemas = BTreeSet::new();
        for state in wire.state.values() {
            wire_schemas.insert(&state.schema.schema);
            wire_schemas.extend(
                state
                    .accepted_versions
                    .values()
                    .map(|reference| &reference.schema),
            );
        }
        for queue in wire.job_queues.values() {
            wire_schemas.insert(&queue.payload.schema);
            wire_schemas.extend(
                [queue.update.as_ref(), queue.result.as_ref()]
                    .into_iter()
                    .flatten()
                    .map(|reference| &reference.schema),
            );
        }
        wire_schemas.extend(
            wire.resources
                .kv
                .values()
                .map(|resource| &resource.schema.schema),
        );
        for name in wire_schemas {
            validate_wire_schema_additive(name, &wire.schemas[name])?;
        }

        for (alias, implemented) in &wire.implements {
            for (operation, transfer) in &implemented.operation_transfers {
                let path = pointer(["implements", alias, "operationTransfers", operation]);
                validate_logical_name(&path, operation, participant_error)?;
                validate_protocol_identifier(
                    &format!("{path}/store"),
                    &transfer.store,
                    participant_error,
                )?;
                if !wire.resources.store.contains_key(&transfer.store) {
                    return Err(participant_error(
                        format!("{path}/store"),
                        format!("unknown local store resource '{}'", transfer.store),
                    ));
                }
                validate_pointer(&format!("{path}/key"), &transfer.key)?;
                for (field, pointer) in [
                    ("contentType", transfer.content_type.as_deref()),
                    ("metadata", transfer.metadata.as_deref()),
                ] {
                    if let Some(pointer) = pointer {
                        validate_pointer(&format!("{path}/{field}"), pointer)?;
                    }
                }
                validate_optional_positive(&format!("{path}/expiresInMs"), transfer.expires_in_ms)?;
                validate_optional_positive(&format!("{path}/maxBytes"), transfer.max_bytes)?;
            }
        }

        for (name, consumer) in &mut wire.event_consumers {
            let path = member_path("eventConsumers", name);
            validate_protocol_identifier(&path, name, participant_error)?;
            if consumer.events.is_empty() {
                return Err(participant_error(
                    format!("{path}/events"),
                    "must contain at least one API alias",
                ));
            }
            for (alias, events) in &mut consumer.events {
                let alias_path = pointer(["eventConsumers", name, "events", alias]);
                validate_protocol_identifier(&alias_path, alias, participant_error)?;
                if events.is_empty() {
                    return Err(participant_error(
                        &alias_path,
                        "must contain at least one event",
                    ));
                }
                for event in events.iter() {
                    validate_logical_name(&alias_path, event, participant_error)?;
                }
                sort_deduplicate(events);
                if wire.implements.contains_key(alias) {
                    continue;
                }
                let used = wire
                    .uses
                    .required
                    .get(alias)
                    .or_else(|| wire.uses.optional.get(alias))
                    .ok_or_else(|| {
                        participant_error(&alias_path, format!("unknown API alias '{alias}'"))
                    })?;
                for event in events.iter() {
                    if !used.events.subscribe.contains(event) {
                        return Err(participant_error(
                            &alias_path,
                            format!(
                                "event '{event}' is not selected under uses.{alias}.events.subscribe"
                            ),
                        ));
                    }
                }
            }
            validate_optional_positive(&format!("{path}/ackWaitMs"), consumer.ack_wait_ms)?;
            validate_optional_positive(&format!("{path}/maxDeliver"), consumer.max_deliver)?;
            validate_backoff(&format!("{path}/backoffMs"), consumer.backoff_ms.as_ref())?;
            if let Some(docs) = &consumer.docs {
                validate_docs(&format!("{path}/docs"), docs)?;
            }
        }

        Ok(Self {
            format: wire.format,
            id: wire.id,
            display_name: wire.display_name,
            description: wire.description,
            docs: wire.docs,
            kind: wire.kind,
            schemas: wire.schemas,
            implements: wire.implements,
            uses: wire.uses,
            state: wire.state,
            job_queues: wire.job_queues,
            event_consumers: wire.event_consumers,
            resources: wire.resources,
        })
    }
}

impl<'de> Deserialize<'de> for ParticipantArtifactV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        parse_participant_v1(&value).map_err(D::Error::custom)
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct WireParticipantArtifactV1 {
    format: String,
    id: String,
    display_name: String,
    description: String,
    #[serde(default)]
    docs: Option<DocumentationV1>,
    kind: ParticipantKindV1,
    #[serde(default)]
    schemas: BTreeMap<String, Value>,
    #[serde(default)]
    implements: BTreeMap<String, ImplementedApiV1>,
    #[serde(default)]
    uses: UsesV1,
    #[serde(default)]
    state: BTreeMap<String, ParticipantStateV1>,
    #[serde(default)]
    job_queues: BTreeMap<String, JobQueueV1>,
    #[serde(default)]
    event_consumers: BTreeMap<String, EventConsumerV1>,
    #[serde(default)]
    resources: ResourcesV1,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct DocumentationV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
    markdown: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
struct SchemaReferenceV1 {
    schema: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImplementedApiV1 {
    api: String,
    api_digest: String,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    operation_transfers: BTreeMap<String, OperationTransferV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct OperationTransferV1 {
    store: String,
    key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    content_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    metadata: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    expires_in_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_bytes: Option<u64>,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct UsesV1 {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    required: BTreeMap<String, UsedApiV1>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    optional: BTreeMap<String, UsedApiV1>,
}

impl UsesV1 {
    fn is_empty(&self) -> bool {
        self.required.is_empty() && self.optional.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct UsedApiV1 {
    api: String,
    api_digest: String,
    #[serde(default, skip_serializing_if = "RpcUsesV1::is_empty")]
    rpc: RpcUsesV1,
    #[serde(default, skip_serializing_if = "OperationUsesV1::is_empty")]
    operations: OperationUsesV1,
    #[serde(default, skip_serializing_if = "EventUsesV1::is_empty")]
    events: EventUsesV1,
    #[serde(default, skip_serializing_if = "FeedUsesV1::is_empty")]
    feeds: FeedUsesV1,
    #[serde(default, skip_serializing_if = "StateUsesV1::is_empty")]
    state: StateUsesV1,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct RpcUsesV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    call: Vec<String>,
}

impl RpcUsesV1 {
    fn is_empty(&self) -> bool {
        self.call.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct OperationUsesV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    invoke: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    observe: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    cancel: Vec<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    control: BTreeMap<String, Vec<String>>,
}

impl OperationUsesV1 {
    fn is_empty(&self) -> bool {
        self.invoke.is_empty()
            && self.observe.is_empty()
            && self.cancel.is_empty()
            && self.control.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct EventUsesV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    publish: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    subscribe: Vec<String>,
}

impl EventUsesV1 {
    fn is_empty(&self) -> bool {
        self.publish.is_empty() && self.subscribe.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct FeedUsesV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    subscribe: Vec<String>,
}

impl FeedUsesV1 {
    fn is_empty(&self) -> bool {
        self.subscribe.is_empty()
    }
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct StateUsesV1 {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    read: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    write: Vec<String>,
}

impl StateUsesV1 {
    fn is_empty(&self) -> bool {
        self.read.is_empty() && self.write.is_empty()
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum StateKindV1 {
    Value,
    Map,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct ParticipantStateV1 {
    kind: StateKindV1,
    schema: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    state_version: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    accepted_versions: BTreeMap<String, SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct JobQueueV1 {
    payload: SchemaReferenceV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    update: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    result: Option<SchemaReferenceV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_deliver: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backoff_ms: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    ack_wait_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    default_deadline_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "is_false")]
    progress: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    logs: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    dlq: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    key_concurrency: Option<KeyConcurrencyV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    queue: Option<QueuePolicyV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct KeyConcurrencyV1 {
    key: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_active: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heartbeat_interval_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    heartbeat_ttl_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    stale_policy: Option<StalePolicyV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum StalePolicyV1 {
    FailStale,
    Block,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct QueuePolicyV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    max_queued_per_key: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    when_full: Option<WhenFullV1>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
enum WhenFullV1 {
    Reject,
    Coalesce,
    ReplaceOldest,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct EventConsumerV1 {
    events: BTreeMap<String, Vec<String>>,
    #[serde(default, skip_serializing_if = "is_replay_new")]
    replay: ReplayV1,
    #[serde(default, skip_serializing_if = "is_ordering_strict")]
    ordering: OrderingV1,
    #[serde(skip_serializing_if = "Option::is_none")]
    ack_wait_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_deliver: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    backoff_ms: Option<Vec<u64>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum ReplayV1 {
    #[default]
    New,
    All,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
enum OrderingV1 {
    #[default]
    Strict,
    Parallel,
}

#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
struct ResourcesV1 {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    kv: BTreeMap<String, KvResourceV1>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    store: BTreeMap<String, StoreResourceV1>,
}

impl ResourcesV1 {
    fn is_empty(&self) -> bool {
        self.kv.is_empty() && self.store.is_empty()
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct KvResourceV1 {
    purpose: String,
    schema: SchemaReferenceV1,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    required: bool,
    #[serde(default = "default_one", skip_serializing_if = "is_one")]
    history: u64,
    #[serde(default, skip_serializing_if = "is_zero")]
    ttl_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_value_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct StoreResourceV1 {
    purpose: String,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    required: bool,
    #[serde(default, skip_serializing_if = "is_zero")]
    ttl_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_object_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_total_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    docs: Option<DocumentationV1>,
}

fn validate_api_reference<'a>(
    alias: &'a str,
    group: &'static str,
    api: &'a str,
    digest: &str,
    aliases: &mut BTreeMap<&'a str, &'static str>,
    api_ids: &mut BTreeMap<&'a str, (&'a str, &'static str)>,
) -> Result<(), ProtocolError> {
    let path = member_path(group, alias);
    validate_protocol_identifier(&path, alias, participant_error)?;
    validate_api_id(&format!("{path}/api"), api, participant_error)?;
    validate_api_digest(&format!("{path}/apiDigest"), digest)?;
    if let Some(previous) = aliases.insert(alias, group) {
        return Err(participant_error(
            path,
            format!("alias '{alias}' also appears under '{previous}'"),
        ));
    }
    if let Some((previous_alias, previous_group)) = api_ids.insert(api, (alias, group)) {
        return Err(participant_error(
            format!("{path}/api"),
            format!(
                "API '{api}' is already referenced by alias '{previous_alias}' under '{previous_group}'"
            ),
        ));
    }
    Ok(())
}

fn validate_api_digest(path: &str, value: &str) -> Result<(), ProtocolError> {
    let decoded = URL_SAFE_NO_PAD
        .decode(value)
        .map_err(|error| participant_error(path, format!("invalid base64url digest: {error}")))?;
    if decoded.len() != 32 {
        return Err(participant_error(
            path,
            format!("must decode to 32 bytes, received {}", decoded.len()),
        ));
    }
    Ok(())
}

fn normalize_used_api(
    requirement: &str,
    alias: &str,
    used: &mut UsedApiV1,
) -> Result<(), ProtocolError> {
    let path = pointer(["uses", requirement, alias]);
    for (group, values) in [
        ("rpc/call", &mut used.rpc.call),
        ("operations/invoke", &mut used.operations.invoke),
        ("operations/observe", &mut used.operations.observe),
        ("operations/cancel", &mut used.operations.cancel),
        ("events/publish", &mut used.events.publish),
        ("events/subscribe", &mut used.events.subscribe),
        ("feeds/subscribe", &mut used.feeds.subscribe),
        ("state/read", &mut used.state.read),
        ("state/write", &mut used.state.write),
    ] {
        for name in values.iter() {
            validate_logical_name(&format!("{path}/{group}"), name, participant_error)?;
        }
        sort_deduplicate(values);
    }
    for (operation, signals) in &mut used.operations.control {
        let operation_path = pointer([
            "uses",
            requirement,
            alias,
            "operations",
            "control",
            operation,
        ]);
        validate_logical_name(&operation_path, operation, participant_error)?;
        for signal in signals.iter() {
            validate_protocol_identifier(
                &pointer([
                    "uses",
                    requirement,
                    alias,
                    "operations",
                    "control",
                    operation,
                    signal,
                ]),
                signal,
                participant_error,
            )?;
        }
        sort_deduplicate(signals);
        if signals.is_empty() {
            return Err(participant_error(
                &operation_path,
                "must select at least one operation signal",
            ));
        }
    }
    if used.rpc.is_empty()
        && used.operations.is_empty()
        && used.events.is_empty()
        && used.feeds.is_empty()
        && used.state.is_empty()
    {
        return Err(participant_error(
            &path,
            "must select at least one API action",
        ));
    }
    Ok(())
}

fn validate_job_queue(
    schemas: &BTreeMap<String, Value>,
    name: &str,
    queue: &JobQueueV1,
) -> Result<(), ProtocolError> {
    let path = member_path("jobQueues", name);
    validate_protocol_identifier(&path, name, participant_error)?;
    require_schema(schemas, &queue.payload, &format!("{path}/payload"))?;
    for (field, schema) in [
        ("update", queue.update.as_ref()),
        ("result", queue.result.as_ref()),
    ] {
        if let Some(schema) = schema {
            require_schema(schemas, schema, &format!("{path}/{field}"))?;
        }
    }
    for (field, value) in [
        ("maxDeliver", queue.max_deliver),
        ("ackWaitMs", queue.ack_wait_ms),
        ("defaultDeadlineMs", queue.default_deadline_ms),
    ] {
        validate_optional_positive(&format!("{path}/{field}"), value)?;
    }
    validate_backoff(&format!("{path}/backoffMs"), queue.backoff_ms.as_ref())?;
    if let Some(keyed) = &queue.key_concurrency {
        if keyed.key.is_empty() {
            return Err(participant_error(
                format!("{path}/keyConcurrency/key"),
                "must contain at least one segment",
            ));
        }
        let mut pointers = BTreeSet::new();
        for (index, pointer) in keyed.key.iter().enumerate() {
            let pointer_path = format!("{path}/keyConcurrency/key/{index}");
            if pointer.is_empty() {
                return Err(participant_error(pointer_path, "must not be empty"));
            }
            if pointer.starts_with('/') {
                validate_pointer(&pointer_path, pointer)?;
            }
            if !pointers.insert(pointer) {
                return Err(participant_error(
                    pointer_path,
                    "key segments must be unique",
                ));
            }
        }
        for (field, value) in [
            ("maxActive", keyed.max_active),
            ("heartbeatIntervalMs", keyed.heartbeat_interval_ms),
            ("heartbeatTtlMs", keyed.heartbeat_ttl_ms),
        ] {
            validate_optional_positive(&format!("{path}/keyConcurrency/{field}"), value)?;
        }
    }
    if let Some(policy) = &queue.queue {
        if policy.max_queued_per_key.is_none() && policy.when_full.is_none() {
            return Err(participant_error(
                format!("{path}/queue"),
                "must contain at least one queue policy field",
            ));
        }
        if policy.when_full.is_some() && policy.max_queued_per_key.is_none() {
            return Err(participant_error(
                format!("{path}/queue/whenFull"),
                "requires maxQueuedPerKey",
            ));
        }
    }
    if let Some(docs) = &queue.docs {
        validate_docs(&format!("{path}/docs"), docs)?;
    }
    Ok(())
}

fn validate_docs(path: &str, docs: &DocumentationV1) -> Result<(), ProtocolError> {
    validate_nonempty_text(
        &format!("{path}/markdown"),
        &docs.markdown,
        participant_error,
    )?;
    if let Some(summary) = &docs.summary {
        validate_nonempty_text(&format!("{path}/summary"), summary, participant_error)?;
    }
    Ok(())
}

fn require_schema(
    schemas: &BTreeMap<String, Value>,
    reference: &SchemaReferenceV1,
    path: &str,
) -> Result<(), ProtocolError> {
    validate_protocol_identifier(path, &reference.schema, participant_error)?;
    if !schemas.contains_key(&reference.schema) {
        return Err(participant_error(
            path,
            format!("unknown local schema '{}'", reference.schema),
        ));
    }
    Ok(())
}

fn validate_pointer(path: &str, pointer: &str) -> Result<(), ProtocolError> {
    let parsed = jsonptr::Pointer::parse(pointer)
        .map_err(|error| participant_error(path, error.to_string()))?;
    if parsed.is_root() {
        return Err(participant_error(path, "must not use the root pointer"));
    }
    Ok(())
}

fn validate_optional_positive(path: &str, value: Option<u64>) -> Result<(), ProtocolError> {
    if value == Some(0) {
        return Err(participant_error(path, "must be greater than zero"));
    }
    Ok(())
}

fn validate_backoff(path: &str, value: Option<&Vec<u64>>) -> Result<(), ProtocolError> {
    if value.is_some_and(Vec::is_empty) {
        return Err(participant_error(path, "must contain at least one delay"));
    }
    Ok(())
}

fn member_path(section: &str, name: &str) -> String {
    pointer(section.split('/').chain(std::iter::once(name)))
}

fn pointer<'a>(tokens: impl IntoIterator<Item = &'a str>) -> String {
    PointerBuf::from_tokens(tokens).to_string()
}

fn insert_nonempty<T: Serialize>(
    projection: &mut Map<String, Value>,
    key: &str,
    value: &BTreeMap<String, T>,
) -> Result<(), ProtocolError> {
    if !value.is_empty() {
        projection.insert(key.to_string(), serde_json::to_value(value)?);
    }
    Ok(())
}

fn insert_map_without_human_fields<T: Serialize>(
    projection: &mut Map<String, Value>,
    key: &str,
    value: &BTreeMap<String, T>,
    remove_purpose: bool,
) -> Result<(), ProtocolError> {
    if value.is_empty() {
        return Ok(());
    }
    let mut value = serde_json::to_value(value)?;
    if let Some(definitions) = value.as_object_mut() {
        for definition in definitions.values_mut().filter_map(Value::as_object_mut) {
            definition.remove("docs");
            if remove_purpose {
                definition.remove("purpose");
            }
        }
    }
    projection.insert(key.to_string(), value);
    Ok(())
}

fn default_true() -> bool {
    true
}

fn default_one() -> u64 {
    1
}

fn is_false(value: &bool) -> bool {
    !*value
}

fn is_true(value: &bool) -> bool {
    *value
}

fn is_zero(value: &u64) -> bool {
    *value == 0
}

fn is_one(value: &u64) -> bool {
    *value == 1
}

fn is_replay_new(value: &ReplayV1) -> bool {
    *value == ReplayV1::New
}

fn is_ordering_strict(value: &OrderingV1) -> bool {
    *value == OrderingV1::Strict
}

#[cfg(test)]
mod tests {
    use std::{collections::BTreeMap, fs, path::PathBuf};

    use serde::Deserialize;
    use serde_json::json;

    use super::*;
    use crate::schema_profile::{lint_participant_authoring, validate_participant_meta_schema};

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields, rename_all = "camelCase")]
    struct Vector {
        name: String,
        schema_valid: bool,
        valid: bool,
        input: Value,
        #[serde(default)]
        normalized_from_input: bool,
        normalized: Option<Value>,
        digest_projection: Option<Value>,
        digest: Option<String>,
        same_normalized_as: Option<String>,
        same_digest_as: Option<String>,
        different_digest_from: Option<String>,
        error_schema: Option<String>,
        error_path: Option<String>,
    }

    #[derive(Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        vectors: Vec<Vector>,
    }

    #[test]
    fn participant_authoring_schema_and_shared_vectors_agree() {
        validate_participant_meta_schema().unwrap();
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../conformance/participant/vectors.json");
        let fixture: Fixture = serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap();
        let mut normalized_values = BTreeMap::new();
        let mut digests = BTreeMap::new();

        for vector in &fixture.vectors {
            assert_eq!(
                lint_participant_authoring(&vector.input).is_ok(),
                vector.schema_valid,
                "authoring lint result for {}",
                vector.name
            );
            let parsed = parse_participant_v1(&vector.input);
            if let (Err(error), Some(schema), Some(path)) =
                (&parsed, &vector.error_schema, &vector.error_path)
            {
                let ProtocolError::SchemaProfile {
                    schema: actual_schema,
                    path: actual_path,
                    ..
                } = error
                else {
                    panic!(
                        "expected schema profile error for {}: {error:?}",
                        vector.name
                    )
                };
                assert_eq!(actual_schema, schema, "schema for {}", vector.name);
                assert_eq!(actual_path, path, "path for {}", vector.name);
            }
            assert_eq!(
                parsed.is_ok(),
                vector.valid,
                "typed result for {}: {parsed:?}",
                vector.name
            );
            assert_eq!(
                serde_json::from_value::<ParticipantArtifactV1>(vector.input.clone()).is_ok(),
                vector.valid,
                "direct deserialization result for {}",
                vector.name
            );
            let Ok(participant) = parsed else {
                continue;
            };
            assert_eq!(participant.id(), vector.input["id"].as_str().unwrap());
            assert_eq!(
                serde_json::to_value(participant.kind()).unwrap(),
                vector.input["kind"]
            );
            let normalized = participant.normalized_value().unwrap();
            assert_eq!(
                participant.canonical_json().unwrap(),
                canonicalize_json(&normalized).unwrap(),
                "{}",
                vector.name
            );
            if vector.normalized_from_input {
                assert_eq!(normalized, vector.input, "{}", vector.name);
            }
            if let Some(expected) = &vector.normalized {
                assert_eq!(&normalized, expected, "{}", vector.name);
            }
            let projection = participant.digest_projection().unwrap();
            if vector.name == "private-schema-base" {
                assert_eq!(projection["schemas"], vector.input["schemas"]);
            }
            if let Some(expected) = &vector.digest_projection {
                assert_eq!(&projection, expected, "{}", vector.name);
            }
            let digest = participant.digest().unwrap();
            if let Some(expected) = &vector.digest {
                assert_eq!(&digest, expected, "{}", vector.name);
            }
            normalized_values.insert(vector.name.clone(), normalized);
            digests.insert(vector.name.clone(), digest);
        }

        for vector in &fixture.vectors {
            let Some(digest) = digests.get(&vector.name) else {
                continue;
            };
            if let Some(other) = &vector.same_normalized_as {
                assert_eq!(
                    &normalized_values[&vector.name], &normalized_values[other],
                    "{} and {other}",
                    vector.name
                );
            }
            if let Some(other) = &vector.same_digest_as {
                assert_eq!(digest, &digests[other], "{} and {other}", vector.name);
            }
            if let Some(other) = &vector.different_digest_from {
                assert_ne!(digest, &digests[other], "{} and {other}", vector.name);
            }
        }
    }

    #[test]
    fn participant_semantic_errors_use_participant_paths() {
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": " invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Invalid identifier.",
                "kind": "service"
            }),
            "/id",
        );
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Invalid API reference.",
                "kind": "app",
                "uses": {
                    "required": {
                        "billing/legacy": {
                            "api": " billing@v1",
                            "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "rpc": { "call": ["Billing.Get"] }
                        }
                    }
                }
            }),
            "/uses/required/billing~1legacy/api",
        );
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Invalid surface.",
                "kind": "app",
                "uses": {
                    "required": {
                        "billing": {
                            "api": "billing@v1",
                            "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "rpc": { "call": ["Billing..Get"] }
                        }
                    }
                }
            }),
            "/uses/required/billing/rpc/call",
        );
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Invalid queue.",
                "kind": "service",
                "schemas": { "Payload": true },
                "jobQueues": {
                    " queue": { "payload": { "schema": "Payload" } }
                }
            }),
            "/jobQueues/ queue",
        );
    }

    #[test]
    fn every_participant_wire_schema_reference_requires_additive_objects() {
        let base = json!({
            "format": PARTICIPANT_FORMAT_V1,
            "id": "example-worker",
            "displayName": "Example Worker",
            "description": "Example participant.",
            "kind": "service",
            "schemas": {
                "State": true, "StateV1": true, "Payload": true,
                "Update": true, "Result": true, "KvValue": true,
                "UnusedPrivate": { "type": "object", "additionalProperties": false }
            },
            "state": {
                "settings": {
                    "kind": "value", "schema": { "schema": "State" },
                    "acceptedVersions": { "v1": { "schema": "StateV1" } }
                }
            },
            "jobQueues": {
                "work": {
                    "payload": { "schema": "Payload" }, "update": { "schema": "Update" },
                    "result": { "schema": "Result" }
                }
            },
            "resources": {
                "kv": {
                    "cache": { "purpose": "Cache values.", "schema": { "schema": "KvValue" } }
                }
            }
        });
        assert!(parse_participant_v1(&base).is_ok());

        for (schema, expected_path) in [
            (
                json!({ "type": "object", "additionalProperties": false }),
                "/additionalProperties",
            ),
            (
                json!({ "type": "object", "not": { "required": ["futureField"] } }),
                "/not",
            ),
            (
                json!({ "type": "object", "if": { "required": ["futureField"] }, "then": false }),
                "/if",
            ),
            (
                json!({ "type": "object", "dependentSchemas": { "futureField": false } }),
                "/dependentSchemas",
            ),
            (
                json!({ "type": "object", "dependentRequired": { "futureField": ["other"] } }),
                "/dependentRequired",
            ),
            (
                json!({
                    "type": "object",
                    "properties": {
                        "nested": { "type": "object", "additionalProperties": false }
                    }
                }),
                "/properties/nested/additionalProperties",
            ),
        ] {
            for name in ["State", "StateV1", "Payload", "Update", "Result", "KvValue"] {
                let mut value = base.clone();
                value["schemas"][name] = schema.clone();
                let error = parse_participant_v1(&value).expect_err("closed wire schema must fail");
                assert_schema_profile(error, name, expected_path);
            }
        }
    }

    #[test]
    fn participant_authored_keys_are_json_pointer_encoded() {
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Missing state schema.",
                "kind": "service",
                "state": {
                    "cache/key": { "kind": "value", "schema": { "schema": "Missing" } }
                }
            }),
            "/state/cache~1key/schema",
        );
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Missing KV schema.",
                "kind": "service",
                "resources": {
                    "kv": {
                        "legacy~cache": {
                            "purpose": "Legacy cache.",
                            "schema": { "schema": "Missing" }
                        }
                    }
                }
            }),
            "/resources/kv/legacy~0cache/schema",
        );
        assert_participant_error(
            json!({
                "format": PARTICIPANT_FORMAT_V1,
                "id": "invalid-participant",
                "displayName": "Invalid Participant",
                "description": "Invalid operation selection.",
                "kind": "app",
                "uses": {
                    "required": {
                        "billing": {
                            "api": "billing@v1",
                            "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                            "operations": { "control": { "Op/~": [" sig/~"] } }
                        }
                    }
                }
            }),
            "/uses/required/billing/operations/control/Op~1~0/ sig~1~0",
        );
    }

    #[test]
    fn operation_signal_selections_sort_and_deduplicate_in_utf16_order() {
        let value = json!({
            "format": PARTICIPANT_FORMAT_V1,
            "id": "example-app",
            "displayName": "Example",
            "description": "Example app.",
            "kind": "app",
            "uses": {
                "required": {
                    "example": {
                        "api": "example@v1",
                        "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "operations": {
                            "control": { "Example.Run": ["\u{e000}", "😀", "😀"] }
                        }
                    }
                }
            }
        });
        assert_eq!(
            parse_participant_v1(&value)
                .unwrap()
                .normalized_value()
                .unwrap()["uses"]["required"]["example"]["operations"]["control"]["Example.Run"],
            json!(["😀", "\u{e000}"])
        );
    }

    #[test]
    fn runtime_extensions_do_not_change_participant_semantics() {
        let base = json!({
            "format": PARTICIPANT_FORMAT_V1,
            "id": "example-app",
            "displayName": "Example",
            "description": "Example app.",
            "kind": "app",
            "uses": {
                "required": {
                    "example": {
                        "api": "example@v1",
                        "apiDigest": "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
                        "rpc": { "call": ["Example.Get"] }
                    }
                }
            }
        });
        let mut extended = base.clone();
        extended["extension"] = json!(true);
        extended["uses"]["required"]["example"]["extension"] = json!(true);

        assert!(lint_participant_v1_authoring(&extended).is_err());
        let base = parse_participant_v1(&base).unwrap();
        let extended = parse_participant_v1(&extended).unwrap();
        assert_eq!(
            extended.normalized_value().unwrap(),
            base.normalized_value().unwrap()
        );
        assert_eq!(extended.digest().unwrap(), base.digest().unwrap());
    }

    fn assert_participant_error(value: Value, expected_path: &str) {
        match parse_participant_v1(&value).unwrap_err() {
            ProtocolError::ParticipantValidation { path, .. } => {
                assert_eq!(path, expected_path);
            }
            error => panic!("expected participant validation error, received {error:?}"),
        }
    }

    fn assert_schema_profile(error: ProtocolError, expected_schema: &str, expected_path: &str) {
        let ProtocolError::SchemaProfile { schema, path, .. } = error else {
            panic!("expected schema profile error, got {error:?}")
        };
        assert_eq!(schema, expected_schema);
        assert_eq!(path, expected_path);
    }
}
