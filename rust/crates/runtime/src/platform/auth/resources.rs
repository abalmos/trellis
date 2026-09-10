use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use async_nats::jetstream::{self, consumer, kv, object_store};
use sha2::{Digest, Sha256};
use trellis_protocol::ParticipantResourceKind;

use super::{
    AuthorizationStateError, ParticipantBindingRecord, ResourceBindingEvidence,
    ResourceBindingState, ResourceProviderIdentity,
};

pub(crate) async fn provision_deployment_resources(
    client: &async_nats::Client,
    binding: &ParticipantBindingRecord,
    deployment_id: &str,
    now: i64,
) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
    let participant = binding.resolve()?;
    let jetstream = jetstream::new(client.clone());
    let mut evidence = Vec::new();

    for name in participant.resources.iter().filter_map(|(name, resource)| {
        (resource.kind == ParticipantResourceKind::State).then_some(name)
    }) {
        evidence.push(resource(
            binding,
            deployment_id,
            "state",
            name,
            ResourceProviderIdentity::State {
                bucket: "trellis_state".to_owned(),
            },
            now,
        ));
    }
    for (name, config) in &participant.resources {
        let kind = match config.kind {
            ParticipantResourceKind::Kv => "kv",
            ParticipantResourceKind::Store => "store",
            _ => continue,
        };
        let token = token(deployment_id, kind, name);
        let provider = match kind {
            "kv" => {
                let bucket = format!("tr_kv_{token}");
                if jetstream.get_key_value(&bucket).await.is_err() {
                    jetstream
                        .create_key_value(kv::Config {
                            bucket: bucket.clone(),
                            history: i64::try_from(config.history.unwrap_or(1))
                                .map_err(|error| invalid(error.to_string()))?,
                            max_age: Duration::from_millis(config.ttl_ms.unwrap_or_default()),
                            max_value_size: config
                                .desired_max_value
                                .and_then(|value| i32::try_from(value).ok())
                                .unwrap_or(-1),
                            ..Default::default()
                        })
                        .await
                        .map_err(|error| storage(error.to_string()))?;
                }
                ResourceProviderIdentity::Kv { bucket }
            }
            "store" => {
                let bucket = format!("tr_obj_{token}");
                if jetstream.get_object_store(&bucket).await.is_err() {
                    jetstream
                        .create_object_store(object_store::Config {
                            bucket: bucket.clone(),
                            max_age: Duration::from_millis(config.ttl_ms.unwrap_or_default()),
                            max_bytes: config
                                .desired_max_total
                                .and_then(|value| i64::try_from(value).ok())
                                .unwrap_or(-1),
                            ..Default::default()
                        })
                        .await
                        .map_err(|error| storage(error.to_string()))?;
                }
                ResourceProviderIdentity::Store { bucket }
            }
            "state" => ResourceProviderIdentity::State {
                bucket: "trellis_state".to_owned(),
            },
            _ => continue,
        };
        evidence.push(resource(binding, deployment_id, kind, name, provider, now));
    }

    let namespace = format!("tr_jobs_{}", token(deployment_id, "jobs", "namespace"));
    if participant
        .resources
        .values()
        .any(|resource| resource.kind == ParticipantResourceKind::JobQueue)
    {
        let bucket = format!("JOBS_KEYS_{namespace}");
        if jetstream.get_key_value(&bucket).await.is_err() {
            jetstream
                .create_key_value(kv::Config {
                    bucket,
                    history: 1,
                    ..Default::default()
                })
                .await
                .map_err(|error| storage(error.to_string()))?;
        }
    }
    for (name, config) in participant
        .resources
        .iter()
        .filter(|(_, resource)| resource.kind == ParticipantResourceKind::JobQueue)
    {
        let item = token(deployment_id, "jobQueue", name);
        let work_subject = format!("trellis.work.{namespace}.{item}");
        let consumer_name = format!("{namespace}_{item}");
        let max_deliver = i64::from(config.retry_attempts.unwrap_or(5));
        let mut backoff = if config.retry_backoff_ms.is_empty() {
            [5_000, 30_000, 120_000, 600_000]
                .into_iter()
                .map(Duration::from_millis)
                .collect()
        } else {
            config
                .retry_backoff_ms
                .iter()
                .copied()
                .map(Duration::from_millis)
                .collect()
        };
        backoff.truncate(max_deliver.saturating_sub(1) as usize);
        jetstream
            .get_stream("JOBS_WORK")
            .await
            .map_err(|error| storage(format!("JOBS_WORK is unavailable: {error}")))?
            .get_or_create_consumer(
                &consumer_name,
                consumer::pull::Config {
                    durable_name: Some(consumer_name.clone()),
                    filter_subject: work_subject.clone(),
                    ack_policy: consumer::AckPolicy::Explicit,
                    ack_wait: backoff
                        .first()
                        .copied()
                        .unwrap_or(Duration::from_millis(300_000)),
                    max_deliver,
                    backoff,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| storage(error.to_string()))?;
        evidence.push(resource(
            binding,
            deployment_id,
            "jobQueue",
            name,
            ResourceProviderIdentity::JobQueue {
                namespace: namespace.clone(),
                work_stream: "JOBS_WORK".to_owned(),
                publish_prefix: format!("trellis.jobs.{namespace}.{item}"),
                updates_prefix: config
                    .update_schema
                    .as_ref()
                    .map(|_| format!("trellis.job_updates.{namespace}.{item}")),
                work_subject,
                consumer: consumer_name,
            },
            now,
        ));
    }

    for (name, config) in participant
        .resources
        .iter()
        .filter(|(_, resource)| resource.kind == ParticipantResourceKind::EventConsumer)
    {
        let mut filters = BTreeSet::new();
        for (api_id, names) in &config.consumer_events {
            let api = participant
                .referenced_apis
                .get(api_id)
                .ok_or_else(|| invalid(format!("event consumer API {api_id} is unavailable")))?;
            for event in names {
                let action = api
                    .actions
                    .get(&format!("event:{event}"))
                    .ok_or_else(|| invalid(format!("event {event} is unavailable")))?;
                filters.insert(format!(
                    "events.v{}.{}{}",
                    api.major,
                    event,
                    ".*".repeat(action.event_parameter_count)
                ));
            }
        }
        let filters = filters.into_iter().collect::<Vec<_>>();
        let max_deliver = i64::from(config.retry_attempts.unwrap_or(6));
        let mut backoff = if config.retry_backoff_ms.is_empty() {
            [5_000, 30_000, 120_000, 600_000, 1_800_000]
                .into_iter()
                .map(Duration::from_millis)
                .collect()
        } else {
            config
                .retry_backoff_ms
                .iter()
                .copied()
                .map(Duration::from_millis)
                .collect()
        };
        backoff.truncate(max_deliver.saturating_sub(1) as usize);
        let consumer_name = format!("tr_cons_{}", token(deployment_id, "eventConsumer", name));
        jetstream
            .get_stream("trellis")
            .await
            .map_err(|error| storage(format!("event stream is unavailable: {error}")))?
            .get_or_create_consumer(
                &consumer_name,
                consumer::pull::Config {
                    durable_name: Some(consumer_name.clone()),
                    filter_subjects: filters.clone(),
                    deliver_policy: if config.consumer_replay_all {
                        consumer::DeliverPolicy::All
                    } else {
                        consumer::DeliverPolicy::New
                    },
                    ack_policy: consumer::AckPolicy::Explicit,
                    ack_wait: duration_default(config, "ackWaitMs", 300_000),
                    max_deliver,
                    backoff,
                    max_ack_pending: i64::from(config.consumer_concurrency.unwrap_or(1).max(1_000))
                        as i32,
                    metadata: HashMap::from([
                        ("trellis.managed_by".to_owned(), "trellis".to_owned()),
                        ("trellis.group".to_owned(), name.clone()),
                        ("trellis.deployment_id".to_owned(), deployment_id.to_owned()),
                    ]),
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| storage(error.to_string()))?;
        evidence.push(resource(
            binding,
            deployment_id,
            "eventConsumer",
            name,
            ResourceProviderIdentity::EventConsumer {
                stream: "trellis".to_owned(),
                consumer: consumer_name,
                filter_subjects: filters,
            },
            now,
        ));
    }
    Ok(evidence)
}

pub(crate) fn identity_resources(
    binding: &ParticipantBindingRecord,
    principal_id: &str,
    now: i64,
) -> Result<Vec<ResourceBindingEvidence>, AuthorizationStateError> {
    Ok(binding
        .resolve()?
        .resources
        .iter()
        .filter_map(|(name, resource)| {
            (resource.kind == ParticipantResourceKind::State).then_some(name)
        })
        .map(|name| {
            resource(
                binding,
                principal_id,
                "state",
                name,
                ResourceProviderIdentity::State {
                    bucket: "trellis_state".to_owned(),
                },
                now,
            )
        })
        .collect())
}

fn resource(
    binding: &ParticipantBindingRecord,
    owner: &str,
    kind: &str,
    name: &str,
    provider_identity: ResourceProviderIdentity,
    now: i64,
) -> ResourceBindingEvidence {
    ResourceBindingEvidence {
        resource_kind: kind.to_owned(),
        local_name: name.to_owned(),
        binding_id: format!("binding:{owner}:{kind}:{name}"),
        owner_participant_id: binding.participant_id.clone(),
        provider_identity,
        state: ResourceBindingState::Available,
        materialized_at: now,
        error: None,
    }
}

fn token(owner: &str, kind: &str, name: &str) -> String {
    Sha256::digest(format!("{owner}\0{kind}\0{name}").as_bytes())[..10]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}
fn invalid(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}
fn storage(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::Storage(message.into())
}
