use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::time::Duration;

use async_nats::jetstream::{self, consumer, kv, object_store};
use serde_json::Value;
use sha2::{Digest, Sha256};
use trellis_protocol::{parse_api, parse_participant};

use super::authority::{validate_dependency_evidence, validate_resource_evidence};
use super::{
    AuthorityEvidenceRepository, AuthorityEvidenceScope, AuthorizationStateError,
    DependencyEvidence, DependencyState, ParticipantBindingRecord, ResourceBindingEvidence,
    ResourceBindingState, ResourceProviderIdentity,
};

pub(crate) async fn ensure_authority_dependencies<R>(
    repository: &R,
    scope: AuthorityEvidenceScope,
    binding: &ParticipantBindingRecord,
    now: i64,
) -> Result<(), AuthorizationStateError>
where
    R: AuthorityEvidenceRepository,
{
    let participant_value: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid(error.to_string()))?;
    let participant =
        parse_participant(&participant_value).map_err(|error| invalid(error.to_string()))?;
    let api_values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| invalid(error.to_string()))?;
    let apis = api_values
        .values()
        .map(|value| {
            parse_api(value)
                .map(|api| (api.id().to_owned(), api))
                .map_err(|error| invalid(error.to_string()))
        })
        .collect::<Result<BTreeMap<_, _>, _>>()?;
    let resolved = trellis_protocol::resolve_participant(&participant, &apis)
        .map_err(|error| invalid(error.to_string()))?;

    let mut providers = Vec::new();
    for evidence in repository.list_active_provider_evidence(now).await? {
        let authority = evidence.authority;
        let instance = evidence.instance;
        let provider_binding = evidence.binding;
        let provider_participant_value: Value =
            serde_json::from_str(&provider_binding.participant_json)
                .map_err(|error| invalid(error.to_string()))?;
        let provider_participant = parse_participant(&provider_participant_value)
            .map_err(|error| invalid(error.to_string()))?;
        let provider_api_values: BTreeMap<String, Value> =
            serde_json::from_str(&provider_binding.api_artifacts_json)
                .map_err(|error| invalid(error.to_string()))?;
        let provider_apis = provider_api_values
            .values()
            .map(|value| {
                parse_api(value)
                    .map(|api| (api.id().to_owned(), api))
                    .map_err(|error| invalid(error.to_string()))
            })
            .collect::<Result<BTreeMap<_, _>, _>>()?;
        let provider = trellis_protocol::resolve_participant(&provider_participant, &provider_apis)
            .map_err(|error| invalid(error.to_string()))?;
        providers.extend(provider.implemented_apis().iter().map(|implementation| {
            (
                implementation.provided().api().to_owned(),
                implementation.provided().api_digest().to_owned(),
                authority.participant_id.clone(),
                authority.deployment_id.clone(),
                instance.instance_id.clone(),
            )
        }));
    }
    providers.sort();
    let dependencies = resolved
        .required_apis()
        .iter()
        .map(|dependency| (dependency, true))
        .chain(
            resolved
                .optional_apis()
                .iter()
                .map(|dependency| (dependency, false)),
        )
        .filter_map(|(dependency, required)| {
            if super::builtins::is_platform_api(dependency.api(), dependency.api_digest()) {
                return None;
            }
            let provider = providers.iter().find(|provider| {
                provider.0 == dependency.api() && provider.1 == dependency.api_digest()
            });
            if required && provider.is_none() {
                tracing::warn!(
                    api_id = %dependency.api(),
                    api_digest = %dependency.api_digest(),
                    available_providers = ?providers,
                    "required authorization dependency has no active provider"
                );
            }
            provider.map(|provider| DependencyEvidence {
                alias: dependency.alias().to_owned(),
                required,
                api_id: dependency.api().to_owned(),
                api_digest: dependency.api_digest().to_owned(),
                provider_participant_id: provider.2.clone(),
                provider_deployment_id: Some(provider.3.clone()),
                provider_instance_id: Some(provider.4.clone()),
                state: DependencyState::Available,
                observed_at: now,
            })
        })
        .collect::<Vec<_>>();
    validate_dependency_evidence(&dependencies)?;
    repository
        .replace_dependency_evidence(scope, dependencies)
        .await
}

pub(crate) async fn ensure_deployment_resources<R>(
    client: &async_nats::Client,
    repository: &R,
    scope: AuthorityEvidenceScope,
    binding: &ParticipantBindingRecord,
    deployment_id: &str,
    now: i64,
) -> Result<(), AuthorizationStateError>
where
    R: AuthorityEvidenceRepository,
{
    let participant: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid(error.to_string()))?;
    parse_participant(&participant).map_err(|error| invalid(error.to_string()))?;
    let api_values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| invalid(error.to_string()))?;
    let jetstream = jetstream::new(client.clone());
    let mut evidence = Vec::new();

    add_state_evidence(&mut evidence, binding, deployment_id, &participant, now);

    for (kind, resources) in participant
        .get("resources")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|resources| resources.iter())
    {
        for (local_name, config) in resources.as_object().into_iter().flatten() {
            let token = resource_token(deployment_id, kind, local_name);
            let provider_identity = match kind.as_str() {
                "kv" => {
                    let bucket = format!("tr_kv_{token}");
                    if jetstream.get_key_value(&bucket).await.is_err() {
                        jetstream
                            .create_key_value(kv::Config {
                                bucket: bucket.clone(),
                                history: config.get("history").and_then(Value::as_u64).unwrap_or(1)
                                    as i64,
                                max_age: duration_ms(config, "ttlMs"),
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
                                max_age: duration_ms(config, "ttlMs"),
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
            evidence.push(resource_evidence(
                binding,
                deployment_id,
                kind,
                local_name,
                provider_identity,
                now,
            ));
        }
    }

    let namespace = format!(
        "tr_jobs_{}",
        resource_token(deployment_id, "jobs", "namespace")
    );
    if participant
        .get("jobQueues")
        .and_then(Value::as_object)
        .is_some_and(|queues| !queues.is_empty())
    {
        let keys_bucket = format!("JOBS_KEYS_{namespace}");
        if jetstream.get_key_value(&keys_bucket).await.is_err() {
            jetstream
                .create_key_value(kv::Config {
                    bucket: keys_bucket,
                    history: 1,
                    ..Default::default()
                })
                .await
                .map_err(|error| storage(error.to_string()))?;
        }
    }
    for (local_name, config) in participant
        .get("jobQueues")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|queues| queues.iter())
    {
        let token = resource_token(deployment_id, "jobQueue", local_name);
        let publish_prefix = format!("trellis.jobs.{namespace}.{token}");
        let work_subject = format!("trellis.work.{namespace}.{token}");
        let consumer_name = format!("{namespace}_{token}");
        let max_deliver = config
            .get("maxDeliver")
            .and_then(Value::as_i64)
            .unwrap_or(5);
        let mut backoff = config
            .get("backoffMs")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_u64)
                    .map(Duration::from_millis)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                [5_000, 30_000, 120_000, 600_000]
                    .into_iter()
                    .map(Duration::from_millis)
                    .collect()
            });
        backoff.truncate(max_deliver.saturating_sub(1) as usize);
        let stream = jetstream
            .get_stream("JOBS_WORK")
            .await
            .map_err(|error| storage(format!("JOBS_WORK is unavailable: {error}")))?;
        stream
            .get_or_create_consumer(
                &consumer_name,
                consumer::pull::Config {
                    durable_name: Some(consumer_name.clone()),
                    filter_subject: work_subject.clone(),
                    ack_policy: consumer::AckPolicy::Explicit,
                    ack_wait: backoff.first().copied().unwrap_or_else(|| {
                        Duration::from_millis(
                            config
                                .get("ackWaitMs")
                                .and_then(Value::as_u64)
                                .unwrap_or(300_000),
                        )
                    }),
                    max_deliver,
                    backoff,
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| storage(error.to_string()))?;
        evidence.push(resource_evidence(
            binding,
            deployment_id,
            "jobQueue",
            local_name,
            ResourceProviderIdentity::JobQueue {
                namespace: namespace.clone(),
                work_stream: "JOBS_WORK".to_owned(),
                publish_prefix,
                updates_prefix: config
                    .get("update")
                    .filter(|value| !value.is_null())
                    .map(|_| format!("trellis.job_updates.{namespace}.{token}")),
                work_subject,
                consumer: consumer_name,
            },
            now,
        ));
    }

    let aliases = participant_api_aliases(&participant)?;
    for (local_name, config) in participant
        .get("eventConsumers")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|consumers| consumers.iter())
    {
        let mut filters = BTreeSet::new();
        for (alias, names) in config
            .get("events")
            .and_then(Value::as_object)
            .into_iter()
            .flat_map(|events| events.iter())
        {
            let api_id = aliases.get(alias).ok_or_else(|| {
                invalid(format!(
                    "event consumer references unknown API alias {alias}"
                ))
            })?;
            let api =
                parse_api(api_values.get(api_id).ok_or_else(|| {
                    invalid(format!("event consumer API {api_id} is unavailable"))
                })?)
                .map_err(|error| invalid(error.to_string()))?;
            let subjects = api
                .derived_subjects()
                .map_err(|error| invalid(error.to_string()))?;
            for name in names.as_array().into_iter().flatten() {
                let name = name
                    .as_str()
                    .ok_or_else(|| invalid("event consumer name must be text"))?;
                filters.insert(
                    subjects
                        .events
                        .get(name)
                        .ok_or_else(|| invalid(format!("event {name} is unavailable")))?
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
        let mut backoff = config
            .get("backoffMs")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_u64)
                    .map(Duration::from_millis)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| {
                [5_000, 30_000, 120_000, 600_000, 1_800_000]
                    .into_iter()
                    .map(Duration::from_millis)
                    .collect()
            });
        backoff.truncate(max_deliver.saturating_sub(1) as usize);
        let consumer_name = format!(
            "tr_cons_{}",
            resource_token(deployment_id, "eventConsumer", local_name)
        );
        let stream = jetstream
            .get_stream("trellis")
            .await
            .map_err(|error| storage(format!("event stream is unavailable: {error}")))?;
        stream
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
                    ack_wait: Duration::from_millis(
                        config
                            .get("ackWaitMs")
                            .and_then(Value::as_u64)
                            .unwrap_or(300_000),
                    ),
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
                        ("trellis.managed_by".to_owned(), "authority".to_owned()),
                        ("trellis.group".to_owned(), local_name.clone()),
                        ("trellis.deployment_id".to_owned(), deployment_id.to_owned()),
                    ]),
                    ..Default::default()
                },
            )
            .await
            .map_err(|error| storage(error.to_string()))?;
        evidence.push(resource_evidence(
            binding,
            deployment_id,
            "eventConsumer",
            local_name,
            ResourceProviderIdentity::EventConsumer {
                stream: "trellis".to_owned(),
                consumer: consumer_name,
                filter_subjects: filters,
            },
            now,
        ));
    }

    validate_resource_evidence(&evidence)?;
    repository.replace_resource_evidence(scope, evidence).await
}

pub(crate) async fn ensure_identity_resources<R>(
    repository: &R,
    scope: AuthorityEvidenceScope,
    binding: &ParticipantBindingRecord,
    principal_id: &str,
    now: i64,
) -> Result<(), AuthorizationStateError>
where
    R: AuthorityEvidenceRepository,
{
    let participant: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid(error.to_string()))?;
    parse_participant(&participant).map_err(|error| invalid(error.to_string()))?;
    let mut evidence = Vec::new();
    add_state_evidence(&mut evidence, binding, principal_id, &participant, now);
    validate_resource_evidence(&evidence)?;
    repository.replace_resource_evidence(scope, evidence).await
}

fn add_state_evidence(
    evidence: &mut Vec<ResourceBindingEvidence>,
    binding: &ParticipantBindingRecord,
    subject_id: &str,
    participant: &Value,
    now: i64,
) {
    for local_name in participant
        .get("state")
        .and_then(Value::as_object)
        .into_iter()
        .flat_map(|state| state.keys())
    {
        evidence.push(resource_evidence(
            binding,
            subject_id,
            "state",
            local_name,
            ResourceProviderIdentity::State {
                bucket: "trellis_state".to_owned(),
            },
            now,
        ));
    }
}

fn participant_api_aliases(
    participant: &Value,
) -> Result<BTreeMap<String, String>, AuthorizationStateError> {
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
            let api = value
                .get("api")
                .and_then(Value::as_str)
                .ok_or_else(|| invalid(format!("participant API alias {alias} is invalid")))?;
            aliases.insert(alias.clone(), api.to_owned());
        }
    }
    Ok(aliases)
}

fn resource_evidence(
    binding: &ParticipantBindingRecord,
    deployment_id: &str,
    kind: &str,
    local_name: &str,
    provider_identity: ResourceProviderIdentity,
    now: i64,
) -> ResourceBindingEvidence {
    ResourceBindingEvidence {
        resource_kind: kind.to_owned(),
        local_name: local_name.to_owned(),
        binding_id: format!("binding:{deployment_id}:{kind}:{local_name}"),
        owner_participant_id: binding.participant_id.clone(),
        provider_identity,
        state: ResourceBindingState::Available,
        materialized_at: now,
        error: None,
    }
}

fn resource_token(deployment_id: &str, kind: &str, local_name: &str) -> String {
    Sha256::digest(format!("{deployment_id}\0{kind}\0{local_name}").as_bytes())[..10]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

fn duration_ms(config: &Value, name: &str) -> Duration {
    Duration::from_millis(config.get(name).and_then(Value::as_u64).unwrap_or_default())
}

fn invalid(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}

fn storage(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::Storage(message.into())
}
