use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use async_nats::jetstream::{self, consumer, kv, object_store};
use serde_json::Value;
use sha2::{Digest, Sha256};
use trellis_protocol::{parse_api, parse_participant};

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
    let participant: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid(error.to_string()))?;
    parse_participant(&participant).map_err(|error| invalid(error.to_string()))?;
    let apis: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| invalid(error.to_string()))?;
    let jetstream = jetstream::new(client.clone());
    let mut evidence = Vec::new();

    for name in participant
        .get("state")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|state| state.keys())
    {
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
    for (kind, resources) in participant
        .get("resources")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|resources| resources.iter())
    {
        for (name, config) in resources.as_object().into_iter().flatten() {
            let token = token(deployment_id, kind, name);
            let provider = match kind.as_str() {
                "kv" => {
                    let bucket = format!("tr_kv_{token}");
                    if jetstream.get_key_value(&bucket).await.is_err() {
                        jetstream
                            .create_key_value(kv::Config {
                                bucket: bucket.clone(),
                                history: config.get("history").and_then(Value::as_u64).unwrap_or(1)
                                    as i64,
                                max_age: duration(config, "ttlMs"),
                                max_value_size: config
                                    .get("maxValueBytes")
                                    .and_then(Value::as_i64)
                                    .unwrap_or(-1)
                                    as i32,
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
                                max_age: duration(config, "ttlMs"),
                                max_bytes: config
                                    .get("maxTotalBytes")
                                    .and_then(Value::as_i64)
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
    }

    let namespace = format!("tr_jobs_{}", token(deployment_id, "jobs", "namespace"));
    if participant
        .get("jobQueues")
        .and_then(Value::as_object)
        .is_some_and(|queues| !queues.is_empty())
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
        .get("jobQueues")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|queues| queues.iter())
    {
        let item = token(deployment_id, "jobQueue", name);
        let work_subject = format!("trellis.work.{namespace}.{item}");
        let consumer_name = format!("{namespace}_{item}");
        let max_deliver = config
            .get("maxDeliver")
            .and_then(Value::as_i64)
            .unwrap_or(5);
        let mut backoff = backoff(config, &[5_000, 30_000, 120_000, 600_000]);
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
                        .unwrap_or_else(|| duration_default(config, "ackWaitMs", 300_000)),
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
                    .get("update")
                    .filter(|value| !value.is_null())
                    .map(|_| format!("trellis.job_updates.{namespace}.{item}")),
                work_subject,
                consumer: consumer_name,
            },
            now,
        ));
    }

    let aliases = aliases(&participant)?;
    for (name, config) in participant
        .get("eventConsumers")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|values| values.iter())
    {
        let mut filters = BTreeSet::new();
        for (alias, names) in config
            .get("events")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|values| values.iter())
        {
            let api_id = aliases.get(alias).ok_or_else(|| {
                invalid(format!(
                    "event consumer references unknown API alias {alias}"
                ))
            })?;
            let subjects =
                parse_api(apis.get(api_id).ok_or_else(|| {
                    invalid(format!("event consumer API {api_id} is unavailable"))
                })?)
                .map_err(|error| invalid(error.to_string()))?
                .derived_subjects()
                .map_err(|error| invalid(error.to_string()))?;
            for event in names.as_array().into_iter().flatten() {
                let event = event
                    .as_str()
                    .ok_or_else(|| invalid("event consumer name must be text"))?;
                filters.insert(
                    subjects
                        .events
                        .get(event)
                        .ok_or_else(|| invalid(format!("event {event} is unavailable")))?
                        .wildcard
                        .clone(),
                );
            }
        }
        let filters = filters.into_iter().collect::<Vec<_>>();
        let max_deliver = config
            .get("maxDeliver")
            .and_then(Value::as_i64)
            .unwrap_or(6);
        let mut backoff = backoff(config, &[5_000, 30_000, 120_000, 600_000, 1_800_000]);
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
                    deliver_policy: if config.get("replay").and_then(Value::as_str) == Some("all") {
                        consumer::DeliverPolicy::All
                    } else {
                        consumer::DeliverPolicy::New
                    },
                    ack_policy: consumer::AckPolicy::Explicit,
                    ack_wait: duration_default(config, "ackWaitMs", 300_000),
                    max_deliver,
                    backoff,
                    max_ack_pending: if config.get("ordering").and_then(Value::as_str)
                        == Some("strict")
                    {
                        1
                    } else {
                        1_000
                    },
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
    let participant: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid(error.to_string()))?;
    parse_participant(&participant).map_err(|error| invalid(error.to_string()))?;
    Ok(participant
        .get("state")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|state| state.keys())
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

fn aliases(participant: &Value) -> Result<BTreeMap<String, String>, AuthorizationStateError> {
    let mut aliases = BTreeMap::new();
    for section in ["implements", "required", "optional"] {
        let values = if section == "implements" {
            participant.get(section)
        } else {
            participant.get("uses").and_then(|uses| uses.get(section))
        };
        for (alias, value) in values
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|values| values.iter())
        {
            aliases.insert(
                alias.clone(),
                value
                    .get("api")
                    .and_then(Value::as_str)
                    .ok_or_else(|| invalid(format!("participant API alias {alias} is invalid")))?
                    .to_owned(),
            );
        }
    }
    Ok(aliases)
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
fn duration(value: &Value, field: &str) -> Duration {
    Duration::from_millis(value.get(field).and_then(Value::as_u64).unwrap_or_default())
}
fn duration_default(value: &Value, field: &str, default: u64) -> Duration {
    Duration::from_millis(value.get(field).and_then(Value::as_u64).unwrap_or(default))
}
fn backoff(value: &Value, default: &[u64]) -> Vec<Duration> {
    value
        .get("backoffMs")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_u64)
                .map(Duration::from_millis)
                .collect()
        })
        .unwrap_or_else(|| default.iter().copied().map(Duration::from_millis).collect())
}
fn invalid(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}
fn storage(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::Storage(message.into())
}
