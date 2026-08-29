use std::collections::BTreeMap;

use serde_json::{json, Map, Value};
use trellis_protocol::{
    lint_api_authoring, lint_participant_authoring, parse_api, parse_participant,
    resolve_participant, ApiArtifact, GrantSet, ParticipantArtifact, ResolvedParticipant,
};

use crate::{
    api_authoring::ContractAuthoringBuilder, authoring_model::AuthoringState,
    ContractCapabilityMetadata, ContractEvent, ContractEventConsumerGroup, ContractFeed,
    ContractJobQueueResource, ContractKind, ContractKvResource, ContractOperation,
    ContractRpcMethod, ContractStateStore, ContractStoreResource, ContractUseRef, ContractsError,
};

/// A validated native API artifact builder.
#[derive(Debug)]
pub struct ApiBuilder {
    value: Value,
    authoring: Option<ContractAuthoringBuilder>,
}

impl ApiBuilder {
    /// Start from a native `trellis.api.v1` value.
    pub fn new(value: Value) -> Self {
        Self {
            value,
            authoring: None,
        }
    }

    /// Start a native API builder whose `id` is only the API identity.
    pub fn authoring(
        id: impl Into<String>,
        version: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self {
            value: Value::Null,
            authoring: Some(ContractAuthoringBuilder::new_api(
                id,
                version,
                display_name,
                description,
            )),
        }
    }

    fn with_authoring(
        mut self,
        update: impl FnOnce(ContractAuthoringBuilder) -> ContractAuthoringBuilder,
    ) -> Self {
        let builder = self
            .authoring
            .take()
            .expect("ApiBuilder authoring method requires ApiBuilder::authoring");
        self.authoring = Some(update(builder));
        self
    }

    /// Attach summarized documentation to a Rust-authored API.
    pub fn docs_with_summary(
        self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.with_authoring(|builder| builder.docs_with_summary(summary, markdown))
    }

    /// Add one schema to a Rust-authored API.
    pub fn schema(self, name: impl Into<String>, schema: Value) -> Self {
        self.with_authoring(|builder| builder.schema(name, schema))
    }

    /// Add one declared capability to a Rust-authored API.
    pub fn capability(self, name: impl Into<String>, metadata: ContractCapabilityMetadata) -> Self {
        self.with_authoring(|builder| builder.capability(name, metadata))
    }

    /// Export one schema from a Rust-authored API.
    pub fn export_schema(self, name: impl Into<String>) -> Self {
        self.with_authoring(|builder| builder.export_schema(name))
    }

    /// Add one declared error to a Rust-authored API.
    pub fn error(
        self,
        name: impl Into<String>,
        error_type: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        self.with_authoring(|builder| builder.error(name, error_type, schema))
    }

    /// Add one RPC surface to a Rust-authored API.
    pub fn rpc(self, name: impl Into<String>, rpc: ContractRpcMethod) -> Self {
        self.with_authoring(|builder| builder.rpc(name, rpc))
    }

    /// Add an operation surface to the native API source.
    pub fn operation(self, name: impl Into<String>, operation: ContractOperation) -> Self {
        self.with_authoring(|builder| builder.operation(name, operation))
    }

    /// Add one event surface to a Rust-authored API.
    pub fn event(self, name: impl Into<String>, event: ContractEvent) -> Self {
        self.with_authoring(|builder| builder.event(name, event))
    }

    /// Add one feed surface to a Rust-authored API.
    pub fn feed(self, name: impl Into<String>, feed: ContractFeed) -> Self {
        self.with_authoring(|builder| builder.feed(name, feed))
    }

    /// Add one state declaration to a Rust-authored API.
    pub fn state(self, name: impl Into<String>, state: ContractStateStore) -> Self {
        self.with_authoring(|builder| builder.state(name, state))
    }

    /// Run the strict authoring lint without parsing the value.
    pub fn lint(&self) -> Result<(), ContractsError> {
        lint_api_authoring(&self.authoring_value()?)?;
        Ok(())
    }

    /// Parse and normalize the native API artifact.
    pub fn build(&self) -> Result<ApiArtifact, ContractsError> {
        self.finalize()
    }

    /// Lint and parse the native API artifact at the authoring boundary.
    pub fn finalize(&self) -> Result<ApiArtifact, ContractsError> {
        self.lint()?;
        Ok(parse_api(&self.authoring_value()?)?)
    }

    /// Return the normalized native API value.
    pub fn normalized(&self) -> Result<Value, ContractsError> {
        Ok(self.build()?.normalized_value()?)
    }

    /// Return the semantic API artifact digest.
    pub fn digest(&self) -> Result<String, ContractsError> {
        Ok(self.build()?.digest()?)
    }

    fn authoring_value(&self) -> Result<Value, ContractsError> {
        if let Some(builder) = &self.authoring {
            return builder
                .clone()
                .build_api()?
                .normalized_value()
                .map_err(Into::into);
        }
        Ok(self.value.clone())
    }
}

/// Native, non-serializable artifacts produced by [`ContractBuilder`].
#[derive(Debug)]
pub struct ContractArtifacts {
    api: ApiArtifact,
    participant: ParticipantArtifact,
    referenced_apis: BTreeMap<String, ApiArtifact>,
    resolved: ResolvedParticipant,
    api_digest: String,
    participant_digest: String,
    participant_needs_digest: String,
    required_grants: GrantSet,
    optional_grants: GrantSet,
}

impl ContractArtifacts {
    /// Return the validated owned API artifact.
    pub fn api(&self) -> &ApiArtifact {
        &self.api
    }

    /// Return the normalized native API JSON value.
    pub fn api_value(&self) -> Result<Value, ContractsError> {
        Ok(self.api.normalized_value()?)
    }

    /// Return the validated participant artifact.
    pub fn participant(&self) -> &ParticipantArtifact {
        &self.participant
    }

    /// Return the normalized native participant JSON value.
    pub fn participant_value(&self) -> Result<Value, ContractsError> {
        Ok(self.participant.normalized_value()?)
    }

    /// Return the contextual participant resolution.
    pub fn resolved(&self) -> &ResolvedParticipant {
        &self.resolved
    }

    /// Return the exact native API artifacts used to resolve the participant.
    pub fn referenced_apis(&self) -> &BTreeMap<String, ApiArtifact> {
        &self.referenced_apis
    }

    /// Return the owned API digest.
    pub fn api_digest(&self) -> Result<String, ContractsError> {
        Ok(self.api_digest.clone())
    }

    /// Return the semantic participant artifact digest.
    pub fn participant_digest(&self) -> Result<String, ContractsError> {
        Ok(self.participant_digest.clone())
    }

    /// Return the authoritative participant-needs digest.
    pub fn participant_needs_digest(&self) -> Result<String, ContractsError> {
        Ok(self.participant_needs_digest.clone())
    }

    /// Return exact required permission grants.
    pub fn required_grants(&self) -> &GrantSet {
        &self.required_grants
    }

    /// Return exact optional permission grants.
    pub fn optional_grants(&self) -> &GrantSet {
        &self.optional_grants
    }
}

/// Builder for native API and participant artifacts from one contract source.
#[derive(Debug)]
pub struct ContractBuilder {
    api: Value,
    participant: Value,
    referenced_apis: BTreeMap<String, Value>,
    authoring: Option<ContractAuthoringBuilder>,
}

impl ContractBuilder {
    /// Start from one already-authored native API and participant artifact.
    pub fn from_native(api: Value, participant: Value) -> Self {
        Self {
            api,
            participant,
            referenced_apis: BTreeMap::new(),
            authoring: None,
        }
    }

    /// Start authoring a participant and its owned API.
    ///
    /// `participant_id` is first and `api_id` is second. They are independent
    /// identities and may differ.
    pub fn authoring(
        participant_id: impl Into<String>,
        api_id: impl Into<String>,
        api_version: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        kind: ContractKind,
    ) -> Self {
        Self {
            api: Value::Null,
            participant: Value::Null,
            referenced_apis: BTreeMap::new(),
            authoring: Some(ContractAuthoringBuilder::new_contract(
                api_id,
                api_version,
                participant_id,
                display_name,
                description,
                kind,
            )),
        }
    }

    fn with_authoring(
        mut self,
        update: impl FnOnce(ContractAuthoringBuilder) -> ContractAuthoringBuilder,
    ) -> Self {
        self.authoring = Some(update(self.authoring.take().expect(
            "ContractBuilder authoring method requires ContractBuilder::authoring",
        )));
        self
    }

    /// Attach summarized documentation to the contract.
    pub fn docs_with_summary(
        self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.with_authoring(|builder| builder.docs_with_summary(summary, markdown))
    }

    /// Add one API schema.
    pub fn schema(self, name: impl Into<String>, schema: Value) -> Self {
        self.with_authoring(|builder| builder.schema(name, schema))
    }

    /// Add one API capability declaration.
    pub fn capability(self, name: impl Into<String>, metadata: ContractCapabilityMetadata) -> Self {
        self.with_authoring(|builder| builder.capability(name, metadata))
    }

    /// Export one API schema.
    pub fn export_schema(self, name: impl Into<String>) -> Self {
        self.with_authoring(|builder| builder.export_schema(name))
    }

    /// Add one API error declaration.
    pub fn error(
        self,
        name: impl Into<String>,
        error_type: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        self.with_authoring(|builder| builder.error(name, error_type, schema))
    }

    /// Add one API RPC.
    pub fn rpc(self, name: impl Into<String>, rpc: ContractRpcMethod) -> Self {
        self.with_authoring(|builder| builder.rpc(name, rpc))
    }

    /// Add one API operation.
    pub fn operation(self, name: impl Into<String>, operation: ContractOperation) -> Self {
        self.with_authoring(|builder| builder.operation(name, operation))
    }

    /// Add one API event.
    pub fn event(self, name: impl Into<String>, event: ContractEvent) -> Self {
        self.with_authoring(|builder| builder.event(name, event))
    }

    /// Add one API feed.
    pub fn feed(self, name: impl Into<String>, feed: ContractFeed) -> Self {
        self.with_authoring(|builder| builder.feed(name, feed))
    }

    /// Add one API and participant State declaration.
    pub fn state(self, name: impl Into<String>, state: ContractStateStore) -> Self {
        self.with_authoring(|builder| builder.state(name, state))
    }

    /// Add one required dependency selection.
    pub fn use_ref(self, alias: impl Into<String>, use_ref: ContractUseRef) -> Self {
        self.with_authoring(|builder| builder.use_ref(alias, use_ref))
    }

    /// Add one optional dependency selection.
    pub fn optional_use_ref(self, alias: impl Into<String>, use_ref: ContractUseRef) -> Self {
        self.with_authoring(|builder| builder.optional_use_ref(alias, use_ref))
    }

    /// Add one participant KV resource.
    pub fn kv_resource(self, name: impl Into<String>, resource: ContractKvResource) -> Self {
        self.with_authoring(|builder| builder.kv_resource(name, resource))
    }

    /// Add one participant object-store resource.
    pub fn store_resource(self, name: impl Into<String>, resource: ContractStoreResource) -> Self {
        self.with_authoring(|builder| builder.store_resource(name, resource))
    }

    /// Add one participant job queue.
    pub fn job_queue(self, name: impl Into<String>, queue: ContractJobQueueResource) -> Self {
        self.with_authoring(|builder| builder.job_queue(name, queue))
    }

    /// Add one participant event consumer.
    pub fn event_consumer(
        self,
        name: impl Into<String>,
        consumer: ContractEventConsumerGroup,
    ) -> Self {
        self.with_authoring(|builder| builder.event_consumer(name, consumer))
    }

    /// Start a native participant facade for an API-owned service source.
    ///
    /// `participant_id` is explicit and is never derived from the API. The
    /// participant copies the API metadata and owned surfaces and pins that API
    /// under `implements.self` with its exact digest.
    pub fn from_api(
        participant_id: impl Into<String>,
        api: Value,
        kind: ContractKind,
    ) -> Result<Self, ContractsError> {
        let api_artifact = ApiBuilder::new(api).build()?;
        let api_value = api_artifact.normalized_value()?;
        let mut participant = Map::new();
        participant.insert(
            "format".to_owned(),
            Value::String("trellis.participant.v1".to_owned()),
        );
        participant.insert("kind".to_owned(), serde_json::to_value(kind)?);
        participant.insert("id".to_owned(), Value::String(participant_id.into()));
        for field in [
            "displayName",
            "description",
            "docs",
            "schemas",
            "state",
            "resources",
        ] {
            if let Some(value) = api_value.get(field) {
                participant.insert(field.to_owned(), value.clone());
            }
        }
        participant.insert(
            "implements".to_owned(),
            json!({
                "self": {
                    "api": api_artifact.id(),
                    "apiDigest": api_artifact.digest()?,
                }
            }),
        );
        Ok(Self::from_native(api_value, Value::Object(participant)))
    }

    /// Supply one referenced native API artifact.
    pub fn referenced_api(mut self, id: impl Into<String>, api: Value) -> Self {
        self.referenced_apis.insert(id.into(), api);
        self
    }

    /// Supply all referenced native API artifacts.
    pub fn referenced_apis(mut self, apis: BTreeMap<String, Value>) -> Self {
        self.referenced_apis = apis;
        self
    }

    /// Build, lint, parse, normalize, and resolve the native artifacts.
    pub fn build(self) -> Result<ContractArtifacts, ContractsError> {
        if let Some(builder) = self.authoring {
            return builder.build_with_referenced_apis(self.referenced_apis);
        }
        let api_value = self.api;
        let participant_value = self.participant;
        lint_api_authoring(&api_value)?;
        lint_participant_authoring(&participant_value)?;
        let api = parse_api(&api_value)?;
        let participant = parse_participant(&participant_value)?;
        let api_digest = api.digest()?;
        let participant_digest = participant.digest()?;
        let mut apis = BTreeMap::new();
        let mut referenced_apis = BTreeMap::new();
        for (id, value) in &self.referenced_apis {
            let api = parse_api(value)?;
            if api.id() != id {
                return Err(invalid(format!(
                    "referenced API map key '{id}' does not match artifact id '{}'",
                    api.id()
                )));
            }
            apis.insert(id.clone(), api.clone());
            referenced_apis.insert(id.clone(), api);
        }
        apis.insert(api.id().to_owned(), api.clone());
        let resolved = resolve_participant(&participant, &apis)?;
        let participant_needs_digest = resolved.needs().digest()?;
        let required_grants = resolved.proposal().required().grant_set().clone();
        let optional_grants = resolved.proposal().optional().grant_set().clone();
        Ok(ContractArtifacts {
            api,
            participant,
            referenced_apis,
            resolved,
            api_digest,
            participant_digest,
            participant_needs_digest,
            required_grants,
            optional_grants,
        })
    }

    /// Finalize the native API and participant artifacts.
    pub fn finalize(self) -> Result<ContractArtifacts, ContractsError> {
        self.build()
    }
}

pub(crate) fn build_api_value(source: &AuthoringState) -> Result<Value, ContractsError> {
    let contract = api_projection(source)?;
    build_api_from_projection(&contract)
}

fn build_api_from_projection(contract: &Map<String, Value>) -> Result<Value, ContractsError> {
    let mut api = Map::new();
    api.insert(
        "format".to_owned(),
        Value::String("trellis.api.v1".to_owned()),
    );
    for field in [
        "id",
        "version",
        "displayName",
        "description",
        "docs",
        "schemas",
        "exports",
    ] {
        copy(contract, &mut api, field);
    }
    normalize_embedded_schemas(&mut api);
    for section in ["rpc", "operations", "events", "feeds", "state"] {
        if let Some(Value::Object(definitions)) = contract.get(section) {
            let mut lowered = definitions
                .iter()
                .filter(|(name, _)| !name.starts_with("_removed."))
                .map(|(name, value)| (name.clone(), value.clone()))
                .collect::<serde_json::Map<_, _>>();
            for definition in lowered.values_mut().filter_map(Value::as_object_mut) {
                definition.remove("subject");
                definition.remove("capabilities");
                if let Some(transfer) = definition
                    .get("transfer")
                    .and_then(Value::as_object)
                    .and_then(|transfer| transfer.get("direction"))
                    .cloned()
                {
                    definition.insert("transfer".to_owned(), json!({"direction": transfer}));
                }
                if definition.get("internal") == Some(&Value::Bool(false)) {
                    definition.remove("internal");
                }
                if definition.get("cancel") == Some(&Value::Bool(false)) {
                    definition.remove("cancel");
                }
                if definition
                    .get("signals")
                    .and_then(Value::as_object)
                    .is_some_and(Map::is_empty)
                {
                    definition.remove("signals");
                }
                if definition
                    .get("params")
                    .and_then(Value::as_array)
                    .is_some_and(Vec::is_empty)
                {
                    definition.remove("params");
                }
                if definition.get("class").and_then(Value::as_str) == Some("domain") {
                    definition.remove("class");
                }
                if definition
                    .get("acceptedVersions")
                    .and_then(Value::as_object)
                    .is_some_and(Map::is_empty)
                {
                    definition.remove("acceptedVersions");
                }
                if let Some(Value::Array(errors)) = definition.get_mut("errors") {
                    for error in errors {
                        if let Some(error_type) = error.get("type").and_then(Value::as_str) {
                            *error = Value::String(error_type.to_owned());
                        }
                    }
                }
            }
            if !lowered.is_empty() {
                api.insert(section.to_owned(), Value::Object(lowered));
            }
        }
    }
    let capabilities = compile_capabilities(contract);
    if !capabilities.is_empty() {
        api.insert("capabilities".to_owned(), Value::Object(capabilities));
    }
    if let Some(Value::Object(declarations)) = contract.get("capabilities") {
        let consent = declarations
            .iter()
            .map(|(name, metadata)| {
                let metadata = metadata
                    .as_object()
                    .ok_or_else(|| invalid(format!("capabilities.{name} must be an object")))?;
                Ok((
                    name.clone(),
                    json!({
                        "title": metadata
                            .get("displayName")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                        "description": metadata
                            .get("description")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                        "consequence": metadata
                            .get("consequence")
                            .and_then(Value::as_str)
                            .unwrap_or_default(),
                    }),
                ))
            })
            .collect::<Result<Map<_, _>, ContractsError>>()?;
        if !consent.is_empty() {
            api.insert("consent".to_owned(), Value::Object(consent));
        }
    }
    if let Some(Value::Object(definitions)) = contract.get("errors") {
        let mut lowered = definitions.clone();
        for definition in lowered.values_mut().filter_map(Value::as_object_mut) {
            definition.remove("type");
        }
        if !lowered.is_empty() {
            api.insert("errors".to_owned(), Value::Object(lowered));
        }
    }
    let referenced_errors = ["rpc", "operations"]
        .into_iter()
        .filter_map(|section| api.get(section).and_then(Value::as_object))
        .flat_map(|definitions| definitions.values())
        .filter_map(|definition| definition.get("errors").and_then(Value::as_array))
        .flatten()
        .filter_map(Value::as_str)
        .map(str::to_owned)
        .collect::<Vec<_>>();
    if !referenced_errors.is_empty() {
        let errors = api
            .entry("errors".to_owned())
            .or_insert_with(|| Value::Object(Map::new()))
            .as_object_mut()
            .expect("compiled errors are an object");
        for error in referenced_errors {
            errors.entry(error).or_insert_with(|| json!({}));
        }
    }
    Ok(Value::Object(api))
}

fn compile_capabilities(contract: &Map<String, Value>) -> Map<String, Value> {
    let api_id = contract
        .get("id")
        .and_then(Value::as_str)
        .unwrap_or_default();
    let mut allows = BTreeMap::<String, Vec<Value>>::new();
    if let Some(Value::Object(declarations)) = contract.get("capabilities") {
        for name in declarations.keys() {
            allows.insert(name.clone(), Vec::new());
        }
    }
    if let Some(declarations) = contract.get("capabilities").and_then(Value::as_object) {
        for name in declarations.keys() {
            allows.entry(name.clone()).or_default();
        }
    }
    for (section, directions, surface) in [
        ("rpc", &[("call", "call")][..], "rpc"),
        (
            "operations",
            &[
                ("call", "invoke"),
                ("observe", "observe"),
                ("cancel", "cancel"),
            ][..],
            "operation",
        ),
        (
            "events",
            &[("publish", "publish"), ("subscribe", "subscribe")][..],
            "event",
        ),
        ("feeds", &[("subscribe", "subscribe")][..], "feed"),
    ] {
        let Some(definitions) = contract.get(section).and_then(Value::as_object) else {
            continue;
        };
        let mut names = definitions.keys().collect::<Vec<_>>();
        names.sort();
        for name in names {
            let definition = &definitions[name];
            let capabilities = definition.get("capabilities").and_then(Value::as_object);
            for (direction, action) in directions {
                for capability in capabilities
                    .and_then(|value| value.get(*direction))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    allows.entry(capability.to_owned()).or_default().push(json!({
                        "action": action,
                        "target": {"kind": "apiSurface", "api": api_id, "surface": surface, "name": name}
                    }));
                }
            }
            if section == "operations" {
                let mut signals = definition
                    .get("signals")
                    .and_then(Value::as_object)
                    .map(|value| value.keys().collect::<Vec<_>>())
                    .unwrap_or_default();
                signals.sort();
                for capability in capabilities
                    .and_then(|value| value.get("control"))
                    .and_then(Value::as_array)
                    .into_iter()
                    .flatten()
                    .filter_map(Value::as_str)
                {
                    for signal in &signals {
                        allows.entry(capability.to_owned()).or_default().push(json!({
                            "action": "control",
                            "target": {"kind": "operationSignal", "api": api_id, "operation": name, "signal": signal}
                        }));
                    }
                }
            }
        }
    }
    allows
        .into_iter()
        .map(|(name, allows)| (name, json!({"allows": allows})))
        .collect()
}

pub(crate) fn build_participant_value(
    source: &AuthoringState,
    own_api: &ApiArtifact,
    apis: &BTreeMap<String, ApiArtifact>,
) -> Result<Value, ContractsError> {
    let contract = participant_projection(source)?;
    build_participant_from_projection(&contract, own_api, apis)
}

fn build_participant_from_projection(
    contract: &Map<String, Value>,
    own_api: &ApiArtifact,
    apis: &BTreeMap<String, ApiArtifact>,
) -> Result<Value, ContractsError> {
    let mut participant = Map::new();
    participant.insert(
        "format".to_owned(),
        Value::String("trellis.participant.v1".to_owned()),
    );
    for field in [
        "id",
        "displayName",
        "description",
        "docs",
        "kind",
        "schemas",
        "state",
    ] {
        copy(contract, &mut participant, field);
    }
    normalize_embedded_schemas(&mut participant);
    let api_digest = own_api
        .digest()
        .map_err(|error| invalid(error.to_string()))?;
    if ["rpc", "operations", "events", "feeds", "state"]
        .iter()
        .any(|section| contract.get(*section).is_some_and(nonempty_object))
    {
        let operation_transfers = contract
            .get("operations")
            .and_then(Value::as_object)
            .into_iter()
            .flatten()
            .filter_map(|(name, operation)| {
                let mut transfer = operation.get("transfer")?.as_object()?.clone();
                (transfer.get("direction").and_then(Value::as_str) == Some("send")).then(|| {
                    transfer.remove("direction");
                    (name.clone(), Value::Object(transfer))
                })
            })
            .collect::<Map<_, _>>();
        let mut implementation = json!({"api": own_api.id(), "apiDigest": api_digest});
        if !operation_transfers.is_empty() {
            implementation["operationTransfers"] = Value::Object(operation_transfers);
        }
        participant.insert("implements".to_owned(), json!({"self": implementation}));
    }
    if let Some(Value::Object(groups)) = contract.get("uses") {
        let mut uses = Map::new();
        for group in ["required", "optional"] {
            let Some(Value::Object(references)) = groups.get(group) else {
                continue;
            };
            let mut lowered = Map::new();
            for (alias, reference) in references {
                let reference = reference
                    .as_object()
                    .ok_or_else(|| invalid(format!("uses.{group}.{alias} must be an object")))?;
                let api_id = reference
                    .get("contract")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid(format!("uses.{group}.{alias}.contract is required")))?;
                let api = apis.get(api_id).ok_or_else(|| {
                    invalid(format!("referenced API artifact '{api_id}' is required"))
                })?;
                let mut used = Map::new();
                used.insert("api".to_owned(), Value::String(api_id.to_owned()));
                used.insert(
                    "apiDigest".to_owned(),
                    Value::String(api.digest().map_err(|error| invalid(error.to_string()))?),
                );
                copy(reference, &mut used, "rpc");
                if let Some(Value::Object(operations)) = reference.get("operations") {
                    let calls = operations.get("call").cloned().unwrap_or_else(|| json!([]));
                    let api_value = api
                        .normalized_value()
                        .map_err(|error| invalid(error.to_string()))?;
                    let cancel = operations
                        .get("cancel")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter(|name| {
                            name.as_str().is_some_and(|name| {
                                api_value["operations"][name]["cancel"].as_bool() == Some(true)
                            })
                        })
                        .cloned()
                        .collect::<Vec<_>>();
                    let control = operations
                        .get("control")
                        .and_then(Value::as_array)
                        .into_iter()
                        .flatten()
                        .filter_map(Value::as_str)
                        .map(|name| {
                            let signals = api_value["operations"][name]["signals"]
                                .as_object()
                                .map(|signals| {
                                    let mut names = signals.keys().cloned().collect::<Vec<_>>();
                                    names.sort();
                                    names
                                })
                                .unwrap_or_default();
                            (name.to_owned(), json!(signals))
                        })
                        .collect::<Map<_, _>>();
                    used.insert(
                        "operations".to_owned(),
                        json!({
                            "invoke": calls.clone(),
                            "observe": calls,
                            "cancel": cancel,
                            "control": control,
                        }),
                    );
                }
                copy(reference, &mut used, "events");
                copy(reference, &mut used, "feeds");
                lowered.insert(alias.clone(), Value::Object(used));
            }
            if !lowered.is_empty() {
                uses.insert(group.to_owned(), Value::Object(lowered));
            }
        }
        if !uses.is_empty() {
            participant.insert("uses".to_owned(), Value::Object(uses));
        }
    }
    if let Some(resources) = contract.get("resources") {
        participant.insert("resources".to_owned(), resources.clone());
    }
    if let Some(Value::Object(jobs)) = contract.get("jobs") {
        let queues = jobs
            .get("queues")
            .and_then(Value::as_object)
            .cloned()
            .unwrap_or_else(|| jobs.clone());
        participant.insert("jobQueues".to_owned(), Value::Object(queues));
    }
    if let Some(Value::Object(consumers)) = contract.get("eventConsumers") {
        let consumers = consumers
            .iter()
            .map(|(name, consumer)| {
                let mut consumer = consumer
                    .as_object()
                    .cloned()
                    .ok_or_else(|| invalid(format!("eventConsumers.{name} must be an object")))?;
                let mut events = consumer
                    .remove("uses")
                    .and_then(|value| value.as_object().cloned())
                    .unwrap_or_default();
                if let Some(owned) = consumer.remove("self") {
                    events.insert("self".to_owned(), owned);
                }
                consumer.insert("events".to_owned(), Value::Object(events));
                Ok((name.clone(), Value::Object(consumer)))
            })
            .collect::<Result<Map<_, _>, ContractsError>>()?;
        participant.insert("eventConsumers".to_owned(), Value::Object(consumers));
    }
    Ok(Value::Object(participant))
}

fn api_projection(source: &AuthoringState) -> Result<Map<String, Value>, ContractsError> {
    let mut value = Map::new();
    insert(&mut value, "id", &source.api_id)?;
    insert(&mut value, "version", &source.api_version)?;
    insert(&mut value, "displayName", &source.display_name)?;
    insert(&mut value, "description", &source.description)?;
    insert_if_some(&mut value, "docs", source.docs.as_ref())?;
    insert_nonempty(&mut value, "capabilities", &source.capabilities)?;
    insert_nonempty(&mut value, "schemas", &source.schemas)?;
    insert_nonempty(&mut value, "exports", &source.exports)?;
    insert_nonempty(&mut value, "state", &source.state)?;
    insert_nonempty(&mut value, "rpc", &source.rpc)?;
    insert_nonempty(&mut value, "operations", &source.operations)?;
    insert_nonempty(&mut value, "events", &source.events)?;
    insert_nonempty(&mut value, "feeds", &source.feeds)?;
    insert_nonempty(&mut value, "errors", &source.errors)?;
    Ok(value)
}

fn participant_projection(source: &AuthoringState) -> Result<Map<String, Value>, ContractsError> {
    let mut value = Map::new();
    insert(
        &mut value,
        "id",
        source
            .participant_id
            .as_ref()
            .expect("participant projection requires ContractBuilder::authoring"),
    )?;
    insert(&mut value, "displayName", &source.display_name)?;
    insert(&mut value, "description", &source.description)?;
    insert_if_some(&mut value, "docs", source.docs.as_ref())?;
    insert(&mut value, "kind", &source.kind)?;
    insert_nonempty(&mut value, "schemas", &source.schemas)?;
    insert_nonempty(&mut value, "state", &source.state)?;
    insert_nonempty(&mut value, "rpc", &source.rpc)?;
    insert_nonempty(&mut value, "operations", &source.operations)?;
    insert_nonempty(&mut value, "events", &source.events)?;
    insert_nonempty(&mut value, "feeds", &source.feeds)?;
    insert_nonempty(&mut value, "uses", &source.uses)?;
    insert_nonempty(&mut value, "resources", &source.resources)?;
    insert_nonempty(&mut value, "jobs", &source.jobs)?;
    insert_nonempty(&mut value, "eventConsumers", &source.event_consumers)?;
    Ok(value)
}

fn insert(
    target: &mut Map<String, Value>,
    key: &str,
    value: &impl serde::Serialize,
) -> Result<(), ContractsError> {
    target.insert(key.to_owned(), serde_json::to_value(value)?);
    Ok(())
}

fn insert_if_some<T: serde::Serialize>(
    target: &mut Map<String, Value>,
    key: &str,
    value: Option<&T>,
) -> Result<(), ContractsError> {
    if let Some(value) = value {
        insert(target, key, value)?;
    }
    Ok(())
}

fn insert_nonempty(
    target: &mut Map<String, Value>,
    key: &str,
    value: &impl serde::Serialize,
) -> Result<(), ContractsError> {
    let value = serde_json::to_value(value)?;
    if !matches!(&value, Value::Object(map) if map.is_empty()) {
        target.insert(key.to_owned(), value);
    }
    Ok(())
}

fn copy(source: &Map<String, Value>, target: &mut Map<String, Value>, key: &str) {
    if let Some(value) = source.get(key) {
        if !value.is_null() && !matches!(value, Value::Object(map) if map.is_empty()) {
            target.insert(key.to_owned(), value.clone());
        }
    }
}

fn nonempty_object(value: &Value) -> bool {
    value.as_object().is_some_and(|object| !object.is_empty())
}

fn normalize_embedded_schemas(artifact: &mut Map<String, Value>) {
    if let Some(Value::Object(schemas)) = artifact.get_mut("schemas") {
        for schema in schemas.values_mut() {
            normalize_schema(schema);
        }
    }
}

fn normalize_schema(value: &mut Value) {
    match value {
        Value::Array(values) => {
            for value in values {
                normalize_schema(value);
            }
        }
        Value::Object(object) => {
            if object.remove("patternProperties").is_some() {
                object.insert("additionalProperties".to_owned(), Value::Bool(true));
            }
            for value in object.values_mut() {
                normalize_schema(value);
            }
        }
        _ => {}
    }
}

fn invalid(details: String) -> ContractsError {
    ContractsError::SchemaValidation {
        kind: "compiled protocol artifacts",
        details,
    }
}
