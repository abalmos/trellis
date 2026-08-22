from pathlib import Path


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


# After protocol scoping is removed, subject resolvers are pure identity wrappers.
# Delete the indirection rather than keeping compatibility shims.
path = "rust/crates/trellis/src/client/connection.rs"
text = read(path)
old = '''    pub(crate) fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: expected one identity descriptor_subject, found {text.count(old)}")
text = text.replace(old, "", 1)
text = text.replace(".request_json(&self.descriptor_subject(D::SUBJECT), value)", ".request_json(D::SUBJECT, value)")
text = text.replace(".request_json(&self.descriptor_subject(D::SUBJECT), input)", ".request_json(D::SUBJECT, input)")
old = "        let prepared = prepare_event::<D>(event)?.with_subject(self.descriptor_subject(D::SUBJECT));\n"
new = "        let prepared = prepare_event::<D>(event)?;\n"
if text.count(old) != 1:
    raise RuntimeError(f"{path}: typed event publish subject overwrite changed")
text = text.replace(old, new, 1)
old = '''    pub async fn publish_prepared(
        &self,
        event: &PreparedTrellisEvent,
    ) -> Result<(), TrellisClientError> {
        let event = event
            .clone()
            .with_subject(self.descriptor_subject(event.subject()));
        let context_digest = self.authorization_context_digest()?;
        publish_prepared_event(
            &self.nats,
            &self.auth,
            &context_digest,
            self.timeout_ms,
            &event,
        )
        .await
    }
'''
new = '''    pub async fn publish_prepared(
        &self,
        event: &PreparedTrellisEvent,
    ) -> Result<(), TrellisClientError> {
        let context_digest = self.authorization_context_digest()?;
        publish_prepared_event(
            &self.nats,
            &self.auth,
            &context_digest,
            self.timeout_ms,
            event,
        )
        .await
    }
'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: prepared event subject rewrite shape changed")
text = text.replace(old, new, 1)
text = text.replace(".subscribe(self.descriptor_subject(D::SUBSCRIBE_SUBJECT))", ".subscribe(D::SUBSCRIBE_SUBJECT)")
text = text.replace(
    "event_consumer_config(&options, self.descriptor_subject(D::SUBSCRIBE_SUBJECT))",
    "event_consumer_config(&options, D::SUBSCRIBE_SUBJECT.to_owned())",
)
text = text.replace("let subject = self.descriptor_subject(D::SUBJECT);", "let subject = D::SUBJECT.to_owned();")
old = '''impl OperationTransport for TrellisClient {
    fn descriptor_subject(&self, subject: &str) -> String {
        self.descriptor_subject(subject)
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: OperationTransport resolver shape changed")
text = text.replace(old, "impl OperationTransport for TrellisClient {\n", 1)
write(path, text)

# Prepared events already carry their exact concrete subject. The private
# subject-rewrite helper existed only for integration-test scoping and is now
# both unnecessary and incorrect for descriptor subjects with dynamic tokens.
path = "rust/crates/trellis/src/client/events.rs"
text = read(path)
old = '''impl PreparedTrellisEvent {
    pub(crate) fn with_subject(mut self, subject: String) -> Self {
        self.subject = subject;
        self
    }
    /// Build a prepared event from an already encoded JSON body payload.
'''
new = '''impl PreparedTrellisEvent {
    /// Build a prepared event from an already encoded JSON body payload.
'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: prepared event subject mutator shape changed")
text = text.replace(old, new, 1)
if "with_subject(" in text:
    raise RuntimeError(f"{path}: stale prepared event subject mutator")
write(path, text)

path = "rust/crates/trellis/src/client/operations.rs"
text = read(path)
old = '''pub trait OperationTransport {
    fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: OperationTransport default resolver shape changed")
text = text.replace(old, "pub trait OperationTransport {\n", 1)
text = text.replace("self.transport.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("control_subject(&D::SUBJECT.to_owned())", "control_subject(D::SUBJECT)")
write(path, text)

path = "rust/crates/trellis/src/generated.rs"
text = read(path)
old = '''impl crate::client::OperationTransport for Caller {
    fn descriptor_subject(&self, subject: &str) -> String {
        self.client.descriptor_subject(subject)
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: generated OperationTransport resolver shape changed")
text = text.replace(old, "impl crate::client::OperationTransport for Caller {\n", 1)
write(path, text)

path = "rust/crates/trellis/src/service/router.rs"
text = read(path)
old = '''    fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: router subject identity helper changed")
text = text.replace(old, "", 1)
old = '''    fn descriptor_name(&self, name: &str) -> String {
        name.to_string()
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: router name identity helper changed")
text = text.replace(old, "", 1)
old = '''    fn descriptor_capabilities(&self, capabilities: &[&str]) -> Vec<String> {
        capabilities
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: router capability identity helper changed")
text = text.replace(old, "", 1)

# Generated descriptor capability lists are static data. Keep them static in the
# routing table and allocate owned strings only when a request asks for the
# required-capability projection.
old = '''enum RouteCapabilities {
    Static(Vec<String>),
    OperationControl {
        observe: Vec<String>,
        cancel: Vec<String>,
        control: Vec<String>,
    },
}
'''
new = '''enum RouteCapabilities {
    Static(&'static [&'static str]),
    OperationControl {
        observe: &'static [&'static str],
        cancel: &'static [&'static str],
        control: &'static [&'static str],
    },
}
'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: RouteCapabilities storage shape changed")
text = text.replace(old, new, 1)
old = '''        let capabilities = match self {
            Self::Static(capabilities) => capabilities,
            Self::OperationControl {
                observe,
                cancel,
                control,
            } => match serde_json::from_slice::<OperationControlRequest>(payload) {
                Ok(request) => match request.action.as_str() {
                    "get" | "wait" | "watch" => observe,
                    "cancel" => cancel,
                    "signal" => control,
                    _ => return Some(Vec::new()),
                },
                Err(_) => return Some(Vec::new()),
            },
        };

        Some(capabilities.to_vec())
'''
new = '''        let capabilities = match self {
            Self::Static(capabilities) => *capabilities,
            Self::OperationControl {
                observe,
                cancel,
                control,
            } => match serde_json::from_slice::<OperationControlRequest>(payload) {
                Ok(request) => match request.action.as_str() {
                    "get" | "wait" | "watch" => *observe,
                    "cancel" => *cancel,
                    "signal" => *control,
                    _ => return Some(Vec::new()),
                },
                Err(_) => return Some(Vec::new()),
            },
        };

        Some(
            capabilities
                .iter()
                .map(|capability| (*capability).to_owned())
                .collect(),
        )
'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: RouteCapabilities projection shape changed")
text = text.replace(old, new, 1)

text = text.replace("let capabilities = self.descriptor_capabilities(D::CALLER_CAPABILITIES);", "let capabilities = D::CALLER_CAPABILITIES;")
text = text.replace("let capabilities = self.descriptor_capabilities(D::SUBSCRIBE_CAPABILITIES);", "let capabilities = D::SUBSCRIBE_CAPABILITIES;")
text = text.replace("self.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("self.descriptor_name(D::KEY)", "D::KEY.to_owned()")
text = text.replace("self.descriptor_capabilities(D::CALLER_CAPABILITIES)", "D::CALLER_CAPABILITIES")
text = text.replace("self.descriptor_capabilities(D::OBSERVE_CAPABILITIES)", "D::OBSERVE_CAPABILITIES")
text = text.replace("self.descriptor_capabilities(D::CANCEL_CAPABILITIES)", "D::CANCEL_CAPABILITIES")
text = text.replace("self.descriptor_capabilities(D::CONTROL_CAPABILITIES)", "D::CONTROL_CAPABILITIES")
write(path, text)

path = "rust/crates/trellis/src/service/runtime_facade.rs"
text = read(path)
old = '''    fn descriptor_subject(&self, subject: &str) -> String {
        self.client.as_ref().map_or_else(
            || subject.to_owned(),
            |client| client.descriptor_subject(subject),
        )
    }

'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: service subject resolver shape changed")
text = text.replace(old, "", 1)
text = text.replace("self.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("control_subject(&D::SUBJECT.to_owned())", "control_subject(D::SUBJECT)")
text = text.replace("client.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace(".subscribe(D::SUBJECT.to_owned())", ".subscribe(D::SUBJECT)")
write(path, text)

# No source-level subject/name/capability compatibility resolver should survive.
for candidate in Path("rust/crates/trellis/src").rglob("*.rs"):
    content = candidate.read_text()
    for token in ("descriptor_subject(", "descriptor_name(", "descriptor_capabilities("):
        if token in content:
            raise RuntimeError(f"stale descriptor identity wrapper {token!r} in {candidate}")
