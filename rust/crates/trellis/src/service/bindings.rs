use std::collections::BTreeMap;

use super::ServerError;

/// Contract identifier and digest pair used for bootstrap checks.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(BootstrapContractRef), "`.")]
pub struct BootstrapContractRef {
    #[doc = concat!("The `", stringify!(id), "` value.")]
    pub id: String,
    #[doc = concat!("The `", stringify!(digest), "` value.")]
    pub digest: String,
}

/// Resolved active binding for one service session.
#[derive(Debug, Clone, PartialEq, Eq)]
#[doc = concat!("Public Trellis data type `", stringify!(BootstrapBinding), "`.")]
pub struct BootstrapBinding {
    #[doc = concat!("The `", stringify!(contract_id), "` value.")]
    pub contract_id: String,
    #[doc = concat!("The `", stringify!(digest), "` value.")]
    pub digest: String,
}

/// Typed service resource bindings resolved from Trellis core bootstrap data.
#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(ServiceResourceBindings), "`.")]
pub struct ServiceResourceBindings {
    /// KV/state resources keyed by contract-local resource name.
    #[doc = concat!("The `", stringify!(kv), "` value.")]
    pub kv: BTreeMap<String, KvResourceBinding>,
    /// Object-store resources keyed by contract-local resource name.
    #[doc = concat!("The `", stringify!(store), "` value.")]
    pub store: BTreeMap<String, StoreResourceBinding>,
    /// Service-private jobs resource, when declared by the contract.
    #[doc = concat!("The `", stringify!(jobs), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub jobs: Option<JobsResourceBinding>,
    /// Durable event consumer groups keyed by contract-local group name.
    #[doc = concat!("The `", stringify!(event_consumers), "` value.")]
    pub event_consumers: BTreeMap<String, EventConsumerResourceBinding>,
}

/// Bound durable event consumer group.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(EventConsumerResourceBinding), "`.")]
pub struct EventConsumerResourceBinding {
    /// JetStream stream that owns the durable consumer.
    #[doc = concat!("The `", stringify!(stream), "` value.")]
    pub stream: String,
    /// Pre-provisioned durable consumer name.
    #[doc = concat!("The `", stringify!(consumer_name), "` value.")]
    pub consumer_name: String,
    /// Concrete event subjects filtered by the consumer.
    #[doc = concat!("The `", stringify!(filter_subjects), "` value.")]
    pub filter_subjects: Vec<String>,
    /// Replay policy used when the consumer was provisioned.
    #[doc = concat!("The `", stringify!(replay), "` value.")]
    pub replay: EventConsumerReplay,
    /// Ordering policy used by the consumer group.
    #[doc = concat!("The `", stringify!(ordering), "` value.")]
    pub ordering: EventConsumerOrdering,
    /// Ack wait in milliseconds for the durable consumer.
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` value.")]
    pub ack_wait_ms: i64,
    /// Maximum delivery attempts before termination.
    #[doc = concat!("The `", stringify!(max_deliver), "` value.")]
    pub max_deliver: i64,
    /// Redelivery backoff schedule in milliseconds.
    #[doc = concat!("The `", stringify!(backoff_ms), "` value.")]
    pub backoff_ms: Vec<i64>,
}

/// Replay policy attached to an event consumer binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[doc = concat!("Public Trellis value set `", stringify!(EventConsumerReplay), "`.")]
pub enum EventConsumerReplay {
    /// Deliver only events published after consumer creation.
    New,
    /// Replay all retained events before live delivery.
    All,
    /// Preserve an unrecognized future wire value.
    Unknown,
}

/// Ordering policy attached to an event consumer binding.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
#[doc = concat!("Public Trellis value set `", stringify!(EventConsumerOrdering), "`.")]
pub enum EventConsumerOrdering {
    /// Process one event at a time in stream order.
    Strict,
    /// Allow concurrent event processing.
    Parallel,
    /// Preserve an unrecognized future wire value.
    Unknown,
}

/// Bound KV/state bucket resource.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(KvResourceBinding), "`.")]
pub struct KvResourceBinding {
    /// Concrete KV bucket name provisioned for this service binding.
    #[doc = concat!("The `", stringify!(bucket), "` value.")]
    pub bucket: String,
    /// Number of historical values retained by the bucket.
    #[doc = concat!("The `", stringify!(history), "` value.")]
    pub history: i64,
    /// Maximum encoded value size in bytes, when configured.
    #[doc = concat!("The `", stringify!(max_value_bytes), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_value_bytes: Option<i64>,
    /// Bucket TTL in milliseconds.
    #[doc = concat!("The `", stringify!(ttl_ms), "` value.")]
    pub ttl_ms: i64,
}

/// Bound object-store resource.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(StoreResourceBinding), "`.")]
pub struct StoreResourceBinding {
    /// Concrete object-store bucket name provisioned for this service binding.
    #[doc = concat!("The `", stringify!(name), "` value.")]
    pub name: String,
    /// Maximum object size in bytes, when configured.
    #[doc = concat!("The `", stringify!(max_object_bytes), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_object_bytes: Option<i64>,
    /// Maximum total store size in bytes, when configured.
    #[doc = concat!("The `", stringify!(max_total_bytes), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_total_bytes: Option<i64>,
    /// Store TTL in milliseconds.
    #[doc = concat!("The `", stringify!(ttl_ms), "` value.")]
    pub ttl_ms: i64,
}

/// Bound service-private jobs resource.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(JobsResourceBinding), "`.")]
pub struct JobsResourceBinding {
    /// Logical registered service name projected in job admin views.
    #[doc = concat!("The `", stringify!(service_name), "` value.")]
    pub service_name: String,
    /// Service-local jobs namespace used in job subjects and stream names.
    #[doc = concat!("The `", stringify!(namespace), "` value.")]
    pub namespace: String,
    /// Work stream used by private job workers, when provisioned.
    #[doc = concat!("The `", stringify!(work_stream), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub work_stream: Option<String>,
    /// Job queues keyed by contract-local queue type.
    #[doc = concat!("The `", stringify!(queues), "` value.")]
    pub queues: BTreeMap<String, JobsQueueResourceBinding>,
}

/// Bound service-private jobs queue resource.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(JobsQueueResourceBinding), "`.")]
pub struct JobsQueueResourceBinding {
    /// Logical queue type from the contract binding.
    #[doc = concat!("The `", stringify!(queue_type), "` value.")]
    pub queue_type: String,
    /// Publish prefix for job lifecycle events.
    #[doc = concat!("The `", stringify!(publish_prefix), "` value.")]
    pub publish_prefix: String,
    /// Publish prefix for live-only job updates, when declared.
    #[doc = concat!("The `", stringify!(updates_prefix), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updates_prefix: Option<String>,
    /// NATS subject consumed by workers for this queue.
    #[doc = concat!("The `", stringify!(work_subject), "` value.")]
    pub work_subject: String,
    /// Durable consumer name for this queue.
    #[doc = concat!("The `", stringify!(consumer_name), "` value.")]
    pub consumer_name: String,
    /// JSON schema reference for queued job payloads.
    #[doc = concat!("The `", stringify!(payload), "` value.")]
    pub payload: JobsSchemaRef,
    /// Optional JSON schema reference for live-only updates.
    #[doc = concat!("The `", stringify!(update), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub update: Option<JobsSchemaRef>,
    /// Optional JSON schema reference for successful job results.
    #[doc = concat!("The `", stringify!(result), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<JobsSchemaRef>,
    /// Maximum delivery attempts before dead-letter handling.
    #[doc = concat!("The `", stringify!(max_deliver), "` value.")]
    pub max_deliver: i64,
    /// Redelivery backoff schedule in milliseconds.
    #[doc = concat!("The `", stringify!(backoff_ms), "` value.")]
    pub backoff_ms: Vec<i64>,
    /// Ack wait in milliseconds for the durable consumer.
    #[doc = concat!("The `", stringify!(ack_wait_ms), "` value.")]
    pub ack_wait_ms: i64,
    /// Optional business deadline applied to newly created jobs.
    #[doc = concat!("The `", stringify!(default_deadline_ms), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_deadline_ms: Option<i64>,
    /// Whether progress events are enabled for this queue.
    #[doc = concat!("The `", stringify!(progress), "` value.")]
    pub progress: bool,
    /// Whether log events are enabled for this queue.
    #[doc = concat!("The `", stringify!(logs), "` value.")]
    pub logs: bool,
    /// Whether dead-letter handling is enabled for this queue.
    #[doc = concat!("The `", stringify!(dlq), "` value.")]
    pub dlq: bool,
    /// Optional normalized keyed concurrency policy for this queue.
    #[doc = concat!("The `", stringify!(key_concurrency), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_concurrency: Option<crate::jobs::bindings::JobKeyConcurrencyBinding>,
    /// Optional normalized queue-depth policy for keyed queues.
    #[doc = concat!("The `", stringify!(queue), "` value.")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub queue: Option<crate::jobs::bindings::JobQueueDepthBinding>,
}

/// Schema reference attached to a jobs queue binding.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
#[doc = concat!("Public Trellis data type `", stringify!(JobsSchemaRef), "`.")]
pub struct JobsSchemaRef {
    #[doc = concat!("The `", stringify!(schema), "` value.")]
    pub schema: String,
}

/// Validate that the expected contract is active and bindings match it.
#[doc = concat!("Trellis API operation `", stringify!(validate_bootstrap_contract_state), "`.")]
pub fn validate_bootstrap_contract_state(
    service_name: &str,
    expected: &BootstrapContractRef,
    catalog_contracts: &[BootstrapContractRef],
    binding: Option<&BootstrapBinding>,
) -> Result<BootstrapBinding, ServerError> {
    let is_active = catalog_contracts
        .iter()
        .any(|contract| contract.id == expected.id && contract.digest == expected.digest);

    if !is_active {
        return Err(ServerError::BootstrapInactiveContract {
            service_name: service_name.to_string(),
            contract_id: expected.id.clone(),
            contract_digest: expected.digest.clone(),
        });
    }

    let binding = binding.ok_or_else(|| ServerError::BootstrapMissingBinding {
        service_name: service_name.to_string(),
        contract_id: expected.id.clone(),
        contract_digest: expected.digest.clone(),
    })?;

    if binding.contract_id != expected.id || binding.digest != expected.digest {
        return Err(ServerError::BootstrapBindingMismatch {
            service_name: (service_name.to_string()).into_boxed_str(),
            expected_contract_id: expected.id.clone(),
            expected_contract_digest: expected.digest.clone(),
            actual_contract_id: binding.contract_id.clone(),
            actual_contract_digest: binding.digest.clone(),
        });
    }

    Ok(binding.clone())
}
