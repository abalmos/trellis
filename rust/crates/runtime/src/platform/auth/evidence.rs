use std::collections::{BTreeMap, BTreeSet};

use serde::{Deserialize, Serialize};
use trellis_idl::{
    api_digest, compile_evidence, participant_digest, selected_permission_atoms, ActionDefinition,
    ActionKind, PackageEvidence, ResourceDefinition,
};
use trellis_protocol::{
    GrantSet, ParticipantKind, ParticipantResourceKind, PermissionAction, PermissionAtom,
};

use super::AuthorizationStateError;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct PackageEvidenceInput {
    pub package_evidence: PackageEvidence,
    pub participant_path: String,
    pub package_digest: String,
}

impl PackageEvidenceInput {
    #[cfg(test)]
    pub(crate) fn from_generated_descriptor(
        package_evidence: trellis_rs::generated::PackageEvidence,
        participant_path: &str,
    ) -> Result<Self, AuthorizationStateError> {
        package_evidence
            .validate()
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
        Ok(Self {
            package_evidence: PackageEvidence {
                root_package: package_evidence.root_package().to_owned(),
                root_digest: package_evidence.root_digest().to_owned(),
                packages: package_evidence
                    .packages()
                    .iter()
                    .map(|package| {
                        Ok(trellis_idl::PackageSourceEvidence {
                            name: package.name().to_owned(),
                            version: package.version().parse().map_err(|error| {
                                AuthorizationStateError::InvalidRecord(format!(
                                    "invalid generated package version: {error}"
                                ))
                            })?,
                            digest: package.digest().to_owned(),
                            source: package.source().to_owned(),
                        })
                    })
                    .collect::<Result<_, AuthorizationStateError>>()?,
            },
            participant_path: participant_path.to_owned(),
            package_digest: package_evidence.root_digest().to_owned(),
        })
    }

    pub(crate) fn from_generated_wire(
        package_evidence: trellis_runtime_apis::types::AuthPackageEvidence,
        participant_path: String,
        package_digest: String,
    ) -> Result<Self, AuthorizationStateError> {
        Ok(Self {
            package_evidence: PackageEvidence {
                root_package: package_evidence.root_package,
                root_digest: package_evidence.root_digest,
                packages: package_evidence
                    .packages
                    .into_iter()
                    .map(|package| {
                        Ok(trellis_idl::PackageSourceEvidence {
                            name: package.name,
                            version: package.version.parse().map_err(|error| {
                                AuthorizationStateError::InvalidRecord(format!(
                                    "invalid package evidence version: {error}"
                                ))
                            })?,
                            digest: package.digest,
                            source: package.source,
                        })
                    })
                    .collect::<Result<_, AuthorizationStateError>>()?,
            },
            participant_path,
            package_digest,
        })
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ParticipantRuntimeProjection {
    pub participant_id: String,
    pub participant_kind: ParticipantKind,
    pub display_name: String,
    pub implemented_apis: BTreeMap<String, ApiRuntimeProjection>,
    pub referenced_apis: BTreeMap<String, ApiRuntimeProjection>,
    pub resources: BTreeMap<String, ResourceRuntimeProjection>,
    pub required_grants: GrantSet,
    pub optional_grant_bundles: BTreeMap<String, GrantSet>,
    pub required_capabilities: Vec<String>,
    pub optional_capability_definitions: BTreeMap<String, GrantSet>,
}

impl ParticipantRuntimeProjection {
    pub(crate) fn select_grants(
        &self,
        optional_capabilities: &[String],
    ) -> Result<GrantSet, AuthorizationStateError> {
        let mut permissions = self.required_grants.permissions().to_vec();
        for capability in optional_capabilities {
            let grant = self.optional_grant_bundles.get(capability).ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(format!(
                    "unknown optional capability {capability}"
                ))
            })?;
            permissions.extend_from_slice(grant.permissions());
        }
        Ok(GrantSet::new(permissions))
    }
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ApiRuntimeProjection {
    pub digest: String,
    pub major: u32,
    pub actions: BTreeMap<String, ActionRuntimeProjection>,
    pub capabilities: BTreeMap<String, CapabilityRuntimeProjection>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct CapabilityRuntimeProjection {
    pub display_name: String,
    pub description: String,
    pub allows: Vec<PermissionAtom>,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ActionRuntimeProjection {
    pub kind: RuntimeActionKind,
    pub upload: bool,
    pub download: bool,
    pub event_parameter_count: usize,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) enum RuntimeActionKind {
    Rpc,
    Operation,
    Event,
    Feed,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct ResourceRuntimeProjection {
    pub kind: ParticipantResourceKind,
    pub optional: bool,
    pub history: Option<u64>,
    pub ttl_ms: Option<u64>,
    pub desired_max_value: Option<u64>,
    pub desired_max_object: Option<u64>,
    pub desired_max_total: Option<u64>,
    pub deadline_ms: Option<u64>,
    pub payload_schema: Option<String>,
    pub result_schema: Option<String>,
    pub update_schema: Option<String>,
    pub retry_attempts: Option<u32>,
    pub retry_backoff_ms: Vec<u64>,
    pub consumer_events: BTreeMap<String, Vec<String>>,
    pub consumer_concurrency: Option<u32>,
    pub consumer_replay_all: bool,
}

pub(crate) fn verify_package_evidence(
    input: &PackageEvidenceInput,
) -> Result<(String, String, ParticipantRuntimeProjection, String), AuthorizationStateError> {
    if input.package_digest != input.package_evidence.root_digest {
        return invalid("packageDigest does not match packageEvidence.rootDigest");
    }
    let evidence_value = serde_json::to_value(&input.package_evidence)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let evidence_json = trellis_protocol::canonicalize_json(&evidence_value)
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    let graph = compile_evidence(input.package_evidence.clone())
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    if graph.root_digest() != input.package_digest {
        return invalid("packageDigest does not match recompiled package semantics");
    }
    let participant_id = format!(
        "{}.{}",
        input.package_evidence.root_package, input.participant_path
    );
    let participant = graph
        .root_package()
        .participants()
        .values()
        .find(|participant| participant.identity().as_str() == participant_id)
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord(format!(
                "participantPath '{}' is absent from package evidence",
                input.participant_path
            ))
        })?;
    let participant_id = participant.identity().as_str().to_owned();
    let participant_kind = match participant.kind() {
        trellis_idl::ParticipantKind::Service => ParticipantKind::Service,
        trellis_idl::ParticipantKind::Device => ParticipantKind::Device,
        trellis_idl::ParticipantKind::App => ParticipantKind::App,
        trellis_idl::ParticipantKind::Agent => ParticipantKind::Agent,
    };
    let mut referenced_apis = BTreeMap::new();
    for api_id in participant
        .implements()
        .iter()
        .chain(participant.uses().keys())
    {
        let api = graph
            .packages()
            .values()
            .find_map(|package| package.apis().get(api_id))
            .ok_or_else(|| {
                AuthorizationStateError::InvalidRecord(format!(
                    "participant references unavailable API {api_id}"
                ))
            })?;
        referenced_apis.insert(
            api_id.as_str().to_owned(),
            project_api(&graph, api_id, api)?,
        );
    }
    let implemented_apis = participant
        .implements()
        .iter()
        .map(|api_id| {
            let projection = referenced_apis
                .get(api_id.as_str())
                .cloned()
                .ok_or_else(|| {
                    AuthorizationStateError::InvalidRecord(format!(
                        "implemented API {api_id} is unavailable"
                    ))
                })?;
            Ok((api_id.as_str().to_owned(), projection))
        })
        .collect::<Result<_, AuthorizationStateError>>()?;

    let needs = graph
        .participant_needs(participant.identity())
        .ok_or_else(|| {
            AuthorizationStateError::InvalidRecord("participant needs are absent".into())
        })?;
    let optional_capabilities = participant
        .uses()
        .values()
        .flat_map(|selection| selection.optional_capabilities.iter())
        .map(|capability| capability.as_str())
        .collect::<BTreeSet<_>>();
    let mut resources = BTreeMap::new();
    for (name, resource) in participant.resources() {
        let (projection, _) = project_resource(resource);
        resources.insert(name.as_str().to_owned(), projection);
    }
    let optional_grant_bundles = needs.optional_grants().clone();
    let participant_digest = participant_digest(&graph, participant.identity())
        .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?;
    Ok((
        participant_digest,
        needs.digest().to_owned(),
        ParticipantRuntimeProjection {
            participant_id,
            participant_kind,
            display_name: participant.name().to_owned(),
            implemented_apis,
            referenced_apis,
            resources,
            required_grants: needs.required_grants().clone(),
            optional_capability_definitions: optional_grant_bundles
                .iter()
                .filter(|(name, _)| optional_capabilities.contains(name.as_str()))
                .map(|(name, grants)| (name.clone(), grants.clone()))
                .collect(),
            optional_grant_bundles,
            required_capabilities: needs
                .required_capabilities()
                .iter()
                .map(|capability| capability.as_str().to_owned())
                .collect(),
        },
        evidence_json,
    ))
}

fn project_api(
    graph: &trellis_idl::PackageGraph,
    api_id: &trellis_idl::ApiId,
    api: &trellis_idl::ApiDefinition,
) -> Result<ApiRuntimeProjection, AuthorizationStateError> {
    Ok(ApiRuntimeProjection {
        digest: api_digest(graph, api_id)
            .map_err(|error| AuthorizationStateError::InvalidRecord(error.to_string()))?,
        major: api.major(),
        actions: api
            .actions()
            .iter()
            .map(|(id, action)| {
                let (kind, upload, download, event_parameter_count) = match action {
                    ActionDefinition::Rpc { download, .. } => {
                        (RuntimeActionKind::Rpc, false, *download, 0)
                    }
                    ActionDefinition::Operation { upload, .. } => {
                        (RuntimeActionKind::Operation, *upload, false, 0)
                    }
                    ActionDefinition::Event { parameters, .. } => {
                        (RuntimeActionKind::Event, false, false, parameters.len())
                    }
                    ActionDefinition::Feed { .. } => (RuntimeActionKind::Feed, false, false, 0),
                };
                (
                    format!("{}:{}", action_kind(id.kind), id.name),
                    ActionRuntimeProjection {
                        kind,
                        upload,
                        download,
                        event_parameter_count,
                    },
                )
            })
            .collect(),
        capabilities: api
            .capabilities()
            .iter()
            .map(|(id, capability)| {
                Ok((
                    id.as_str().to_owned(),
                    CapabilityRuntimeProjection {
                        display_name: capability.title.clone(),
                        description: capability.description.clone(),
                        allows: capability
                            .allows
                            .iter()
                            .map(|selection| {
                                selected_permission_atoms(api_id, selection, api).map_err(|error| {
                                    AuthorizationStateError::InvalidRecord(error.to_string())
                                })
                            })
                            .collect::<Result<Vec<_>, _>>()?
                            .into_iter()
                            .flatten()
                            .collect(),
                    },
                ))
            })
            .collect::<Result<_, AuthorizationStateError>>()?,
    })
}

fn project_resource(
    resource: &ResourceDefinition,
) -> (ResourceRuntimeProjection, Vec<PermissionAction>) {
    let mut projection = ResourceRuntimeProjection {
        kind: ParticipantResourceKind::State,
        optional: false,
        history: None,
        ttl_ms: None,
        desired_max_value: None,
        desired_max_object: None,
        desired_max_total: None,
        deadline_ms: None,
        payload_schema: None,
        result_schema: None,
        update_schema: None,
        retry_attempts: None,
        retry_backoff_ms: Vec::new(),
        consumer_events: BTreeMap::new(),
        consumer_concurrency: None,
        consumer_replay_all: false,
    };
    let actions = match resource {
        ResourceDefinition::State { optional, .. } => {
            projection.optional = *optional;
            vec![
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ]
        }
        ResourceDefinition::Kv {
            optional,
            history,
            ttl_ms,
            desired_max_value,
            ..
        } => {
            projection.kind = ParticipantResourceKind::Kv;
            projection.optional = *optional;
            projection.history = Some(*history);
            projection.ttl_ms = Some(*ttl_ms);
            projection.desired_max_value = *desired_max_value;
            vec![
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ]
        }
        ResourceDefinition::Store {
            optional,
            ttl_ms,
            desired_max_object,
            desired_max_total,
            ..
        } => {
            projection.kind = ParticipantResourceKind::Store;
            projection.optional = *optional;
            projection.ttl_ms = Some(*ttl_ms);
            projection.desired_max_object = *desired_max_object;
            projection.desired_max_total = *desired_max_total;
            vec![
                PermissionAction::Read,
                PermissionAction::Write,
                PermissionAction::Delete,
            ]
        }
        ResourceDefinition::Job {
            optional,
            payload,
            result,
            update,
            deadline_ms,
            retry,
            ..
        } => {
            projection.kind = ParticipantResourceKind::JobQueue;
            projection.optional = *optional;
            projection.deadline_ms = *deadline_ms;
            projection.payload_schema = Some(payload.id.as_str().to_owned());
            projection.result_schema = result.as_ref().map(|value| value.id.as_str().to_owned());
            projection.update_schema = update.as_ref().map(|value| value.id.as_str().to_owned());
            if let Some(retry) = retry {
                projection.retry_attempts = Some(retry.attempts);
                projection.retry_backoff_ms = retry.backoff_ms.clone();
            }
            vec![PermissionAction::Submit, PermissionAction::Process]
        }
        ResourceDefinition::Consumer {
            optional,
            events,
            concurrency,
            replay,
            retry,
            ..
        } => {
            projection.kind = ParticipantResourceKind::EventConsumer;
            projection.optional = *optional;
            projection.consumer_concurrency = Some(*concurrency);
            projection.consumer_replay_all = matches!(replay, trellis_idl::Replay::All);
            for (api, event) in events {
                projection
                    .consumer_events
                    .entry(api.as_str().to_owned())
                    .or_default()
                    .push(event.clone());
            }
            if let Some(retry) = retry {
                projection.retry_attempts = Some(retry.attempts);
                projection.retry_backoff_ms = retry.backoff_ms.clone();
            }
            vec![PermissionAction::Consume]
        }
    };
    (projection, actions)
}

fn action_kind(kind: ActionKind) -> &'static str {
    match kind {
        ActionKind::Rpc => "rpc",
        ActionKind::Operation => "operation",
        ActionKind::Event => "event",
        ActionKind::Feed => "feed",
    }
}

fn protocol(error: trellis_protocol::ProtocolError) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use trellis_rs::generated::ParticipantDescriptor;

    fn generated_platform_evidence() -> PackageEvidenceInput {
        let generated =
            trellis_runtime_apis::participants::trellis_platform::Participant::package_evidence();
        PackageEvidenceInput::from_generated_descriptor(
            generated,
            trellis_runtime_apis::participants::trellis_platform::PARTICIPANT_PATH,
        )
        .expect("decode native evidence")
    }

    #[test]
    fn generated_evidence_resolves_to_the_exact_participant_digest() {
        let evidence = generated_platform_evidence();
        let graph = compile_evidence(evidence.package_evidence.clone()).expect("compile evidence");
        let participant = graph
            .root_package()
            .participants()
            .values()
            .find(|participant| {
                participant.identity().as_str()
                    == trellis_runtime_apis::participants::trellis_platform::PARTICIPANT_ID
            })
            .expect("platform participant");
        let expected_needs = graph
            .participant_needs(participant.identity())
            .expect("platform participant needs");
        let (digest, needs_digest, projection, _) =
            verify_package_evidence(&evidence).expect("verify evidence");
        assert_eq!(
            digest,
            trellis_runtime_apis::participants::trellis_platform::PARTICIPANT_DIGEST
        );
        assert_eq!(needs_digest, expected_needs.digest());
        assert_eq!(
            projection.required_grants,
            *expected_needs.required_grants()
        );
        assert_eq!(
            projection.optional_grant_bundles,
            *expected_needs.optional_grants()
        );
        assert_eq!(
            projection.participant_id,
            trellis_runtime_apis::participants::trellis_platform::PARTICIPANT_ID
        );
    }

    #[test]
    fn tampered_generated_source_evidence_is_rejected() {
        let mut evidence = generated_platform_evidence();
        evidence.package_evidence.packages[0].source = evidence.package_evidence.packages[0]
            .source
            .replacen("package \"trellis\";", "package \"forged\";", 1);
        assert!(verify_package_evidence(&evidence).is_err());
    }
}

fn invalid<T>(message: impl Into<String>) -> Result<T, AuthorizationStateError> {
    Err(AuthorizationStateError::InvalidRecord(message.into()))
}
