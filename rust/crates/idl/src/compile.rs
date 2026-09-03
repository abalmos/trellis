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
            lower_schema(project, api, schema, &mut vec![name.clone()])?,
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
            .keys()
            .map(|name| (name.clone(), json!({})))
            .collect(),
    );
    let mut capabilities = BTreeMap::<String, Vec<Value>>::new();
    insert_surfaces(project, api, &mut capabilities, object, "rpc", &api.rpcs)?;
    insert_surfaces(
        project,
        api,
        &mut capabilities,
        object,
        "operations",
        &api.operations,
    )?;
    insert_surfaces(
        project,
        api,
        &mut capabilities,
        object,
        "events",
        &api.events,
    )?;
    insert_surfaces(project, api, &mut capabilities, object, "feeds", &api.feeds)?;
    insert_nonempty(
        object,
        "capabilities",
        capabilities
            .into_iter()
            .map(|(name, allows)| (name, json!({"allows": allows})))
            .collect(),
    );
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
            "Gate 1 service participants implement exactly one API",
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
        "schemas": api_value["schemas"],
        "implements": {"self": implementation},
    });
    let object = value.as_object_mut().expect("participant is an object");
    if let Some(docs) = api_value.get("docs") {
        object.insert("docs".to_owned(), docs.clone());
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
        require_api_schema(project, &api_value, schema)?;
        let mut value = json!({"purpose": purpose, "schema": {"schema": schema.value}});
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
        require_api_schema(project, &api_value, payload)?;
        let mut value = json!({"payload": {"schema": payload.value}});
        if let Some(result) = &resource.result {
            require_api_schema(project, &api_value, result)?;
            value["result"] = json!({"schema": result.value});
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

fn lower_schema(
    project: &Project,
    api: &Api,
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
                    lower_type(project, api, &field.ty, stack)?,
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
            api,
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
    api: &Api,
    ty: &Spanned<Type>,
    stack: &mut Vec<String>,
) -> miette::Result<Value> {
    match &ty.value {
        Type::Named(name) => {
            if stack.contains(name) {
                return Err(at(
                    project,
                    ty,
                    format!("recursive schema reference '{}' is not supported", name),
                ));
            }
            let declaration = api
                .schemas
                .get(name)
                .ok_or_else(|| at(project, ty, format!("unknown schema reference '{name}'")))?;
            stack.push(name.clone());
            let value = lower_schema(project, api, declaration, stack);
            stack.pop();
            value
        }
        Type::String(constraints) => {
            constrained(json!({"type": "string"}), constraints, ty, project)
        }
        Type::Bool => Ok(json!({"type": "boolean"})),
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
        Type::List(member) => {
            Ok(json!({"type": "array", "items": lower_type(project, api, member, stack)?}))
        }
        Type::Map(member) => Ok(json!({
            "type": "object",
            "additionalProperties": lower_type(project, api, member, stack)?
        })),
        Type::Literal(value) => Ok(json!({"type": "string", "const": value})),
        Type::Null => Ok(json!({"type": "null"})),
        Type::Union(members) => Ok(json!({
            "anyOf": members.iter().map(|member| lower_type(project, api, member, stack)).collect::<miette::Result<Vec<_>>>()?
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
