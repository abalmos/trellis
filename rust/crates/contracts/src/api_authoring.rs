use serde_json::Value;

use crate::{
    authoring_model::AuthoringState, ContractCapabilities, ContractCapabilityMetadata,
    ContractDocs, ContractErrorDecl, ContractErrorRef, ContractEvent, ContractExports,
    ContractFeed, ContractJobQueueResource, ContractKind, ContractKvResource, ContractOperation,
    ContractOperationSignal, ContractOperationTransfer, ContractOperationTransferDirection,
    ContractResources, ContractRpcMethod, ContractRpcTransfer, ContractRpcTransferDirection,
    ContractSchemaRef, ContractStateKind, ContractStateStore, ContractStoreResource,
    ContractUseFeed, ContractUseOperation, ContractUsePubSub, ContractUseRef, ContractUseRpc,
    ContractsError, FeedCapabilities, JobKeyConcurrencyDescriptor, JobQueueDepthDescriptor,
    OperationCapabilities, PubSubCapabilities, RpcCapabilities,
};

/// Private typed builder over shared in-memory authoring state.
#[derive(Clone, Debug)]
pub(crate) struct ContractAuthoringBuilder {
    manifest: AuthoringState,
}

impl ContractAuthoringBuilder {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(new), "`.")]
    pub fn new_api(
        api_id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
    ) -> Self {
        Self::new(api_id, None, display_name, description, ContractKind::App)
    }

    pub fn new_contract(
        api_id: impl Into<String>,
        participant_id: impl Into<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        kind: ContractKind,
    ) -> Self {
        Self::new(
            api_id,
            Some(participant_id.into()),
            display_name,
            description,
            kind,
        )
    }

    fn new(
        api_id: impl Into<String>,
        participant_id: Option<String>,
        display_name: impl Into<String>,
        description: impl Into<String>,
        kind: ContractKind,
    ) -> Self {
        Self {
            manifest: AuthoringState {
                api_id: api_id.into(),
                participant_id,
                display_name: display_name.into(),
                description: description.into(),
                docs: None,
                kind,
                capabilities: Default::default(),
                schemas: Default::default(),
                exports: ContractExports::default(),
                uses: Default::default(),
                state: Default::default(),
                rpc: Default::default(),
                operations: Default::default(),
                events: Default::default(),
                feeds: Default::default(),
                errors: Default::default(),
                jobs: Default::default(),
                event_consumers: Default::default(),
                resources: ContractResources::default(),
            },
        }
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(schema), "`.")]
    pub fn schema(mut self, name: impl Into<String>, schema: Value) -> Self {
        self.manifest.schemas.insert(name.into(), schema);
        self
    }

    /// Attach summarized programmer-facing Markdown documentation to the contract.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.manifest.docs = Some(ContractDocs {
            summary: Some(summary.into()),
            markdown: markdown.into(),
        });
        self
    }

    /// Declare a contract-local structured error backed by a named schema.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(error), "`.")]
    pub fn error(
        mut self,
        name: impl Into<String>,
        error_type: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        self.manifest.errors.insert(
            name.into(),
            ContractErrorDecl {
                error_type: error_type.into(),
                schema: Some(schema_ref(schema)),
            },
        );
        self
    }

    /// Declare human-facing metadata for a contract-local capability.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(capability), "`.")]
    pub fn capability(
        mut self,
        name: impl Into<String>,
        metadata: ContractCapabilityMetadata,
    ) -> Self {
        self.manifest.capabilities.insert(name.into(), metadata);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(use_ref), "`.")]
    pub fn use_ref(mut self, alias: impl Into<String>, use_ref: ContractUseRef) -> Self {
        self.manifest
            .uses
            .required_mut()
            .insert(alias.into(), use_ref);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(optional_use_ref), "`.")]
    pub fn optional_use_ref(mut self, alias: impl Into<String>, use_ref: ContractUseRef) -> Self {
        self.manifest
            .uses
            .optional_mut()
            .insert(alias.into(), use_ref);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(export_schema), "`.")]
    pub fn export_schema(mut self, name: impl Into<String>) -> Self {
        self.manifest.exports.schemas.push(name.into());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(state), "`.")]
    pub fn state(mut self, name: impl Into<String>, state: ContractStateStore) -> Self {
        self.manifest.state.insert(name.into(), state);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(rpc), "`.")]
    pub fn rpc(mut self, name: impl Into<String>, rpc: ContractRpcMethod) -> Self {
        self.manifest.rpc.insert(name.into(), rpc);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(operation), "`.")]
    pub fn operation(mut self, name: impl Into<String>, operation: ContractOperation) -> Self {
        self.manifest.operations.insert(name.into(), operation);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(event), "`.")]
    pub fn event(mut self, name: impl Into<String>, event: ContractEvent) -> Self {
        self.manifest.events.insert(name.into(), event);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(feed), "`.")]
    pub fn feed(mut self, name: impl Into<String>, feed: ContractFeed) -> Self {
        self.manifest.feeds.insert(name.into(), feed);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(kv_resource), "`.")]
    pub fn kv_resource(mut self, name: impl Into<String>, kv: ContractKvResource) -> Self {
        self.manifest.resources.kv.insert(name.into(), kv);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(store_resource), "`.")]
    pub fn store_resource(mut self, name: impl Into<String>, store: ContractStoreResource) -> Self {
        self.manifest.resources.store.insert(name.into(), store);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(job_queue), "`.")]
    pub fn job_queue(
        mut self,
        queue_type: impl Into<String>,
        queue: ContractJobQueueResource,
    ) -> Self {
        self.manifest.jobs.insert(queue_type.into(), queue);
        self
    }

    /// Add one durable event consumer group to the participant.
    pub fn event_consumer(
        mut self,
        name: impl Into<String>,
        consumer: crate::ContractEventConsumerGroup,
    ) -> Self {
        self.manifest.event_consumers.insert(name.into(), consumer);
        self
    }

    pub(crate) fn build_api(mut self) -> Result<crate::ApiArtifact, ContractsError> {
        self.project_declared_capabilities()?;
        crate::ApiBuilder::new(crate::native_artifacts::build_api_value(&self.manifest)?).build()
    }

    /// Finalize with exact native API artifacts for every selected dependency.
    pub fn build_with_referenced_apis(
        mut self,
        referenced_apis: std::collections::BTreeMap<String, Value>,
    ) -> Result<crate::ContractArtifacts, ContractsError> {
        self.project_declared_capabilities()?;
        let api = crate::native_artifacts::build_api_value(&self.manifest)?;
        let api_artifact = crate::ApiBuilder::new(api.clone()).build()?;
        let mut apis = referenced_apis
            .iter()
            .map(|(id, value)| Ok((id.clone(), crate::ApiBuilder::new(value.clone()).build()?)))
            .collect::<Result<std::collections::BTreeMap<_, _>, ContractsError>>()?;
        apis.insert(api_artifact.id().to_owned(), api_artifact.clone());
        let participant =
            crate::native_artifacts::build_participant_value(&self.manifest, &api_artifact, &apis)?;
        crate::ContractBuilder::from_native(api, participant)
            .referenced_apis(referenced_apis)
            .build()
    }

    fn project_declared_capabilities(&mut self) -> Result<(), ContractsError> {
        if self.manifest.capabilities.is_empty() {
            assert_no_undeclared_local_capabilities(&self.manifest)?;
            return Ok(());
        }

        let declared = self.manifest.capabilities.clone();
        project_contract_capabilities(&mut self.manifest, &declared)?;
        Ok(())
    }
}

fn project_contract_capabilities(
    manifest: &mut AuthoringState,
    declared: &ContractCapabilities,
) -> Result<(), ContractsError> {
    let contract_id = manifest.api_id.clone();
    assert_declared_capabilities_do_not_duplicate_namespace(&contract_id, declared)?;
    manifest.capabilities = declared
        .iter()
        .map(|(name, metadata)| (global_capability_name(&contract_id, name), metadata.clone()))
        .collect();

    for method in manifest.rpc.values_mut() {
        if let Some(capabilities) = method.capabilities.as_mut() {
            project_capability_list(
                &mut capabilities.call,
                &contract_id,
                declared,
                "rpc call capabilities",
            )?;
        }
    }
    for operation in manifest.operations.values_mut() {
        if let Some(capabilities) = operation.capabilities.as_mut() {
            project_capability_list(
                &mut capabilities.call,
                &contract_id,
                declared,
                "operation call capabilities",
            )?;
            project_capability_list(
                &mut capabilities.observe,
                &contract_id,
                declared,
                "operation observe capabilities",
            )?;
            project_capability_list(
                &mut capabilities.cancel,
                &contract_id,
                declared,
                "operation cancel capabilities",
            )?;
            project_capability_list(
                &mut capabilities.control,
                &contract_id,
                declared,
                "operation control capabilities",
            )?;
        }
    }
    for event in manifest.events.values_mut() {
        if let Some(capabilities) = event.capabilities.as_mut() {
            project_capability_list(
                &mut capabilities.publish,
                &contract_id,
                declared,
                "event publish capabilities",
            )?;
            project_capability_list(
                &mut capabilities.subscribe,
                &contract_id,
                declared,
                "event subscribe capabilities",
            )?;
        }
    }
    for feed in manifest.feeds.values_mut() {
        if let Some(capabilities) = feed.capabilities.as_mut() {
            project_capability_list(
                &mut capabilities.subscribe,
                &contract_id,
                declared,
                "feed subscribe capabilities",
            )?;
        }
    }
    Ok(())
}

fn assert_declared_capabilities_do_not_duplicate_namespace(
    contract_id: &str,
    declared: &ContractCapabilities,
) -> Result<(), ContractsError> {
    let prefixes = local_capability_namespace_prefixes(contract_id);
    for capability in declared.keys() {
        for prefix in &prefixes {
            if capability.starts_with(prefix) {
                return Err(ContractsError::InvalidLocalCapability {
                    capability: capability.clone(),
                    prefix: prefix.clone(),
                });
            }
        }
    }
    Ok(())
}

fn local_capability_namespace_prefixes(contract_id: &str) -> Vec<String> {
    let namespace = contract_capability_namespace(contract_id);
    let mut prefixes = vec![format!("{namespace}.")];
    if let Some(leaf) = namespace.rsplit('.').next() {
        if leaf != namespace {
            prefixes.push(format!("{leaf}."));
        }
    }
    prefixes
}

fn project_capability_list(
    capabilities: &mut Option<Vec<String>>,
    contract_id: &str,
    declared: &ContractCapabilities,
    context: &str,
) -> Result<(), ContractsError> {
    let Some(capabilities) = capabilities else {
        return Ok(());
    };
    for capability in &mut *capabilities {
        if declared.contains_key(capability) {
            *capability = global_capability_name(contract_id, capability);
        } else if !is_external_capability_ref(capability) {
            return Err(ContractsError::UndeclaredCapability {
                context: context.to_string(),
                capability: capability.clone(),
            });
        }
    }
    capabilities.sort();
    capabilities.dedup();
    Ok(())
}

fn is_external_capability_ref(capability: &str) -> bool {
    matches!(capability, "admin" | "service") || capability.contains("::")
}

fn assert_no_undeclared_local_capabilities(
    manifest: &AuthoringState,
) -> Result<(), ContractsError> {
    for method in manifest.rpc.values() {
        if let Some(capabilities) = method.capabilities.as_ref() {
            assert_capability_list_external(&capabilities.call, "rpc call capabilities")?;
        }
    }
    for operation in manifest.operations.values() {
        if let Some(capabilities) = operation.capabilities.as_ref() {
            assert_capability_list_external(&capabilities.call, "operation call capabilities")?;
            assert_capability_list_external(
                &capabilities.observe,
                "operation observe capabilities",
            )?;
            assert_capability_list_external(&capabilities.cancel, "operation cancel capabilities")?;
            assert_capability_list_external(
                &capabilities.control,
                "operation control capabilities",
            )?;
        }
    }
    for event in manifest.events.values() {
        if let Some(capabilities) = event.capabilities.as_ref() {
            assert_capability_list_external(&capabilities.publish, "event publish capabilities")?;
            assert_capability_list_external(
                &capabilities.subscribe,
                "event subscribe capabilities",
            )?;
        }
    }
    for feed in manifest.feeds.values() {
        if let Some(capabilities) = feed.capabilities.as_ref() {
            assert_capability_list_external(
                &capabilities.subscribe,
                "feed subscribe capabilities",
            )?;
        }
    }
    Ok(())
}

fn assert_capability_list_external(
    capabilities: &Option<Vec<String>>,
    context: &str,
) -> Result<(), ContractsError> {
    let Some(capabilities) = capabilities else {
        return Ok(());
    };
    for capability in capabilities {
        if !is_external_capability_ref(capability) {
            return Err(ContractsError::UndeclaredCapability {
                context: context.to_string(),
                capability: capability.clone(),
            });
        }
    }
    Ok(())
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(schema_ref), "`.")]
pub fn schema_ref(name: impl Into<String>) -> ContractSchemaRef {
    ContractSchemaRef {
        schema: name.into(),
    }
}

/// Return the global capability namespace for a contract id.
#[doc = concat!("Constructs or updates contract data with `", stringify!(contract_capability_namespace), "`.")]
pub fn contract_capability_namespace(contract_id: &str) -> String {
    let Some((namespace, version)) = contract_id.rsplit_once("@v") else {
        return contract_id.to_string();
    };
    if version.chars().all(|char| char.is_ascii_digit()) && !version.is_empty() {
        namespace.to_string()
    } else {
        contract_id.to_string()
    }
}

/// Return the globally qualified name for a contract-local capability.
#[doc = concat!("Constructs or updates contract data with `", stringify!(global_capability_name), "`.")]
pub fn global_capability_name(contract_id: &str, local_capability: &str) -> String {
    let namespace = contract_capability_namespace(contract_id);
    for prefix in local_capability_namespace_prefixes(contract_id) {
        assert!(
        !local_capability.starts_with(&prefix),
        "local capability '{local_capability}' must not start with contract namespace prefix '{prefix}'"
    );
    }
    format!("{namespace}::{local_capability}")
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(rpc), "`.")]
pub fn rpc(
    version: impl Into<String>,
    subject: impl Into<String>,
    input_schema: impl Into<String>,
    output_schema: impl Into<String>,
) -> ContractRpcMethod {
    ContractRpcMethod {
        version: version.into(),
        subject: subject.into(),
        input: schema_ref(input_schema),
        output: schema_ref(output_schema),
        capabilities: None,
        errors: None,
        transfer: None,
        internal: None,
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(operation), "`.")]
pub fn operation(
    version: impl Into<String>,
    subject: impl Into<String>,
    input_schema: impl Into<String>,
    progress_schema: Option<impl Into<String>>,
    output_schema: Option<impl Into<String>>,
) -> ContractOperation {
    ContractOperation {
        version: version.into(),
        subject: subject.into(),
        input: schema_ref(input_schema),
        update: None,
        progress: progress_schema.map(schema_ref),
        output: output_schema.map(schema_ref),
        errors: None,
        transfer: None,
        capabilities: None,
        cancel: None,
        signals: Default::default(),
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(event), "`.")]
pub fn event(
    version: impl Into<String>,
    subject: impl Into<String>,
    event_schema: impl Into<String>,
) -> ContractEvent {
    ContractEvent {
        version: version.into(),
        subject: subject.into(),
        params: None,
        event: schema_ref(event_schema),
        capabilities: None,
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(feed), "`.")]
pub fn feed(
    version: impl Into<String>,
    subject: impl Into<String>,
    input_schema: impl Into<String>,
    event_schema: impl Into<String>,
) -> ContractFeed {
    ContractFeed {
        version: version.into(),
        subject: subject.into(),
        input: schema_ref(input_schema),
        event: schema_ref(event_schema),
        capabilities: None,
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(state), "`.")]
pub fn state(kind: ContractStateKind, schema: impl Into<String>) -> ContractStateStore {
    ContractStateStore {
        kind,
        schema: schema_ref(schema),
        state_version: None,
        accepted_versions: Default::default(),
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(use_contract), "`.")]
pub fn use_contract(contract: impl Into<String>) -> ContractUseRef {
    ContractUseRef {
        contract: contract.into(),
        rpc: None,
        operations: None,
        events: None,
        feeds: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(kv), "`.")]
pub fn kv(purpose: impl Into<String>, schema: impl Into<String>) -> ContractKvResource {
    ContractKvResource {
        purpose: purpose.into(),
        schema: schema_ref(schema),
        required: None,
        history: None,
        ttl_ms: None,
        max_value_bytes: None,
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(store), "`.")]
pub fn store(purpose: impl Into<String>) -> ContractStoreResource {
    ContractStoreResource {
        purpose: purpose.into(),
        required: None,
        ttl_ms: None,
        max_object_bytes: None,
        max_total_bytes: None,
        docs: None,
    }
}

#[doc = concat!("Constructs or updates contract data with `", stringify!(job_queue), "`.")]
pub fn job_queue(
    payload: ContractSchemaRef,
    result: Option<ContractSchemaRef>,
) -> ContractJobQueueResource {
    ContractJobQueueResource {
        payload,
        update: None,
        result,
        max_deliver: None,
        backoff_ms: None,
        ack_wait_ms: None,
        default_deadline_ms: None,
        progress: None,
        logs: None,
        dlq: None,
        key_concurrency: None,
        queue: None,
        docs: None,
    }
}

fn docs(summary: impl Into<String>, markdown: impl Into<String>) -> ContractDocs {
    ContractDocs {
        summary: Some(summary.into()),
        markdown: markdown.into(),
    }
}

impl ContractRpcMethod {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(ContractDocs {
            summary: Some(summary.into()),
            markdown: markdown.into(),
        });
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_receive_transfer), "`.")]
    pub fn with_receive_transfer(mut self) -> Self {
        self.transfer = Some(ContractRpcTransfer {
            direction: ContractRpcTransferDirection::Receive,
        });
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_call_capabilities), "`.")]
    pub fn with_call_capabilities(
        mut self,
        call: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.capabilities = Some(RpcCapabilities {
            call: Some(call.into_iter().map(Into::into).collect()),
        });
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_error_types), "`.")]
    pub fn with_error_types(
        mut self,
        error_types: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.errors = Some(
            error_types
                .into_iter()
                .map(|error_type| ContractErrorRef {
                    error_type: error_type.into(),
                })
                .collect(),
        );
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(internal), "`.")]
    pub fn internal(mut self) -> Self {
        self.internal = Some(true);
        self
    }
}

impl ContractOperation {
    /// Declare the live-only update payload schema for this operation.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_update_schema), "`.")]
    pub fn with_update_schema(mut self, schema: impl Into<String>) -> Self {
        self.update = Some(schema_ref(schema));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_transfer), "`.")]
    pub fn with_transfer(
        mut self,
        store: impl Into<String>,
        key: impl Into<String>,
        content_type: Option<impl Into<String>>,
        metadata: Option<impl Into<String>>,
        expires_in_ms: Option<i64>,
        max_bytes: Option<i64>,
    ) -> Self {
        self.transfer = Some(ContractOperationTransfer {
            direction: ContractOperationTransferDirection::Send,
            store: store.into(),
            key: key.into(),
            content_type: content_type.map(Into::into),
            metadata: metadata.map(Into::into),
            expires_in_ms,
            max_bytes,
        });
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_error_types), "`.")]
    pub fn with_error_types(
        mut self,
        error_types: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.errors = Some(
            error_types
                .into_iter()
                .map(|error_type| ContractErrorRef {
                    error_type: error_type.into(),
                })
                .collect(),
        );
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_call_capabilities), "`.")]
    pub fn with_call_capabilities(
        mut self,
        call: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(OperationCapabilities::default);
        capabilities.call = Some(call.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_observe_capabilities), "`.")]
    pub fn with_observe_capabilities(
        mut self,
        observe: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(OperationCapabilities::default);
        capabilities.observe = Some(observe.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_cancel_capabilities), "`.")]
    pub fn with_cancel_capabilities(
        mut self,
        cancel_capabilities: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(OperationCapabilities::default);
        capabilities.cancel = Some(cancel_capabilities.into_iter().map(Into::into).collect());
        self
    }

    /// Sets the capabilities required for named operation-control signals.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_control_capabilities), "`.")]
    pub fn with_control_capabilities(
        mut self,
        control_capabilities: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(OperationCapabilities::default);
        capabilities.control = Some(control_capabilities.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(cancel), "`.")]
    pub fn cancel(mut self, cancel: bool) -> Self {
        self.cancel = Some(cancel);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(signal), "`.")]
    pub fn signal(mut self, name: impl Into<String>, input: impl Into<String>) -> Self {
        self.signals.insert(
            name.into(),
            ContractOperationSignal {
                input: schema_ref(input),
                docs: None,
            },
        );
        self
    }
}

impl ContractJobQueueResource {
    /// Declare the live-only update payload schema for this jobs queue.
    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_update_schema), "`.")]
    pub fn with_update_schema(mut self, schema: impl Into<String>) -> Self {
        self.update = Some(schema_ref(schema));
        self
    }

    /// Set the per-key active job policy for this queue.
    pub fn key_concurrency(mut self, policy: JobKeyConcurrencyDescriptor) -> Self {
        self.key_concurrency = Some(policy);
        self
    }

    /// Set the queue-depth policy for this queue.
    pub fn queue_policy(mut self, policy: JobQueueDepthDescriptor) -> Self {
        self.queue = Some(policy);
        self
    }
}

impl ContractEvent {
    /// Set the payload pointers used to derive event subject parameters.
    pub fn with_params(mut self, params: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.params = Some(params.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_publish_capabilities), "`.")]
    pub fn with_publish_capabilities(
        mut self,
        publish: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(PubSubCapabilities::default);
        capabilities.publish = Some(publish.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_subscribe_capabilities), "`.")]
    pub fn with_subscribe_capabilities(
        mut self,
        subscribe: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(PubSubCapabilities::default);
        capabilities.subscribe = Some(subscribe.into_iter().map(Into::into).collect());
        self
    }
}

impl ContractFeed {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_subscribe_capabilities), "`.")]
    pub fn with_subscribe_capabilities(
        mut self,
        subscribe: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let capabilities = self
            .capabilities
            .get_or_insert_with(FeedCapabilities::default);
        capabilities.subscribe = Some(subscribe.into_iter().map(Into::into).collect());
        self
    }
}

impl ContractStateStore {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(state_version), "`.")]
    pub fn state_version(mut self, state_version: impl Into<String>) -> Self {
        self.state_version = Some(state_version.into());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(accepted_version), "`.")]
    pub fn accepted_version(
        mut self,
        version: impl Into<String>,
        schema: impl Into<String>,
    ) -> Self {
        self.accepted_versions
            .insert(version.into(), schema_ref(schema));
        self
    }
}

impl ContractUseRef {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_rpc_call), "`.")]
    pub fn with_rpc_call(mut self, call: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.rpc = Some(ContractUseRpc {
            call: Some(call.into_iter().map(Into::into).collect()),
        });
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_event_publish), "`.")]
    pub fn with_event_publish(
        mut self,
        publish: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let events = self.events.get_or_insert_with(ContractUsePubSub::default);
        events.publish = Some(publish.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_operation_call), "`.")]
    pub fn with_operation_call(
        mut self,
        call: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.operations = Some(ContractUseOperation {
            call: Some(call.into_iter().map(Into::into).collect()),
            ..ContractUseOperation::default()
        });
        self
    }

    /// Select operation cancel controls from this dependency.
    pub fn with_operation_cancel(
        mut self,
        cancel: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.operations
            .get_or_insert_with(ContractUseOperation::default)
            .cancel = Some(cancel.into_iter().map(Into::into).collect());
        self
    }

    /// Select operation signal controls from this dependency.
    pub fn with_operation_control(
        mut self,
        control: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.operations
            .get_or_insert_with(ContractUseOperation::default)
            .control = Some(control.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_event_subscribe), "`.")]
    pub fn with_event_subscribe(
        mut self,
        subscribe: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        let events = self.events.get_or_insert_with(ContractUsePubSub::default);
        events.subscribe = Some(subscribe.into_iter().map(Into::into).collect());
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(with_feed_subscribe), "`.")]
    pub fn with_feed_subscribe(
        mut self,
        subscribe: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.feeds = Some(ContractUseFeed {
            subscribe: Some(subscribe.into_iter().map(Into::into).collect()),
        });
        self
    }
}

impl ContractKvResource {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(required), "`.")]
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(history), "`.")]
    pub fn history(mut self, history: i64) -> Self {
        self.history = Some(history);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(ttl_ms), "`.")]
    pub fn ttl_ms(mut self, ttl_ms: i64) -> Self {
        self.ttl_ms = Some(ttl_ms);
        self
    }
}

impl ContractStoreResource {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(required), "`.")]
    pub fn required(mut self, required: bool) -> Self {
        self.required = Some(required);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(ttl_ms), "`.")]
    pub fn ttl_ms(mut self, ttl_ms: i64) -> Self {
        self.ttl_ms = Some(ttl_ms);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(max_object_bytes), "`.")]
    pub fn max_object_bytes(mut self, max_object_bytes: i64) -> Self {
        self.max_object_bytes = Some(max_object_bytes);
        self
    }

    #[doc = concat!("Constructs or updates contract data with `", stringify!(max_total_bytes), "`.")]
    pub fn max_total_bytes(mut self, max_total_bytes: i64) -> Self {
        self.max_total_bytes = Some(max_total_bytes);
        self
    }
}

impl ContractJobQueueResource {
    #[doc = concat!("Constructs or updates contract data with `", stringify!(docs_with_summary), "`.")]
    pub fn docs_with_summary(
        mut self,
        summary: impl Into<String>,
        markdown: impl Into<String>,
    ) -> Self {
        self.docs = Some(docs(summary, markdown));
        self
    }
}
