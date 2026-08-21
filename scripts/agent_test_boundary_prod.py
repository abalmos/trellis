from pathlib import Path
import re

def read(path: str) -> str:
    return Path(path).read_text()

def write(path: str, text: str) -> None:
    Path(path).write_text(text)

def replace_once(path: str, old: str, new: str) -> None:
    text = read(path)
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{path}: expected one occurrence, found {count}: {old[:120]!r}")
    write(path, text.replace(old, new, 1))

replace_once("rust/crates/trellis/Cargo.toml", "integration-test-scoping = []\n", "")
replace_once(
    "rust/crates/trellis-test/Cargo.toml",
    'trellis-rs = { path = "../trellis", version = "0.12.0", features = ["integration-test-scoping", "test-support"] }',
    'trellis-rs = { path = "../trellis", version = "0.12.0", features = ["test-support"] }',
)
replace_once(
    "rust/crates/trellis/src/lib.rs",
    '#[cfg(feature = "integration-test-scoping")]\n#[doc(hidden)]\npub mod integration_test_scoping;\n\n',
    "",
)
scope_module = Path("rust/crates/trellis/src/integration_test_scoping.rs")
if not scope_module.is_file():
    raise RuntimeError("integration-test-scoping module is missing")
scope_module.unlink()

for path in (
    "rust/crates/trellis/src/client/mod.rs",
    "rust/crates/trellis/src/client/authorization/mod.rs",
    "rust/crates/trellis/src/client/authorization/provider_cache.rs",
):
    write(
        path,
        read(path).replace(
            '#[cfg(feature = "integration-test-scoping")]',
            '#[cfg(feature = "test-support")]',
        ),
    )

path = "rust/crates/trellis/src/client/connection.rs"
text = read(path).replace(
    '#[cfg(feature = "integration-test-scoping")]',
    '#[cfg(feature = "test-support")]',
)
text = text.replace(
    '#[cfg(feature = "test-support")]\nuse crate::integration_test_scoping::IntegrationTestScope;\n',
    "",
)
text = re.sub(
    r'\n\s*#\[cfg\(feature = "test-support"\)\]\n\s*(?:pub\(crate\) )?integration_test_scope: Option<IntegrationTestScope>,',
    "",
    text,
)
text = re.sub(
    r'\n\s*#\[cfg\(feature = "test-support"\)\]\n\s*integration_test_scope: (?:None|opts\.integration_test_scope\.clone\(\)),',
    "",
    text,
)
text, count = re.subn(
    r'\n    /// Apply an immutable integration-test contract namespace to this connection\.\n'
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    #\[doc\(hidden\)\]\n'
    r'    pub fn with_integration_test_scope\(mut self, scope: IntegrationTestScope\) -> Self \{\n'
    r'        self\.integration_test_scope = Some\(scope\);\n'
    r'        self\n'
    r'    \}\n',
    "\n",
    text,
)
if count != 2:
    raise RuntimeError(f"{path}: expected two option scope builders, removed {count}")
text, count = re.subn(
    r'    pub\(crate\) fn descriptor_subject\(&self, subject: &str\) -> String \{\n.*?'
    r'    \}\n\n'
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    pub\(crate\) fn integration_test_scope\(&self\) -> Option<&IntegrationTestScope> \{\n'
    r'        self\.integration_test_scope\.as_ref\(\)\n'
    r'    \}\n\n',
    "",
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: failed to remove descriptor subject scope boundary")
text = text.replace(".request_json(&self.descriptor_subject(D::SUBJECT), value)", ".request_json(D::SUBJECT, value)")
text = text.replace(".request_json(&self.descriptor_subject(D::SUBJECT), input)", ".request_json(D::SUBJECT, input)")
text = text.replace(
    "prepare_event::<D>(event)?.with_subject(self.descriptor_subject(D::SUBJECT))",
    "prepare_event::<D>(event)?.with_subject(D::SUBJECT)",
)
text = text.replace(
    'let event = event\n            .clone()\n            .with_subject(self.descriptor_subject(event.subject()));',
    "let event = event.clone();",
)
text = text.replace(".subscribe(self.descriptor_subject(D::SUBSCRIBE_SUBJECT))", ".subscribe(D::SUBSCRIBE_SUBJECT)")
text = text.replace(
    "event_consumer_config(&options, self.descriptor_subject(D::SUBSCRIBE_SUBJECT))",
    "event_consumer_config(&options, D::SUBSCRIBE_SUBJECT.to_owned())",
)
text = text.replace("let subject = self.descriptor_subject(D::SUBJECT);", "let subject = D::SUBJECT.to_owned();")
text, count = re.subn(
    r'impl OperationTransport for TrellisClient \{\n'
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{\n'
    r'        self\.descriptor_subject\(subject\)\n'
    r'    \}\n\n',
    "impl OperationTransport for TrellisClient {\n",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: failed to remove OperationTransport subject resolver")
if "IntegrationTestScope" in text or "integration_test_scope" in text:
    raise RuntimeError(f"{path}: scope state survived client cleanup")
write(path, text)

path = "rust/crates/trellis/src/client/operations.rs"
text = read(path)
text, count = re.subn(
    r'pub trait OperationTransport \{\n'
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{\n'
    r'        subject\.to_string\(\)\n'
    r'    \}\n\n',
    "pub trait OperationTransport {\n",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: operation transport subject hook changed")
text = text.replace("self.transport.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("control_subject(&D::SUBJECT.to_owned())", "control_subject(D::SUBJECT)")
write(path, text)

path = "rust/crates/trellis/src/generated.rs"
text = read(path).replace(
    '#[cfg(all(feature = "integration-test-scoping", feature = "test-support"))]',
    '#[cfg(feature = "test-support")]',
)
text, count = re.subn(
    r"    /// Resolve a descriptor subject through this connection's integration-test scope\.\n"
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    #\[doc\(hidden\)\]\n'
    r'    pub fn integration_test_descriptor_subject\(&self, subject: &str\) -> String \{\n.*?'
    r'    \}\n\n'
    r"    /// Resolve a capability through this connection's integration-test scope\.\n"
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    #\[doc\(hidden\)\]\n'
    r'    pub fn integration_test_descriptor_capability\(&self, capability: &str\) -> String \{\n.*?'
    r'    \}\n\n',
    "",
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: generated scope helpers changed")
text, count = re.subn(
    r'impl crate::client::OperationTransport for Caller \{\n'
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{\n'
    r'        self\.client\.descriptor_subject\(subject\)\n'
    r'    \}\n\n',
    "impl crate::client::OperationTransport for Caller {\n",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: generated operation resolver changed")
text = re.sub(
    r',\n    #\[cfg\(feature = "integration-test-scoping"\)\] integration_test_scope: Option<\n'
    r'        crate::integration_test_scoping::IntegrationTestScope,\n'
    r'    >',
    "",
    text,
)
text = re.sub(
    r'\n            #\[cfg\(feature = "integration-test-scoping"\)\]\n'
    r'            integration_test_scope,',
    "",
    text,
)
if "integration-test-scoping" in text or "integration_test_scoping" in text:
    raise RuntimeError(f"{path}: generated scope feature survived")
write(path, text)

path = "rust/crates/trellis/src/service/router.rs"
text = read(path)
text = re.sub(
    r'\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n'
    r'    integration_test_scope: Option<crate::integration_test_scoping::IntegrationTestScope>,',
    "",
    text,
)
text, count = re.subn(
    r'    #\[cfg\(feature = "integration-test-scoping"\)\]\n'
    r'    pub\(crate\) fn set_integration_test_scope\(\n'
    r'        &mut self,\n'
    r'        scope: Option<crate::integration_test_scoping::IntegrationTestScope>,\n'
    r'    \) \{\n'
    r'        self\.integration_test_scope = scope;\n'
    r'    \}\n\n',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: router scope setter changed")
router_helpers = (
    '    fn descriptor_capabilities(&self, capabilities: &[&str]) -> Vec<String> {\n'
    '        capabilities\n'
    '            .iter()\n'
    '            .map(|capability| (*capability).to_string())\n'
    '            .collect()\n'
    '    }\n'
)
text, count = re.subn(
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{\n.*?'
    r'    \}\n\n'
    r'    fn descriptor_capabilities\(&self, capabilities: &\[&str\]\) -> Vec<String> \{\n.*?'
    r'    \}\n\n'
    r'    fn descriptor_name\(&self, name: &str\) -> String \{\n.*?'
    r'    \}\n',
    router_helpers,
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: router descriptor helpers changed")
text = text.replace("self.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("self.descriptor_name(D::KEY)", "D::KEY.to_owned()")
if "integration-test-scoping" in text or "integration_test_scope" in text:
    raise RuntimeError(f"{path}: router scope survived")
write(path, text)

path = "rust/crates/trellis/src/service/runtime_facade.rs"
text = read(path).replace(
    '#[cfg(feature = "integration-test-scoping")]',
    '#[cfg(feature = "test-support")]',
)
text = text.replace(
    '#[cfg(feature = "test-support")]\nuse crate::integration_test_scoping::IntegrationTestScope;\n',
    "",
)
text = re.sub(
    r'\n    #\[cfg\(feature = "test-support"\)\]\n    integration_test_scope: Option<IntegrationTestScope>,',
    "",
    text,
)
text = re.sub(
    r'\n            #\[cfg\(feature = "test-support"\)\]\n            integration_test_scope: None,',
    "",
    text,
)
text, count = re.subn(
    r'    /// Apply an immutable integration-test contract namespace to this connection\.\n'
    r'    #\[cfg\(feature = "test-support"\)\]\n'
    r'    #\[doc\(hidden\)\]\n'
    r'    pub fn with_integration_test_scope\(mut self, scope: IntegrationTestScope\) -> Self \{\n'
    r'        self\.integration_test_scope = Some\(scope\);\n'
    r'        self\n'
    r'    \}\n\n',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: service option scope builder changed")
text = re.sub(
    r'\n        #\[cfg\(feature = "test-support"\)\]\n'
    r'        let mut router = router;\n'
    r'        #\[cfg\(feature = "test-support"\)\]\n'
    r'        router\.set_integration_test_scope\(client\.integration_test_scope\(\)\.cloned\(\)\);',
    "",
    text,
)
text = re.sub(
    r'\n                #\[cfg\(feature = "test-support"\)\]\n'
    r'                integration_test_scope: options\.integration_test_scope\.clone\(\),',
    "",
    text,
)
text, count = re.subn(
    r'\n    #\[cfg\(feature = "test-support"\)\]\n'
    r'    let \(event_api_id, event_name, publish_capabilities\) = match client\.integration_test_scope\(\) \{\n.*?'
    r'    \};',
    "",
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: event verifier scope block changed")
text, count = re.subn(
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{\n'
    r'        self\.client\.as_ref\(\)\.map_or_else\(\n'
    r'            \|\| subject\.to_owned\(\),\n'
    r'            \|client\| client\.descriptor_subject\(subject\),\n'
    r'        \)\n'
    r'    \}\n\n',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: service descriptor resolver changed")
text = text.replace("self.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace("control_subject(&D::SUBJECT.to_owned())", "control_subject(D::SUBJECT)")
text = text.replace("client.descriptor_subject(D::SUBJECT)", "D::SUBJECT.to_owned()")
text = text.replace(".subscribe(D::SUBJECT.to_owned())", ".subscribe(D::SUBJECT)")
if "IntegrationTestScope" in text or "integration_test_scope" in text:
    raise RuntimeError(f"{path}: service scope survived")
write(path, text)

for root in (Path("rust/crates/trellis/src"),):
    for candidate in root.rglob("*"):
        if not candidate.is_file() or candidate.suffix not in {".rs", ".toml"}:
            continue
        content = candidate.read_text()
        for token in (
            "integration-test-scoping",
            "integration_test_scoping",
            "IntegrationTestScope",
            "integration_test_scope",
            "with_integration_test_scope",
        ):
            if token in content:
                raise RuntimeError(f"stale production test-scope token {token!r} in {candidate}")
