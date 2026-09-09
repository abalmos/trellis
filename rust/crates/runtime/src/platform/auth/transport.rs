use std::collections::{BTreeMap, BTreeSet};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;
use trellis_protocol::{
    parse_api, parse_participant, resolve_participant, ApiArtifact, ApiSurfaceKind,
    AuthorizationPrincipalKind, ParticipantResourceKind, PermissionAction,
    UnsignedAuthorizationContext,
};

use super::{
    AuthorizationRegistryBinding, AuthorizationStateError, ParticipantBindingRecord,
    ResourceBindingEvidence, ResourceProviderIdentity,
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransportPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

pub(crate) fn compile_transport_permissions(
    context: &UnsignedAuthorizationContext,
    binding: &ParticipantBindingRecord,
    resource_bindings: &[ResourceBindingEvidence],
    registry: &AuthorizationRegistryBinding,
) -> Result<TransportPermissions, AuthorizationStateError> {
    if binding.participant_id != context.participant_id {
        return invalid("issuable state does not match participant binding");
    }
    let participant_value: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid_error(format!("participant JSON is invalid: {error}")))?;
    let participant =
        parse_participant(&participant_value).map_err(|error| invalid_error(error.to_string()))?;
    let api_values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| invalid_error(format!("API artifact map is invalid: {error}")))?;
    let mut apis = BTreeMap::new();
    for (api_id, value) in api_values {
        let api = parse_api(&value).map_err(|error| invalid_error(error.to_string()))?;
        if api.id() != api_id {
            return invalid("API artifact map key does not match artifact ID");
        }
        apis.insert(api_id, api);
    }
    let resolved = resolve_participant(&participant, &apis)
        .map_err(|error| invalid_error(error.to_string()))?;
    if resolved.participant_digest() != binding.artifact_digest
        || resolved
            .needs()
            .digest()
            .map_err(|error| invalid_error(error.to_string()))?
            != binding.needs_digest
    {
        return invalid("resolved participant identity does not match issuable state");
    }

    let mut publish = BTreeSet::new();
    let mut subscribe = BTreeSet::from([format!("{}.>", context.inbox_prefix)]);

    // Contexts and exact revocation watches are public protocol evidence, not
    // participant resources. Issuer keys are resolved over the configured HTTPS origin.
    publish.insert("$JS.API.INFO".to_owned());
    let context_stream = format!("KV_{}", registry.context_bucket);
    publish.insert(format!("$JS.FC.{context_stream}.>"));
    publish.insert(format!(
        "$JS.API.DIRECT.GET.{context_stream}.$KV.{}.*",
        registry.context_bucket
    ));
    publish.insert(format!(
        "$JS.API.DIRECT.GET.{context_stream}.$KV.{}.revocation.*",
        registry.context_bucket
    ));
    subscribe.insert(format!("$KV.{}.revocation.*", registry.context_bucket));
    if matches!(
        context.principal_kind,
        AuthorizationPrincipalKind::Service | AuthorizationPrincipalKind::Device
    ) {
        publish.insert("$JS.API.INFO".to_owned());
        if context.principal_kind == AuthorizationPrincipalKind::Service {
            let instance_id = context
                .instance_id
                .as_deref()
                .ok_or_else(|| invalid_error("service instance is missing"))?;
            let bucket = format!("trellis_operations_{instance_id}");
            let stream = format!("KV_{bucket}");
            publish.insert(format!("$KV.{bucket}.>"));
            publish.insert(format!("$JS.API.STREAM.CREATE.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.DELETE.{stream}.>"));
            publish.insert(format!("$JS.API.$KV.{bucket}.>"));
            kv_read(&bucket, &mut publish);
        }
        let deployment_id = context.deployment_id.as_deref().ok_or_else(|| {
            invalid_error("deployed principal is missing deployment identity".to_owned())
        })?;
        let instance_id = context.instance_id.as_deref().ok_or_else(|| {
            invalid_error("deployed principal is missing instance identity".to_owned())
        })?;
        let kind = match context.principal_kind {
            AuthorizationPrincipalKind::Service => "service",
            AuthorizationPrincipalKind::Device => "device",
            AuthorizationPrincipalKind::User => unreachable!(),
        };
        publish.insert(format!(
            "health.v1.heartbeat.{kind}.{}.{}.{}.{}.{}",
            URL_SAFE_NO_PAD.encode(context.participant_id.as_bytes()),
            URL_SAFE_NO_PAD.encode(binding.artifact_digest.as_bytes()),
            URL_SAFE_NO_PAD.encode(deployment_id.as_bytes()),
            URL_SAFE_NO_PAD.encode(instance_id.as_bytes()),
            context.session_key,
        ));
    }

    for implementation in resolved.implemented_apis() {
        let provided = implementation.provided();
        let api = apis
            .get(provided.api())
            .ok_or_else(|| invalid_error("implemented API artifact is missing".to_owned()))?;
        let api_value = api
            .normalized_value()
            .map_err(|error| invalid_error(error.to_string()))?;
        let session_prefix = &context.session_key[..16.min(context.session_key.len())];
        subscribe.extend(provided.rpc().values().cloned());
        for name in provided.rpc().keys() {
            if api_value["rpc"][name]["transfer"]["direction"].as_str() == Some("receive") {
                subscribe.insert(format!("transfer.v1.download.{session_prefix}.*"));
            }
        }
        for operation in provided.operations().values() {
            subscribe.insert(operation.subject().to_owned());
            subscribe.insert(format!("{}.control", operation.subject()));
        }
        for name in provided.operations().keys() {
            if api_value["operations"][name]["transfer"]["direction"].as_str() == Some("send") {
                subscribe.insert(format!("transfer.v1.upload.{session_prefix}.*"));
            }
        }
        subscribe.extend(provided.feeds().values().cloned());
    }

    for atom in context.grants.permissions() {
        if let Some((api_id, surface, name)) = atom.target().as_api_surface() {
            let api = apis
                .get(api_id)
                .ok_or_else(|| invalid_error(format!("grant references unknown API {api_id}")))?;
            compile_api_surface(
                api,
                surface,
                name,
                atom.action(),
                &mut publish,
                &mut subscribe,
            )?;
        } else if let Some((api_id, operation, _signal)) = atom.target().as_operation_signal() {
            let subject = api_subject(apis.get(api_id), ApiSurfaceKind::Operation, operation)?;
            if atom.action() != PermissionAction::Control {
                return invalid("operation signal grant must use control action");
            }
            publish.insert(format!("{subject}.control"));
        } else if let Some((participant_id, kind, name)) = atom.target().as_participant_resource() {
            if participant_id != context.participant_id {
                return invalid("resource grant belongs to another participant");
            }
            let resource = resource_binding(resource_bindings, kind, name)?;
            compile_resource(resource, atom.action(), &mut publish, &mut subscribe)?;
        }
    }

    Ok(TransportPermissions {
        publish: publish.into_iter().collect(),
        subscribe: subscribe.into_iter().collect(),
    })
}

fn compile_api_surface(
    api: &ApiArtifact,
    surface: ApiSurfaceKind,
    name: &str,
    action: PermissionAction,
    publish: &mut BTreeSet<String>,
    subscribe: &mut BTreeSet<String>,
) -> Result<(), AuthorizationStateError> {
    let subjects = api
        .derived_subjects()
        .map_err(|error| invalid_error(error.to_string()))?;
    let api_value = api
        .normalized_value()
        .map_err(|error| invalid_error(error.to_string()))?;
    match (surface, action) {
        (ApiSurfaceKind::Rpc, PermissionAction::Call) => {
            publish.insert(required_subject(subjects.rpc.get(name), "RPC", name)?);
            if api_value["rpc"][name]["transfer"]["direction"].as_str() == Some("receive") {
                publish.insert("transfer.v1.download.*.*".to_owned());
            }
        }
        (ApiSurfaceKind::Operation, PermissionAction::Invoke) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(subject.clone());
            publish.insert(format!("{subject}.control"));
            if api_value["operations"][name]["transfer"]["direction"].as_str() == Some("send") {
                publish.insert("transfer.v1.upload.*.*".to_owned());
            }
        }
        (ApiSurfaceKind::Operation, PermissionAction::Observe) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(format!("{subject}.control"));
        }
        (ApiSurfaceKind::Operation, PermissionAction::Cancel | PermissionAction::Control) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(format!("{subject}.control"));
        }
        (ApiSurfaceKind::Event, PermissionAction::Publish) => {
            publish.insert(
                subjects
                    .events
                    .get(name)
                    .ok_or_else(|| invalid_error(format!("unknown event {name}")))?
                    .wildcard
                    .clone(),
            );
        }
        (ApiSurfaceKind::Event, PermissionAction::Subscribe) => {
            subscribe.insert(
                subjects
                    .events
                    .get(name)
                    .ok_or_else(|| invalid_error(format!("unknown event {name}")))?
                    .wildcard
                    .clone(),
            );
        }
        (ApiSurfaceKind::Feed, PermissionAction::Subscribe) => {
            publish.insert(required_subject(subjects.feeds.get(name), "feed", name)?);
        }
        (ApiSurfaceKind::State, PermissionAction::Read) => {
            publish.insert("rpc.v1.State.Get".to_owned());
            publish.insert("rpc.v1.State.List".to_owned());
        }
        (ApiSurfaceKind::State, PermissionAction::Write) => {
            publish.insert("rpc.v1.State.Put".to_owned());
        }
        (ApiSurfaceKind::State, PermissionAction::Delete) => {
            publish.insert("rpc.v1.State.Delete".to_owned());
        }
        _ => return invalid("grant action does not match API surface"),
    }
    Ok(())
}

fn api_subject(
    api: Option<&ApiArtifact>,
    surface: ApiSurfaceKind,
    name: &str,
) -> Result<String, AuthorizationStateError> {
    let api = api.ok_or_else(|| invalid_error("grant references unknown API"))?;
    let subjects = api
        .derived_subjects()
        .map_err(|error| invalid_error(error.to_string()))?;
    match surface {
        ApiSurfaceKind::Operation => {
            required_subject(subjects.operations.get(name), "operation", name)
        }
        _ => invalid("unsupported subject lookup"),
    }
}

fn required_subject(
    subject: Option<&String>,
    kind: &str,
    name: &str,
) -> Result<String, AuthorizationStateError> {
    subject
        .cloned()
        .ok_or_else(|| invalid_error(format!("unknown {kind} {name}")))
}

fn resource_binding<'a>(
    resources: &'a [ResourceBindingEvidence],
    kind: ParticipantResourceKind,
    name: &str,
) -> Result<&'a ResourceBindingEvidence, AuthorizationStateError> {
    let kind = match kind {
        ParticipantResourceKind::Kv => "kv",
        ParticipantResourceKind::Store => "store",
        ParticipantResourceKind::JobQueue => "jobQueue",
        ParticipantResourceKind::EventConsumer => "eventConsumer",
        ParticipantResourceKind::State => "state",
    };
    resources
        .iter()
        .find(|resource| resource.resource_kind == kind && resource.local_name == name)
        .ok_or_else(|| invalid_error(format!("missing physical binding for {kind} {name}")))
}

fn compile_resource(
    resource: &ResourceBindingEvidence,
    action: PermissionAction,
    publish: &mut BTreeSet<String>,
    subscribe: &mut BTreeSet<String>,
) -> Result<(), AuthorizationStateError> {
    match (&resource.provider_identity, action) {
        (ResourceProviderIdentity::Kv { bucket }, PermissionAction::Read) => {
            kv_read(bucket, publish);
        }
        (ResourceProviderIdentity::Kv { bucket }, PermissionAction::Write)
        | (ResourceProviderIdentity::Kv { bucket }, PermissionAction::Delete) => {
            publish.insert(format!("$KV.{bucket}.>"));
            publish.insert(format!("$JS.API.STREAM.INFO.KV_{bucket}"));
        }
        (
            ResourceProviderIdentity::State { .. },
            PermissionAction::Read | PermissionAction::Write | PermissionAction::Delete,
        ) => {}
        (ResourceProviderIdentity::Store { bucket }, PermissionAction::Read) => {
            let stream = format!("OBJ_{bucket}");
            publish.insert("$JS.API.INFO".to_owned());
            publish.insert(format!("$JS.API.STREAM.INFO.{stream}"));
            publish.insert(format!("$JS.API.STREAM.MSG.GET.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.INFO.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.DELETE.{stream}.>"));
            publish.insert(format!("$JS.FC.{stream}.>"));
            publish.insert(format!("$JS.ACK.{stream}.>"));
        }
        (
            ResourceProviderIdentity::Store { bucket },
            PermissionAction::Write | PermissionAction::Delete,
        ) => {
            publish.insert(format!("$O.{bucket}.C.>"));
            publish.insert(format!("$O.{bucket}.M.>"));
            publish.insert(format!("$JS.API.STREAM.PURGE.OBJ_{bucket}"));
        }
        (
            ResourceProviderIdentity::JobQueue {
                namespace: _,
                work_stream,
                publish_prefix,
                updates_prefix,
                ..
            },
            PermissionAction::Submit,
        ) => {
            publish.insert(format!("{publish_prefix}.>"));
            publish.insert(format!("$JS.API.CONSUMER.INFO.{work_stream}.>"));
            if let Some(prefix) = updates_prefix {
                subscribe.insert(format!("{prefix}.>"));
            }
        }
        (
            ResourceProviderIdentity::JobQueue {
                namespace,
                work_stream,
                publish_prefix,
                work_subject,
                consumer,
                updates_prefix,
                ..
            },
            PermissionAction::Process,
        ) => {
            let keys_bucket = format!("JOBS_KEYS_{namespace}");
            subscribe.insert(work_subject.clone());
            subscribe.insert(format!("{publish_prefix}.>"));
            publish.insert(format!("{publish_prefix}.>"));
            publish.insert(format!("trellis.jobs.workers.{namespace}.>"));
            publish.insert("$JS.API.DIRECT.GET.JOBS".to_owned());
            publish.insert("$JS.API.DIRECT.GET.JOBS.>".to_owned());
            publish.insert("$JS.API.STREAM.MSG.GET.JOBS".to_owned());
            publish.insert(format!("$KV.{keys_bucket}.>"));
            kv_read(&keys_bucket, publish);
            publish.insert(format!("$JS.API.STREAM.INFO.{work_stream}"));
            publish.insert(format!("$JS.API.CONSUMER.INFO.{work_stream}.{consumer}"));
            publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{work_stream}.>"));
            publish.insert(format!("$JS.ACK.{work_stream}.>"));
            if let Some(prefix) = updates_prefix {
                publish.insert(format!("{prefix}.>"));
                subscribe.insert(format!("{prefix}.>"));
            }
        }
        (
            ResourceProviderIdentity::EventConsumer {
                stream, consumer, ..
            },
            PermissionAction::Consume,
        ) => {
            publish.insert(format!("$JS.API.CONSUMER.INFO.{stream}.{consumer}"));
            publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.{consumer}"));
            publish.insert(format!("$JS.ACK.{stream}.{consumer}.>"));
        }
        _ => return invalid("resource action does not match physical binding"),
    }
    Ok(())
}

fn kv_read(bucket: &str, publish: &mut BTreeSet<String>) {
    let stream = format!("KV_{bucket}");
    publish.insert(format!("$JS.API.STREAM.INFO.{stream}"));
    publish.insert(format!("$JS.API.STREAM.MSG.GET.{stream}"));
    publish.insert(format!("$JS.API.DIRECT.GET.{stream}"));
    publish.insert(format!("$JS.API.DIRECT.GET.{stream}.>"));
    publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}.>"));
    publish.insert(format!("$JS.API.CONSUMER.INFO.{stream}.>"));
    publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.>"));
    publish.insert(format!("$JS.ACK.{stream}.>"));
}

fn invalid<T>(message: impl Into<String>) -> Result<T, AuthorizationStateError> {
    Err(invalid_error(message))
}

fn invalid_error(message: impl Into<String>) -> AuthorizationStateError {
    AuthorizationStateError::InvalidRecord(message.into())
}

#[cfg(test)]
mod tests {
    use super::compile_resource;
    use crate::platform::auth::{
        ResourceBindingEvidence, ResourceBindingState, ResourceProviderIdentity,
    };
    use std::collections::BTreeSet;
    use trellis_protocol::PermissionAction;

    #[test]
    fn job_process_update_subscription_is_resource_exact() {
        let resource = ResourceBindingEvidence {
            resource_kind: "jobQueue".to_owned(),
            local_name: "documents".to_owned(),
            binding_id: "binding-documents".to_owned(),
            owner_participant_id: "acme.jobs@v1".to_owned(),
            provider_identity: ResourceProviderIdentity::JobQueue {
                namespace: "tr_jobs_documents".to_owned(),
                work_stream: "JOBS_WORK_DOCUMENTS".to_owned(),
                publish_prefix: "trellis.jobs.tr_jobs_documents.process".to_owned(),
                updates_prefix: Some("trellis.job_updates.tr_jobs_documents.process".to_owned()),
                work_subject: "trellis.work.tr_jobs_documents.process".to_owned(),
                consumer: "worker-documents".to_owned(),
            },
            state: ResourceBindingState::Available,
            materialized_at: 1,
            error: None,
        };
        let mut publish = BTreeSet::new();
        let mut subscribe = BTreeSet::new();

        compile_resource(
            &resource,
            PermissionAction::Process,
            &mut publish,
            &mut subscribe,
        )
        .unwrap();

        assert_eq!(
            subscribe,
            BTreeSet::from([
                "trellis.job_updates.tr_jobs_documents.process.>".to_owned(),
                "trellis.jobs.tr_jobs_documents.process.>".to_owned(),
                "trellis.work.tr_jobs_documents.process".to_owned(),
            ])
        );
    }
}
