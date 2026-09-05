use crate::{
    ast::{
        Api, Constraint, ConstraintValue, Docs, Participant, Project, SchemaDecl, Spanned, Surface,
        Transfer, Type,
    },
    parser::diagnostic,
};
use miette::IntoDiagnostic;
use serde_json::{json, Map, Value};
use std::collections::{BTreeMap, BTreeSet};
use trellis_protocol::{
    lint_api_authoring, lint_participant_authoring, parse_api, parse_participant,
    resolve_participant, ApiArtifact, ParticipantArtifact,
};

pub(crate) fn apis(project: &Project) -> miette::Result<BTreeMap<String, ApiArtifact>> {
    let mut compiled = BTreeMap::new();
    for declaration in &project.apis {
        if compiled.contains_key(&declaration.value.id) {
            return Err(at(
                project,
                declaration,
                format!("duplicate API declaration '{}'", declaration.value.id),
            ));
        }
        let value = api_value(project, declaration)?;
        lint_api_authoring(&value).map_err(|error| at(project, declaration, error.to_string()))?;
        let artifact =
            parse_api(&value).map_err(|error| at(project, declaration, error.to_string()))?;
        compiled.insert(artifact.id().to_owned(), artifact);
    }
    Ok(compiled)
}

pub(crate) fn participants(
    project: &Project,
    apis: &BTreeMap<String, ApiArtifact>,
) -> miette::Result<Vec<ParticipantArtifact>> {
    let mut ids = BTreeSet::new();
    let mut compiled = Vec::new();
    for declaration in &project.participants {
        if !ids.insert(declaration.value.id.clone()) {
            return Err(at(
                project,
                declaration,
                format!(
                    "duplicate participant declaration '{}'",
                    declaration.value.id
                ),
            ));
        }
        let value = participant_value(project, declaration, apis)?;
        lint_participant_authoring(&value)
            .map_err(|error| at(project, declaration, error.to_string()))?;
        let artifact = parse_participant(&value)
            .map_err(|error| at(project, declaration, error.to_string()))?;
        resolve_participant(&artifact, apis)
            .map_err(|error| at(project, declaration, error.to_string()))?;
        compiled.push(artifact);
    }
    Ok(compiled)
}

fn api_value(project: &Project, declaration: &Spanned<Api>) -> miette::Result<Value> {
    let api = &declaration.value;
    let version = required(project, declaration, api.version.as_ref(), "version")?;
    semver::Version::parse(&version.value)
        .into_diagnostic()
        .map_err(|error| at(project, version, format!("invalid API version: {error}")))?;
    let display_name = required(
        project,
        declaration,
        api.display_name.as_ref(),
        "display_name",
    )?;
    let description = required(
        project,
        declaration,
        api.description.as_ref(),
        "description",
    )?;
    let mut schemas = Map::new();
    for (name, schema) in &api.schemas {
        schemas.insert(
            name.clone(),
            lower_schema(project, &api.schemas, schema, &mut vec![name.clone()])?,
        );
    }

    for exported in &api.exports {
        require_schema(project, api, exported)?;
    }
    let mut value = json!({
        "format": "trellis.api.v1",
        "id": api.id,
        "version": version.value,
        "displayName": display_name.value,
        "description": description.value,
        "schemas": schemas,
        "exports": {"schemas": api.exports.iter().map(|item| item.value.clone()).collect::<Vec<_>>()},
    });
    let object = value.as_object_mut().expect("API is an object");
    insert_docs(object, &api.docs);
    insert_nonempty(
        object,
        "errors",
        api.errors
            .iter()
            .map(|(name, declaration)| {
                let mut value = Map::new();
                if let Some(code) = &declaration.value.code {
                    value.insert("code".to_owned(), json!(code.value));
                }
                if let Some(schema) = &declaration.value.schema {
                    value.insert("schema".to_owned(), json!({"schema": schema.value}));
                }
                (name.clone(), Value::Object(value))
            })
            .collect(),
    );
    for declaration in api.errors.values() {
        if let Some(schema) = &declaration.value.schema {
            require_schema(project, api, schema)?;
        }
    }
    let mut capability_allows = BTreeMap::<String, Vec<Value>>::new();
    insert_surfaces(
        project,
        api,
        &mut capability_allows,
        object,
        "rpc",
        &api.rpcs,
    )?;
    insert_surfaces(
        project,
        api,
        &mut capability_allows,
        object,
        "operations",
        &api.operations,
    )?;
    insert_surfaces(
        project,
        api,
        &mut capability_allows,
        object,
        "events",
        &api.events,
    )?;
    insert_surfaces(
        project,
        api,
        &mut capability_allows,
        object,
        "feeds",
        &api.feeds,
    )?;
    let mut capabilities = Map::new();
    for (name, declaration) in &api.capabilities {
        let capability = &declaration.value;
        let mut value = Map::new();
        insert_option(
            &mut value,
            "displayName",
            &capability.display_name.as_ref().map(|v| v.value.clone()),
        );
        insert_option(
            &mut value,
            "description",
            &capability.description.as_ref().map(|v| v.value.clone()),
        );
        insert_option(
            &mut value,
            "consequence",
            &capability.consequence.as_ref().map(|v| v.value.clone()),
        );
        if let Some(allows) = capability_allows.remove(name) {
            value.insert("allows".to_owned(), Value::Array(allows));
        }
        capabilities.insert(name.clone(), Value::Object(value));
    }
    for (name, allows) in capability_allows {
        capabilities.insert(name, json!({"allows": allows}));
    }
    insert_nonempty(object, "capabilities", capabilities);
    Ok(value)
}

fn insert_surfaces(
    project: &Project,
    api: &Api,
    capabilities: &mut BTreeMap<String, Vec<Value>>,
    target: &mut Map<String, Value>,
    section: &str,
    surfaces: &BTreeMap<String, Spanned<Surface>>,
) -> miette::Result<()> {
    let mut values = Map::new();
    for (name, declaration) in surfaces {
        values.insert(
            name.clone(),
            surface_value(project, api, capabilities, section, name, declaration)?,
        );
    }
    if !values.is_empty() {
        target.insert(section.to_owned(), Value::Object(values));
    }
    Ok(())
}

fn surface_value(
    project: &Project,
    api: &Api,
    capabilities: &mut BTreeMap<String, Vec<Value>>,
    section: &str,
    name: &str,
    declaration: &Spanned<Surface>,
) -> miette::Result<Value> {
    let surface = &declaration.value;
    let version = required(project, declaration, surface.version.as_ref(), "version")?;
    let mut object = Map::new();
    object.insert("version".to_owned(), json!(version.value));
    if let Some(subject) = &surface.subject {
        object.insert("subject".to_owned(), json!(subject.value));
    }
    if let Some(class) = &surface.class {
        object.insert("class".to_owned(), json!(class.value));
    }
    match section {
        "rpc" => {
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "input",
                &surface.input,
            )?;
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "output",
                &surface.output,
            )?;
            if let Some(Transfer::Receive) = surface.transfer {
                object.insert("transfer".to_owned(), json!({"direction": "receive"}));
            } else if surface.transfer.is_some() {
                return Err(at(project, declaration, "RPC transfers must be 'receive'"));
            }
        }
        "operations" => {
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "input",
                &surface.input,
            )?;
            insert_optional_schema_ref(project, api, &mut object, "progress", &surface.progress)?;
            insert_optional_schema_ref(project, api, &mut object, "output", &surface.output)?;
            if let Some(Transfer::Send) = surface.transfer {
                object.insert("transfer".to_owned(), json!({"direction": "send"}));
            } else if surface.transfer.is_some() {
                return Err(at(
                    project,
                    declaration,
                    "operation transfers must be 'send'",
                ));
            }
            if surface.cancellable {
                object.insert("cancel".to_owned(), Value::Bool(true));
            }
        }
        "events" => {
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "event",
                &surface.event,
            )?;
            if !surface.params.is_empty() {
                object.insert("params".to_owned(), json!(surface.params));
            }
        }
        "feeds" => {
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "input",
                &surface.input,
            )?;
            insert_schema_ref(
                project,
                api,
                declaration,
                &mut object,
                "event",
                &surface.event,
            )?;
        }
        _ => unreachable!("known surface section"),
    }
    if !surface.errors.is_empty() {
        for error in &surface.errors {
            if !api.errors.contains_key(&error.value) {
                return Err(at(
                    project,
                    error,
                    format!("unknown error '{}'", error.value),
                ));
            }
        }
        object.insert(
            "errors".to_owned(),
            json!(surface
                .errors
                .iter()
                .map(|error| error.value.clone())
                .collect::<Vec<_>>()),
        );
    }
    insert_docs(&mut object, &surface.docs);
    let surface_kind = match section {
        "operations" => "operation",
        "events" => "event",
        "feeds" => "feed",
        _ => "rpc",
    };
    for (source_action, names) in &surface.capabilities {
        let action = match (section, source_action.as_str()) {
            ("rpc", "call") => "call",
            ("operations", "call") => "invoke",
            ("operations", "observe") => "observe",
            ("operations", "cancel") if surface.cancellable => "cancel",
            ("events", "publish") => "publish",
            ("events", "subscribe") => "subscribe",
            ("feeds", "subscribe") => "subscribe",
            _ => {
                return Err(at(
                    project,
                    declaration,
                    format!("capability action '{source_action}' is invalid for {surface_kind}"),
                ));
            }
        };
        for capability in names {
            capabilities
                .entry(capability.clone())
                .or_default()
                .push(json!({
                    "action": action,
                    "target": {"kind": "apiSurface", "api": api.id, "surface": surface_kind, "name": name}
                }));
        }
    }
    Ok(Value::Object(object))
}

fn participant_value(
    project: &Project,
    declaration: &Spanned<Participant>,
    apis: &BTreeMap<String, ApiArtifact>,
) -> miette::Result<Value> {
    let participant = &declaration.value;
    if !matches!(
        participant.kind.as_str(),
        "service" | "device" | "app" | "agent"
    ) {
        return Err(at(
            project,
            declaration,
            format!("unknown participant kind '{}'", participant.kind),
        ));
    }
    let implemented = participant
        .implements
        .first()
        .ok_or_else(|| at(project, declaration, "participant must implement an API"))?;
    if participant.implements.len() != 1 {
        return Err(at(
            project,
            declaration,
            "participant must implement exactly one API",
        ));
    }
    let api = apis.get(&implemented.value).ok_or_else(|| {
        at(
            project,
            implemented,
            format!("unknown implemented API '{}'", implemented.value),
        )
    })?;
    let api_value = api.normalized_value().into_diagnostic()?;
    let mut schemas = if participant.schemas.is_empty() {
        api_value["schemas"]
            .as_object()
            .cloned()
            .unwrap_or_default()
    } else {
        Map::new()
    };
    for (name, schema) in &participant.schemas {
        schemas.insert(
            name.clone(),
            lower_schema(
                project,
                &participant.schemas,
                schema,
                &mut vec![name.clone()],
            )?,
        );
    }
    let mut implementation = json!({"api": api.id(), "apiDigest": api.digest().into_diagnostic()?});
    if !participant.bindings.is_empty() {
        let mut transfers = Map::new();
        for (name, binding) in &participant.bindings {
            if api_value["operations"].get(name).is_none() {
                return Err(at(
                    project,
                    binding,
                    format!("unknown implemented operation '{name}'"),
                ));
            }
            let store = required(project, binding, binding.value.store.as_ref(), "store")?;
            if !participant.stores.contains_key(&store.value) {
                return Err(at(
                    project,
                    store,
                    format!("unknown store '{}'", store.value),
                ));
            }
            let key = binding
                .value
                .key
                .as_ref()
                .ok_or_else(|| at(project, binding, "transfer requires 'key'"))?;
            let mut transfer = json!({"store": store.value, "key": key});
            let object = transfer.as_object_mut().expect("transfer is an object");
            insert_option(object, "contentType", &binding.value.content_type);
            insert_option(object, "metadata", &binding.value.metadata);
            if let Some(expires) = binding.value.expires_in_ms {
                object.insert("expiresInMs".to_owned(), json!(expires));
            }
            transfers.insert(name.clone(), transfer);
        }
        implementation["operationTransfers"] = Value::Object(transfers);
    }
    let display_name = api_value["displayName"]
        .as_str()
        .expect("validated API displayName");
    let description = api_value["description"]
        .as_str()
        .expect("validated API description");
    let mut value = json!({
        "format": "trellis.participant.v1",
        "id": participant.id,
        "displayName": display_name,
        "description": description,
        "kind": participant.kind,
        "schemas": schemas,
        "implements": {"self": implementation},
    });
    let object = value.as_object_mut().expect("participant is an object");
    let mut subscribed_events = Map::new();
    if !participant.subscribed_events.is_empty() {
        subscribed_events.insert(
            "self".to_owned(),
            Value::Array(
                participant
                    .subscribed_events
                    .iter()
                    .map(|event| {
                        json!(canonical_selection_name(&api_value, "events", &event.value))
                    })
                    .collect(),
            ),
        );
    }
    if !participant.uses.is_empty() || !participant.state.is_empty() {
        let mut required_uses = Map::new();
        let mut optional_uses = Map::new();
        for (alias, declaration) in &participant.uses {
            let used = &declaration.value;
            let referenced = apis.get(&used.api.value).ok_or_else(|| {
                at(
                    project,
                    &used.api,
                    format!("unknown used API '{}'", used.api.value),
                )
            })?;
            let mut use_value = json!({
                "api": referenced.id(),
                "apiDigest": referenced.digest().into_diagnostic()?,
            });
            let referenced_value = referenced.normalized_value().into_diagnostic()?;
            let use_object = use_value.as_object_mut().expect("API use is an object");
            for selection in &used.selections {
                let selection = &selection.value;
                let section = match selection.surface.as_str() {
                    "rpc" => "rpc",
                    "operation" => "operations",
                    "event" => "events",
                    "feed" => "feeds",
                    "state" => "state",
                    _ => unreachable!("parser validates selection surfaces"),
                };
                let section_value = use_object
                    .entry(section)
                    .or_insert_with(|| Value::Object(Map::new()));
                let section_object = section_value
                    .as_object_mut()
                    .expect("selection section is an object");
                let name = canonical_selection_name(&referenced_value, section, &selection.name);
                if selection.action == "control" {
                    section_object
                        .entry("control")
                        .or_insert_with(|| Value::Object(Map::new()))
                        .as_object_mut()
                        .expect("control selection is an object")
                        .entry(&name)
                        .or_insert_with(|| Value::Array(Vec::new()))
                        .as_array_mut()
                        .expect("signal selection is an array")
                        .push(json!(selection.signal));
                } else {
                    section_object
                        .entry(&selection.action)
                        .or_insert_with(|| Value::Array(Vec::new()))
                        .as_array_mut()
                        .expect("selection is an array")
                        .push(json!(name));
                    if section == "events" && selection.action == "subscribe" {
                        subscribed_events
                            .entry(alias)
                            .or_insert_with(|| Value::Array(Vec::new()))
                            .as_array_mut()
                            .expect("subscribed events are an array")
                            .push(json!(name));
                    }
                }
            }
            if used.required {
                required_uses.insert(alias.clone(), use_value);
            } else {
                optional_uses.insert(alias.clone(), use_value);
            }
        }
        if !participant.state.is_empty() {
            let state_api = apis.get("trellis.state@v1").ok_or_else(|| {
                at(
                    project,
                    declaration,
                    "State requires the 'trellis.state@v1' API dependency",
                )
            })?;
            let alias = participant
                .uses
                .iter()
                .find(|(_, used)| used.value.api.value == "trellis.state@v1")
                .map(|(alias, _)| alias.as_str())
                .unwrap_or("state");
            if let Some(existing) = required_uses
                .get(alias)
                .or_else(|| optional_uses.get(alias))
            {
                if existing["api"] != "trellis.state@v1" {
                    return Err(at(
                        project,
                        declaration,
                        "State baseline alias 'state' is already in use",
                    ));
                }
            }
            let state_digest = state_api.digest().into_diagnostic()?;
            let mut baseline = required_uses
                .remove(alias)
                .or_else(|| optional_uses.remove(alias))
                .unwrap_or_else(|| json!({"api": state_api.id(), "apiDigest": state_digest}));
            let rpc = baseline
                .as_object_mut()
                .expect("API use")
                .entry("rpc")
                .or_insert_with(|| json!({}));
            let calls = rpc
                .as_object_mut()
                .expect("RPC selection")
                .entry("call")
                .or_insert_with(|| json!([]))
                .as_array_mut()
                .expect("RPC calls");
            for name in ["State.Get", "State.Put", "State.Delete", "State.List"] {
                if !calls.iter().any(|call| call == name) {
                    calls.push(json!(name));
                }
            }
            required_uses.insert(alias.to_owned(), baseline);
        }
        let mut uses = Map::new();
        insert_nonempty(&mut uses, "required", required_uses);
        insert_nonempty(&mut uses, "optional", optional_uses);
        object.insert("uses".to_owned(), Value::Object(uses));
    }
    if participant.kind == "service" && !subscribed_events.is_empty() {
        object.insert(
            "eventConsumers".to_owned(),
            json!({"events": {"events": subscribed_events}}),
        );
    }
    if let Some(docs) = api_value.get("docs") {
        object.insert("docs".to_owned(), docs.clone());
    }
    let mut state = Map::new();
    for (name, declaration) in &participant.state {
        let declaration_value = &declaration.value;
        if !matches!(declaration_value.kind.as_str(), "value" | "map") {
            return Err(at(
                project,
                declaration,
                format!("unknown state kind '{}'", declaration_value.kind),
            ));
        }
        let schema = if schemas.contains_key(&declaration_value.schema.value) {
            declaration_value.schema.value.clone()
        } else {
            require_api_schema(project, &api_value, &declaration_value.schema)?;
            canonical_selection_name(&api_value, "schemas", &declaration_value.schema.value)
        };
        let mut state_value = json!({
            "kind": declaration_value.kind,
            "schema": {"schema": schema},
        });
        let state_object = state_value.as_object_mut().expect("state is an object");
        insert_option(
            state_object,
            "stateVersion",
            &declaration_value.state_version,
        );
        insert_docs(state_object, &declaration_value.docs);
        state.insert(name.clone(), state_value);
    }
    if !state.is_empty() {
        object.insert("state".to_owned(), Value::Object(state));
    }
    let mut resources = Map::new();
    let mut stores = Map::new();
    for (name, declaration) in &participant.stores {
        let resource = &declaration.value;
        let purpose = resource
            .purpose
            .as_ref()
            .ok_or_else(|| at(project, declaration, "store requires 'purpose'"))?;
        let mut value = json!({"purpose": purpose});
        let object = value.as_object_mut().expect("resource is an object");
        insert_number(object, "ttlMs", resource.ttl_ms);
        insert_number(object, "maxObjectBytes", resource.max_object_bytes);
        insert_number(object, "maxTotalBytes", resource.max_total_bytes);
        insert_number(object, "maxValueBytes", resource.max_value_bytes);
        insert_docs(object, &resource.docs);
        stores.insert(name.clone(), value);
    }
    if !stores.is_empty() {
        resources.insert("store".to_owned(), Value::Object(stores));
    }
    let mut kv = Map::new();
    for (name, declaration) in &participant.kv {
        let resource = &declaration.value;
        let purpose = resource
            .purpose
            .as_ref()
            .ok_or_else(|| at(project, declaration, "KV resource requires 'purpose'"))?;
        let schema = required(project, declaration, resource.schema.as_ref(), "schema")?;
        let schema = if schemas.contains_key(&schema.value) {
            schema.value.clone()
        } else {
            require_api_schema(project, &api_value, schema)?;
            canonical_selection_name(&api_value, "schemas", &schema.value)
        };
        let mut value = json!({"purpose": purpose, "schema": {"schema": schema}});
        let object = value.as_object_mut().expect("resource is an object");
        insert_number(object, "history", resource.history);
        insert_number(object, "ttlMs", resource.ttl_ms);
        insert_docs(object, &resource.docs);
        kv.insert(name.clone(), value);
    }
    if !kv.is_empty() {
        resources.insert("kv".to_owned(), Value::Object(kv));
    }
    if !resources.is_empty() {
        object.insert("resources".to_owned(), Value::Object(resources));
    }
    let mut jobs = Map::new();
    for (name, declaration) in &participant.jobs {
        let resource = &declaration.value;
        let payload = required(project, declaration, resource.payload.as_ref(), "payload")?;
        let payload_name = if schemas.contains_key(&payload.value) {
            payload.value.clone()
        } else {
            require_api_schema(project, &api_value, payload)?;
            canonical_selection_name(&api_value, "schemas", &payload.value)
        };
        let mut value = json!({"payload": {"schema": payload_name}});
        if let Some(result) = &resource.result {
            let result_name = if schemas.contains_key(&result.value) {
                result.value.clone()
            } else {
                require_api_schema(project, &api_value, result)?;
                canonical_selection_name(&api_value, "schemas", &result.value)
            };
            value["result"] = json!({"schema": result_name});
        }
        insert_docs(
            value.as_object_mut().expect("job is an object"),
            &resource.docs,
        );
        jobs.insert(name.clone(), value);
    }
    if !jobs.is_empty() {
        object.insert("jobQueues".to_owned(), Value::Object(jobs));
    }
    Ok(value)
}

fn canonical_selection_name(api: &Value, section: &str, authored: &str) -> String {
    api[section]
        .as_object()
        .and_then(|entries| {
            entries
                .keys()
                .find(|name| *name == authored || authored.ends_with(&format!(".{name}")))
        })
        .cloned()
        .unwrap_or_else(|| authored.to_owned())
}

fn lower_schema(
    project: &Project,
    schemas: &BTreeMap<String, Spanned<SchemaDecl>>,
    declaration: &Spanned<SchemaDecl>,
    stack: &mut Vec<String>,
) -> miette::Result<Value> {
    match &declaration.value {
        SchemaDecl::Model(fields) => {
            let mut properties = Map::new();
            let mut required = Vec::new();
            let mut names = BTreeSet::new();
            for field in fields {
                if !names.insert(&field.name) {
                    return Err(at(
                        project,
                        &field.ty,
                        format!("duplicate field '{}'", field.name),
                    ));
                }
                properties.insert(
                    field.name.clone(),
                    lower_type(project, schemas, &field.ty, stack)?,
                );
                if !field.optional {
                    required.push(Value::String(field.name.clone()));
                }
            }
            let mut schema = json!({"type": "object", "properties": properties});
            if !required.is_empty() {
                schema["required"] = Value::Array(required);
            }
            Ok(schema)
        }
        SchemaDecl::Alias(ty) => lower_type(
            project,
            schemas,
            &Spanned {
                value: ty.clone(),
                source: declaration.source,
                span: declaration.span.clone(),
            },
            stack,
        ),
        SchemaDecl::Enum(values) => Ok(json!({
            "anyOf": values.iter().map(|value| json!({"type": "string", "const": value})).collect::<Vec<_>>()
        })),
    }
}

fn lower_type(
    project: &Project,
    schemas: &BTreeMap<String, Spanned<SchemaDecl>>,
    ty: &Spanned<Type>,
    stack: &mut Vec<String>,
) -> miette::Result<Value> {
    match &ty.value {
        Type::Json => Ok(json!({})),
        Type::Named(name) => {
            if stack.contains(name) {
                return Err(at(
                    project,
                    ty,
                    format!("recursive schema reference '{}' is not supported", name),
                ));
            }
            let declaration = schemas
                .get(name)
                .ok_or_else(|| at(project, ty, format!("unknown schema reference '{name}'")))?;
            stack.push(name.clone());
            let value = lower_schema(project, schemas, declaration, stack);
            stack.pop();
            value
        }
        Type::String(constraints) => {
            constrained(json!({"type": "string"}), constraints, ty, project)
        }
        Type::Bool => Ok(json!({"type": "boolean"})),
        Type::BoolLiteral(value) => Ok(json!({"type": "boolean", "const": value})),
        Type::Integer {
            unsigned,
            constraints,
        } => {
            let mut value = json!({"type": "integer"});
            if *unsigned {
                value["minimum"] = json!(0);
            }
            constrained(value, constraints, ty, project)
        }
        Type::Number(constraints) => {
            constrained(json!({"type": "number"}), constraints, ty, project)
        }
        Type::List {
            member,
            constraints,
        } => constrained(
            json!({"type": "array", "items": lower_type(project, schemas, member, stack)?}),
            constraints,
            ty,
            project,
        ),
        Type::Map(member) => Ok(json!({
            "type": "object",
            "additionalProperties": lower_type(project, schemas, member, stack)?
        })),
        Type::Literal(value) => Ok(json!({"type": "string", "const": value})),
        Type::Null => Ok(json!({"type": "null"})),
        Type::Union(members) => Ok(json!({
            "anyOf": members.iter().map(|member| lower_type(project, schemas, member, stack)).collect::<miette::Result<Vec<_>>>()?
        })),
    }
}

fn constrained(
    mut value: Value,
    constraints: &[Constraint],
    ty: &Spanned<Type>,
    project: &Project,
) -> miette::Result<Value> {
    let object = value.as_object_mut().expect("schema is an object");
    for constraint in constraints {
        let keyword = match constraint.name.as_str() {
            "minimum" => "minimum",
            "maximum" => "maximum",
            "min_length" => "minLength",
            "max_length" => "maxLength",
            "pattern" => "pattern",
            "format" => "format",
            "min_items" => "minItems",
            "max_items" => "maxItems",
            other => return Err(at(project, ty, format!("unsupported constraint '{other}'"))),
        };
        object.insert(
            keyword.to_owned(),
            match &constraint.value {
                ConstraintValue::Integer(value) => json!(value),
                ConstraintValue::String(value) => json!(value),
            },
        );
    }
    Ok(value)
}

fn require_schema(project: &Project, api: &Api, reference: &Spanned<String>) -> miette::Result<()> {
    if api.schemas.contains_key(&reference.value) {
        Ok(())
    } else {
        Err(at(
            project,
            reference,
            format!("unknown schema reference '{}'", reference.value),
        ))
    }
}

fn require_api_schema(
    project: &Project,
    api: &Value,
    reference: &Spanned<String>,
) -> miette::Result<()> {
    if api["schemas"].get(&reference.value).is_some() {
        Ok(())
    } else {
        Err(at(
            project,
            reference,
            format!("unknown schema reference '{}'", reference.value),
        ))
    }
}

fn insert_schema_ref<T>(
    project: &Project,
    api: &Api,
    declaration: &Spanned<T>,
    target: &mut Map<String, Value>,
    key: &str,
    reference: &Option<Spanned<String>>,
) -> miette::Result<()> {
    let reference = required(project, declaration, reference.as_ref(), key)?;
    require_schema(project, api, reference)?;
    target.insert(key.to_owned(), json!({"schema": reference.value}));
    Ok(())
}

fn insert_optional_schema_ref(
    project: &Project,
    api: &Api,
    target: &mut Map<String, Value>,
    key: &str,
    reference: &Option<Spanned<String>>,
) -> miette::Result<()> {
    if let Some(reference) = reference {
        require_schema(project, api, reference)?;
        target.insert(key.to_owned(), json!({"schema": reference.value}));
    }
    Ok(())
}

fn required<'a, T, U>(
    project: &Project,
    declaration: &Spanned<T>,
    value: Option<&'a Spanned<U>>,
    name: &str,
) -> miette::Result<&'a Spanned<U>> {
    value.ok_or_else(|| at(project, declaration, format!("missing required '{name}'")))
}

fn insert_docs(target: &mut Map<String, Value>, docs: &Option<Docs>) {
    let Some(docs) = docs else { return };
    let Some(markdown) = &docs.markdown else {
        return;
    };
    let mut value = json!({"markdown": markdown});
    if let Some(summary) = &docs.summary {
        value["summary"] = json!(summary);
    }
    target.insert("docs".to_owned(), value);
}

fn insert_nonempty(target: &mut Map<String, Value>, key: &str, value: Map<String, Value>) {
    if !value.is_empty() {
        target.insert(key.to_owned(), Value::Object(value));
    }
}

fn insert_option(target: &mut Map<String, Value>, key: &str, value: &Option<String>) {
    if let Some(value) = value {
        target.insert(key.to_owned(), json!(value));
    }
}

fn insert_number(target: &mut Map<String, Value>, key: &str, value: Option<u64>) {
    if let Some(value) = value {
        target.insert(key.to_owned(), json!(value));
    }
}

fn at<T>(project: &Project, value: &Spanned<T>, message: impl Into<String>) -> miette::Report {
    diagnostic(&project.sources[value.source], value.span.clone(), message)
}
