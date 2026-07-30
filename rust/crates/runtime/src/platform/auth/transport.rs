use std::collections::{BTreeMap, BTreeSet};

use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use serde_json::Value;
use trellis_protocol::{
    parse_api_v1, parse_participant_v1, resolve_participant_v1, ApiArtifactV1, ApiSurfaceKindV1,
    AuthorizationPrincipalKindV1, ParticipantResourceKindV1, PermissionActionV1,
};

use super::{
    AuthorizationStateError, IssuableAuthorizationState, ParticipantBindingRecord,
    ResourceBindingEvidence, ResourceProviderIdentity,
};

const REQUEST_VALIDATE_SUBJECT: &str = "rpc.v1.Auth.Requests.Validate";
const EVENT_VALIDATE_SUBJECT: &str = "rpc.v1.Auth.Events.Validate";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransportPermissions {
    pub publish: Vec<String>,
    pub subscribe: Vec<String>,
}

pub(crate) fn compile_transport_permissions(
    state: &IssuableAuthorizationState,
    binding: &ParticipantBindingRecord,
) -> Result<TransportPermissions, AuthorizationStateError> {
    if binding.participant_id != state.participant.id
        || binding.artifact_digest != state.participant.artifact_digest
        || binding.needs_digest != state.participant.needs_digest
    {
        return invalid("issuable state does not match participant binding");
    }
    let participant_value: Value = serde_json::from_str(&binding.participant_json)
        .map_err(|error| invalid_error(format!("participant JSON is invalid: {error}")))?;
    let participant = parse_participant_v1(&participant_value)
        .map_err(|error| invalid_error(error.to_string()))?;
    let api_values: BTreeMap<String, Value> = serde_json::from_str(&binding.api_artifacts_json)
        .map_err(|error| invalid_error(format!("API artifact map is invalid: {error}")))?;
    let mut apis = BTreeMap::new();
    for (api_id, value) in api_values {
        let api = parse_api_v1(&value).map_err(|error| invalid_error(error.to_string()))?;
        if api.id() != api_id {
            return invalid("API artifact map key does not match artifact ID");
        }
        apis.insert(api_id, api);
    }
    let resolved = resolve_participant_v1(&participant, &apis)
        .map_err(|error| invalid_error(error.to_string()))?;
    if resolved.participant_digest() != state.participant.artifact_digest
        || resolved
            .needs()
            .digest()
            .map_err(|error| invalid_error(error.to_string()))?
            != state.participant.needs_digest
    {
        return invalid("resolved participant identity does not match issuable state");
    }

    let mut publish = BTreeSet::from([
        REQUEST_VALIDATE_SUBJECT.to_owned(),
        EVENT_VALIDATE_SUBJECT.to_owned(),
    ]);
    let mut subscribe = BTreeSet::from([format!("{}.>", state.inbox_prefix)]);
    if matches!(
        state.principal.kind,
        AuthorizationPrincipalKindV1::Service | AuthorizationPrincipalKindV1::Device
    ) {
        publish.insert("$JS.API.INFO".to_owned());
        if state.principal.kind == AuthorizationPrincipalKindV1::Service {
            let instance_id = state
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
        let deployment_id = state.deployment_id.as_deref().ok_or_else(|| {
            invalid_error("deployed principal is missing deployment identity".to_owned())
        })?;
        let instance_id = state.instance_id.as_deref().ok_or_else(|| {
            invalid_error("deployed principal is missing instance identity".to_owned())
        })?;
        let kind = match state.principal.kind {
            AuthorizationPrincipalKindV1::Service => "service",
            AuthorizationPrincipalKindV1::Device => "device",
            AuthorizationPrincipalKindV1::User => unreachable!(),
        };
        publish.insert(format!(
            "health.v1.heartbeat.{kind}.{}.{}.{}.{}.{}",
            URL_SAFE_NO_PAD.encode(state.participant.id.as_bytes()),
            URL_SAFE_NO_PAD.encode(state.participant.artifact_digest.as_bytes()),
            URL_SAFE_NO_PAD.encode(deployment_id.as_bytes()),
            URL_SAFE_NO_PAD.encode(instance_id.as_bytes()),
            state.session_public_key,
        ));
    }

    // ponytail: transitional until the Jobs owner is folded into trellis-server.
    if state.principal.kind == AuthorizationPrincipalKindV1::Service
        && state.participant.id == "trellis.jobs@v1"
    {
        publish.insert("trellis.jobs.>".to_owned());
        subscribe.insert("feed.v1.Jobs.Watch".to_owned());
        for stream in ["JOBS", "JOBS_ADVISORIES"] {
            publish.insert(format!("$JS.API.STREAM.INFO.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.LIST.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.NAMES.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.DURABLE.CREATE.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.INFO.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.DELETE.{stream}.>"));
            publish.insert(format!("$JS.ACK.{stream}.>"));
        }
        publish.insert("$JS.API.STREAM.MSG.GET.JOBS".to_owned());
        publish.insert("$JS.API.DIRECT.GET.JOBS".to_owned());
        publish.insert("$JS.API.DIRECT.GET.JOBS.>".to_owned());
        publish.insert("$JS.API.STREAM.INFO.JOBS_WORK".to_owned());
        publish.insert("$JS.API.STREAM.MSG.GET.JOBS_WORK".to_owned());
        publish.insert("$JS.API.DIRECT.GET.JOBS_WORK".to_owned());
        publish.insert("$JS.API.DIRECT.GET.JOBS_WORK.>".to_owned());
    }

    for implementation in resolved.implemented_apis() {
        let provided = implementation.provided();
        let api = apis
            .get(provided.api())
            .ok_or_else(|| invalid_error("implemented API artifact is missing".to_owned()))?;
        let api_value = api
            .normalized_value()
            .map_err(|error| invalid_error(error.to_string()))?;
        let session_prefix = &state.session_public_key[..16.min(state.session_public_key.len())];
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

    for atom in state.grant_set.permissions() {
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
            let subject = api_subject(apis.get(api_id), ApiSurfaceKindV1::Operation, operation)?;
            if atom.action() != PermissionActionV1::Control {
                return invalid("operation signal grant must use control action");
            }
            publish.insert(format!("{subject}.control"));
        } else if let Some((participant_id, kind, name)) = atom.target().as_participant_resource() {
            if participant_id != state.participant.id {
                return invalid("resource grant belongs to another participant");
            }
            let resource = resource_binding(&state.resource_bindings, kind, name)?;
            compile_resource(resource, atom.action(), &mut publish, &mut subscribe)?;
        }
    }

    Ok(TransportPermissions {
        publish: publish.into_iter().collect(),
        subscribe: subscribe.into_iter().collect(),
    })
}

fn compile_api_surface(
    api: &ApiArtifactV1,
    surface: ApiSurfaceKindV1,
    name: &str,
    action: PermissionActionV1,
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
        (ApiSurfaceKindV1::Rpc, PermissionActionV1::Call) => {
            publish.insert(required_subject(subjects.rpc.get(name), "RPC", name)?);
            if api_value["rpc"][name]["transfer"]["direction"].as_str() == Some("receive") {
                publish.insert("transfer.v1.download.*.*".to_owned());
            }
        }
        (ApiSurfaceKindV1::Operation, PermissionActionV1::Invoke) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(subject.clone());
            publish.insert(format!("{subject}.control"));
            if api_value["operations"][name]["transfer"]["direction"].as_str() == Some("send") {
                publish.insert("transfer.v1.upload.*.*".to_owned());
            }
        }
        (ApiSurfaceKindV1::Operation, PermissionActionV1::Observe) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(format!("{subject}.control"));
        }
        (ApiSurfaceKindV1::Operation, PermissionActionV1::Cancel | PermissionActionV1::Control) => {
            let subject = required_subject(subjects.operations.get(name), "operation", name)?;
            publish.insert(format!("{subject}.control"));
        }
        (ApiSurfaceKindV1::Event, PermissionActionV1::Publish) => {
            publish.insert(
                subjects
                    .events
                    .get(name)
                    .ok_or_else(|| invalid_error(format!("unknown event {name}")))?
                    .wildcard
                    .clone(),
            );
        }
        (ApiSurfaceKindV1::Event, PermissionActionV1::Subscribe) => {
            subscribe.insert(
                subjects
                    .events
                    .get(name)
                    .ok_or_else(|| invalid_error(format!("unknown event {name}")))?
                    .wildcard
                    .clone(),
            );
        }
        (ApiSurfaceKindV1::Feed, PermissionActionV1::Subscribe) => {
            publish.insert(required_subject(subjects.feeds.get(name), "feed", name)?);
        }
        (ApiSurfaceKindV1::State, PermissionActionV1::Read) => {
            publish.insert("rpc.v1.State.Get".to_owned());
            publish.insert("rpc.v1.State.List".to_owned());
        }
        (ApiSurfaceKindV1::State, PermissionActionV1::Write) => {
            publish.insert("rpc.v1.State.Put".to_owned());
        }
        (ApiSurfaceKindV1::State, PermissionActionV1::Delete) => {
            publish.insert("rpc.v1.State.Delete".to_owned());
        }
        _ => return invalid("grant action does not match API surface"),
    }
    Ok(())
}

fn api_subject(
    api: Option<&ApiArtifactV1>,
    surface: ApiSurfaceKindV1,
    name: &str,
) -> Result<String, AuthorizationStateError> {
    let api = api.ok_or_else(|| invalid_error("grant references unknown API"))?;
    let subjects = api
        .derived_subjects()
        .map_err(|error| invalid_error(error.to_string()))?;
    match surface {
        ApiSurfaceKindV1::Operation => {
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
    kind: ParticipantResourceKindV1,
    name: &str,
) -> Result<&'a ResourceBindingEvidence, AuthorizationStateError> {
    let kind = match kind {
        ParticipantResourceKindV1::Kv => "kv",
        ParticipantResourceKindV1::Store => "store",
        ParticipantResourceKindV1::JobQueue => "jobQueue",
        ParticipantResourceKindV1::EventConsumer => "eventConsumer",
        ParticipantResourceKindV1::State => "state",
    };
    resources
        .iter()
        .find(|resource| resource.resource_kind == kind && resource.local_name == name)
        .ok_or_else(|| invalid_error(format!("missing physical binding for {kind} {name}")))
}

fn compile_resource(
    resource: &ResourceBindingEvidence,
    action: PermissionActionV1,
    publish: &mut BTreeSet<String>,
    subscribe: &mut BTreeSet<String>,
) -> Result<(), AuthorizationStateError> {
    match (&resource.provider_identity, action) {
        (ResourceProviderIdentity::Kv { bucket }, PermissionActionV1::Read)
        | (ResourceProviderIdentity::State { bucket }, PermissionActionV1::Read) => {
            kv_read(bucket, publish);
        }
        (ResourceProviderIdentity::Kv { bucket }, PermissionActionV1::Write)
        | (ResourceProviderIdentity::Kv { bucket }, PermissionActionV1::Delete)
        | (ResourceProviderIdentity::State { bucket }, PermissionActionV1::Write)
        | (ResourceProviderIdentity::State { bucket }, PermissionActionV1::Delete) => {
            publish.insert(format!("$KV.{bucket}.>"));
            publish.insert(format!("$JS.API.STREAM.INFO.KV_{bucket}"));
        }
        (ResourceProviderIdentity::Store { bucket }, PermissionActionV1::Read) => {
            let stream = format!("OBJ_{bucket}");
            publish.insert("$JS.API.INFO".to_owned());
            publish.insert(format!("$JS.API.STREAM.INFO.{stream}"));
            publish.insert(format!("$JS.API.STREAM.MSG.GET.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}"));
            publish.insert(format!("$JS.API.CONSUMER.CREATE.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.MSG.NEXT.{stream}.>"));
            publish.insert(format!("$JS.API.CONSUMER.DELETE.{stream}.>"));
            publish.insert(format!("$JS.FC.{stream}.>"));
            publish.insert(format!("$JS.ACK.{stream}.>"));
        }
        (
            ResourceProviderIdentity::Store { bucket },
            PermissionActionV1::Write | PermissionActionV1::Delete,
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
            PermissionActionV1::Submit,
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
            PermissionActionV1::Process,
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
            }
        }
        (
            ResourceProviderIdentity::EventConsumer {
                stream, consumer, ..
            },
            PermissionActionV1::Consume,
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
    use super::compile_transport_permissions;
    use crate::platform::auth::{
        AuthorityKind, IssuableAuthorizationState, ParticipantBindingRecord,
        ParticipantBindingState, ResourceBindingEvidence, ResourceBindingState,
        ResourceProviderIdentity,
    };
    use serde_json::Value;
    use trellis_protocol::{
        parse_api_v1, parse_participant_v1, resolve_participant_v1, ApiSurfaceKindV1,
        AuthorizationAuthorityKindV1, AuthorizationAuthorityRefV1, AuthorizationParticipantV1,
        AuthorizationPrincipalKindV1, AuthorizationPrincipalV1, GrantSetV1,
        ParticipantResourceKindV1, PermissionActionV1, PermissionAtomV1, PermissionTargetV1,
    };

    #[test]
    fn compiler_is_exact_sorted_and_action_scoped() {
        let (binding, mut state) = fixture();
        state.grant_set = GrantSetV1::new(vec![
            PermissionAtomV1::new(
                PermissionTargetV1::api_surface(
                    "trellis.auth@v1",
                    ApiSurfaceKindV1::Rpc,
                    "Auth.Sessions.Me",
                )
                .unwrap(),
                PermissionActionV1::Call,
            )
            .unwrap(),
            PermissionAtomV1::new(
                PermissionTargetV1::participant_resource(
                    "trellis-auth-runtime",
                    ParticipantResourceKindV1::Kv,
                    "browserFlows",
                )
                .unwrap(),
                PermissionActionV1::Read,
            )
            .unwrap(),
        ]);
        state.resource_bindings = vec![ResourceBindingEvidence {
            resource_kind: "kv".to_owned(),
            local_name: "browserFlows".to_owned(),
            binding_id: "binding-browser-flows".to_owned(),
            owner_participant_id: "trellis-auth-runtime".to_owned(),
            provider_identity: ResourceProviderIdentity::Kv {
                bucket: "AUTH_BROWSER_FLOWS".to_owned(),
            },
            state: ResourceBindingState::Available,
            materialized_at: 1,
            error: None,
        }];

        let permissions = compile_transport_permissions(&state, &binding).unwrap();
        assert!(permissions.publish.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(permissions
            .subscribe
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        assert!(permissions
            .publish
            .contains(&"rpc.v1.Auth.Sessions.Me".to_owned()));
        assert!(!permissions
            .publish
            .contains(&"rpc.v1.Auth.Users.Create".to_owned()));
        assert!(permissions
            .publish
            .contains(&"$JS.API.DIRECT.GET.KV_AUTH_BROWSER_FLOWS".to_owned()));
        assert!(!permissions
            .publish
            .contains(&"$KV.AUTH_BROWSER_FLOWS.>".to_owned()));
        assert!(permissions
            .subscribe
            .contains(&"_INBOX.session.>".to_owned()));
        assert_eq!(
            permissions,
            compile_transport_permissions(&state, &binding).unwrap()
        );
    }

    #[test]
    fn resource_binding_never_creates_authority() {
        let (binding, mut state) = fixture();
        state.resource_bindings = vec![kv_binding("trellis-auth-runtime")];

        state.grant_set = GrantSetV1::new(Vec::new());
        let none = compile_transport_permissions(&state, &binding).unwrap();
        assert!(!none
            .publish
            .iter()
            .any(|subject| subject.contains("AUTH_BROWSER_FLOWS")));

        state.grant_set = GrantSetV1::new(vec![kv_atom(PermissionActionV1::Read)]);
        let read = compile_transport_permissions(&state, &binding).unwrap();
        assert!(read
            .publish
            .contains(&"$JS.API.DIRECT.GET.KV_AUTH_BROWSER_FLOWS".to_owned()));
        assert!(!read
            .publish
            .contains(&"$KV.AUTH_BROWSER_FLOWS.>".to_owned()));

        state.grant_set = GrantSetV1::new(vec![kv_atom(PermissionActionV1::Write)]);
        let write = compile_transport_permissions(&state, &binding).unwrap();
        assert!(write
            .publish
            .contains(&"$KV.AUTH_BROWSER_FLOWS.>".to_owned()));
        assert!(!write
            .publish
            .iter()
            .any(|subject| subject.starts_with("$JS.API.DIRECT.GET.KV_AUTH_BROWSER_FLOWS")));

        state.grant_set = GrantSetV1::new(Vec::new());
        let reduced = compile_transport_permissions(&state, &binding).unwrap();
        assert!(!reduced
            .publish
            .iter()
            .any(|subject| subject.contains("AUTH_BROWSER_FLOWS")));
    }

    #[test]
    fn resource_atom_for_another_participant_fails_closed() {
        let (binding, mut state) = fixture();
        state.resource_bindings = vec![kv_binding("trellis-auth-runtime")];
        state.grant_set = GrantSetV1::new(vec![PermissionAtomV1::new(
            PermissionTargetV1::participant_resource(
                "other-participant",
                ParticipantResourceKindV1::Kv,
                "browserFlows",
            )
            .unwrap(),
            PermissionActionV1::Read,
        )
        .unwrap()]);
        assert!(compile_transport_permissions(&state, &binding).is_err());
    }

    fn kv_atom(action: PermissionActionV1) -> PermissionAtomV1 {
        PermissionAtomV1::new(
            PermissionTargetV1::participant_resource(
                "trellis-auth-runtime",
                ParticipantResourceKindV1::Kv,
                "browserFlows",
            )
            .unwrap(),
            action,
        )
        .unwrap()
    }

    fn kv_binding(owner: &str) -> ResourceBindingEvidence {
        ResourceBindingEvidence {
            resource_kind: "kv".to_owned(),
            local_name: "browserFlows".to_owned(),
            binding_id: "binding-browser-flows".to_owned(),
            owner_participant_id: owner.to_owned(),
            provider_identity: ResourceProviderIdentity::Kv {
                bucket: "AUTH_BROWSER_FLOWS".to_owned(),
            },
            state: ResourceBindingState::Available,
            materialized_at: 1,
            error: None,
        }
    }

    fn fixture() -> (ParticipantBindingRecord, IssuableAuthorizationState) {
        let api_value: Value =
            serde_json::from_str(include_str!("../../../trellis.api.json")).expect("auth API JSON");
        let participant_value: Value =
            serde_json::from_str(include_str!("../../../trellis.participant.json"))
                .expect("auth participant JSON");
        let api = parse_api_v1(&api_value).expect("auth API");
        let participant = parse_participant_v1(&participant_value).expect("auth participant");
        let apis = std::collections::BTreeMap::from([(api.id().to_owned(), api.clone())]);
        let resolved = resolve_participant_v1(&participant, &apis).expect("resolved participant");
        let binding = ParticipantBindingRecord {
            participant_id: participant.id().to_owned(),
            participant_kind: participant.kind(),
            artifact_digest: participant.digest().unwrap(),
            needs_digest: resolved.needs().digest().unwrap(),
            participant_json: participant.canonical_json().unwrap(),
            api_artifacts_json: serde_json::to_string(&std::collections::BTreeMap::from([(
                api.id().to_owned(),
                api_value,
            )]))
            .unwrap(),
            resolved_at: 1,
            state: ParticipantBindingState::Resolved,
            error: None,
        };
        let state = IssuableAuthorizationState {
            principal: AuthorizationPrincipalV1 {
                kind: AuthorizationPrincipalKindV1::Service,
                id: "svc_auth".to_owned(),
            },
            session_id: "ses_01".to_owned(),
            session_public_key: "session-key".to_owned(),
            session_key_id: "session-key-id".to_owned(),
            inbox_prefix: "_INBOX.session".to_owned(),
            participant: AuthorizationParticipantV1 {
                kind: binding.participant_kind,
                id: binding.participant_id.clone(),
                artifact_digest: binding.artifact_digest.clone(),
                needs_digest: binding.needs_digest.clone(),
            },
            authority_ref: AuthorizationAuthorityRefV1 {
                kind: AuthorizationAuthorityKindV1::Deployment,
                id: "dpa_auth".to_owned(),
                version: 1,
            },
            deployment_id: Some("dep_auth".to_owned()),
            instance_id: Some("ins_auth".to_owned()),
            grant_set: GrantSetV1::new(Vec::new()),
            resource_bindings: Vec::new(),
            capabilities: vec!["service".to_owned()],
            session_expires_at: None,
            effective_authority_expires_at: None,
            delegation_expires_at: None,
            materialization_version: 1,
        };
        let _ = AuthorityKind::Deployment;
        (binding, state)
    }
}
