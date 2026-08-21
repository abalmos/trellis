from __future__ import annotations

from pathlib import Path
import re


def read(path: str) -> str:
    return Path(path).read_text()


def write(path: str, text: str) -> None:
    Path(path).write_text(text)


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(f"{label}: expected one occurrence, found {count}")
    return text.replace(old, new, 1)


def replace_all(text: str, old: str, new: str, label: str, minimum: int = 1) -> str:
    count = text.count(old)
    if count < minimum:
        raise RuntimeError(f"{label}: expected at least {minimum} occurrences, found {count}")
    return text.replace(old, new)


def regex_sub(text: str, pattern: str, replacement: str, label: str, minimum: int = 1, flags: int = 0) -> str:
    updated, count = re.subn(pattern, replacement, text, flags=flags)
    if count < minimum:
        raise RuntimeError(f"{label}: expected at least {minimum} matches, found {count}")
    return updated


# trellis-rs no longer exposes a production feature whose only purpose is to
# rewrite contract identity and subjects for integration tests.
path = "rust/crates/trellis/Cargo.toml"
text = read(path)
text = replace_once(text, "integration-test-scoping = []\n", "", "remove scoping feature")
write(path, text)

path = "rust/crates/trellis/src/lib.rs"
text = read(path)
text = replace_once(
    text,
    '#[cfg(feature = "integration-test-scoping")]\n#[doc(hidden)]\npub mod integration_test_scoping;\n\n',
    "",
    "remove scoping module export",
)
write(path, text)

scope_path = Path("rust/crates/trellis/src/integration_test_scoping.rs")
if not scope_path.is_file():
    raise RuntimeError("integration_test_scoping.rs is missing")
scope_path.unlink()

# Private live-test helpers retain raw observation hooks, but no longer enable
# contract/subject mutation in the production facade.
path = "rust/crates/trellis-test/Cargo.toml"
text = read(path)
text = replace_once(
    text,
    'trellis-rs = { path = "../trellis", version = "0.12.0", features = ["integration-test-scoping", "test-support"] }',
    'trellis-rs = { path = "../trellis", version = "0.12.0", features = ["test-support"] }',
    "trellis-test feature dependency",
)
write(path, text)

# Test-only provider diagnostics belong to test-support, not a behavior-mutating
# feature. These files contain observation helpers only.
for path in [
    "rust/crates/trellis/src/client/authorization/provider_cache.rs",
    "rust/crates/trellis/src/client/authorization/mod.rs",
    "rust/crates/trellis/src/client/mod.rs",
]:
    text = read(path)
    text = text.replace('feature = "integration-test-scoping"', 'feature = "test-support"')
    write(path, text)

# Connection semantics: static descriptor identity stays static. Raw malformed
# admission/device proof hooks remain available only under test-support.
path = "rust/crates/trellis/src/client/connection.rs"
text = read(path)
text = text.replace(
    '#[cfg(feature = "integration-test-scoping")]\nuse crate::integration_test_scoping::IntegrationTestScope;\n',
    "",
)
text = regex_sub(
    text,
    r'\n\s*#\[cfg\(feature = "integration-test-scoping"\)\]\n\s*integration_test_scope: Option<IntegrationTestScope>,',
    "",
    "remove connection scope fields",
    minimum=3,
)
text = regex_sub(
    text,
    r'\n\s*#\[cfg\(feature = "integration-test-scoping"\)\]\n\s*integration_test_scope: None,',
    "",
    "remove connection scope initializers",
    minimum=2,
)
text = regex_sub(
    text,
    r'\n\s*#\[cfg\(feature = "integration-test-scoping"\)\]\n\s*integration_test_scope: opts\.integration_test_scope\.clone\(\),',
    "",
    "remove connection scope propagation",
    minimum=2,
)
text = regex_sub(
    text,
    r'\n    /// Apply an immutable integration-test contract namespace to this connection\.\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n    #\[doc\(hidden\)\]\n    pub fn with_integration_test_scope\(mut self, scope: IntegrationTestScope\) -> Self \{\n        self\.integration_test_scope = Some\(scope\);\n        self\n    \}\n',
    "\n",
    "remove connection scope setters",
    minimum=2,
)
old = '''    pub(crate) fn descriptor_subject(&self, subject: &str) -> String {
        #[cfg(feature = "integration-test-scoping")]
        {
            crate::integration_test_scoping::resolve_descriptor_subject(
                self.integration_test_scope.as_ref(),
                subject,
            )
            .expect("generated contract descriptor subjects are valid")
            .into_owned()
        }
        #[cfg(not(feature = "integration-test-scoping"))]
        subject.to_string()
    }

    #[cfg(feature = "integration-test-scoping")]
    pub(crate) fn integration_test_scope(&self) -> Option<&IntegrationTestScope> {
        self.integration_test_scope.as_ref()
    }
'''
new = '''    pub(crate) fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }
'''
text = replace_once(text, old, new, "static client descriptor subject")
# Any remaining scoping cfg in this module gates raw test hooks, not behavior.
text = text.replace('feature = "integration-test-scoping"', 'feature = "test-support"')
write(path, text)

# Router registration uses the descriptor exactly as authored/generated.
path = "rust/crates/trellis/src/service/router.rs"
text = read(path)
text = regex_sub(
    text,
    r'\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n    integration_test_scope: Option<crate::integration_test_scoping::IntegrationTestScope>,',
    "",
    "remove router scope field",
)
text = regex_sub(
    text,
    r'\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n    pub\(crate\) fn set_integration_test_scope\([\s\S]*?\n    \}\n\n    fn descriptor_subject',
    "\n    fn descriptor_subject",
    "remove router scope setter",
)
text = regex_sub(
    text,
    r'    fn descriptor_subject\(&self, subject: &str\) -> String \{[\s\S]*?\n    \}\n\n    fn descriptor_capabilities',
    '''    fn descriptor_subject(&self, subject: &str) -> String {
        subject.to_string()
    }

    fn descriptor_capabilities''',
    "router descriptor subject",
)
text = regex_sub(
    text,
    r'    fn descriptor_capabilities\(&self, capabilities: &\[&str\]\) -> Vec<String> \{[\s\S]*?\n    \}\n\n    fn descriptor_name',
    '''    fn descriptor_capabilities(&self, capabilities: &[&str]) -> Vec<String> {
        capabilities
            .iter()
            .map(|capability| (*capability).to_string())
            .collect()
    }

    fn descriptor_name''',
    "router descriptor capabilities",
)
text = regex_sub(
    text,
    r'    fn descriptor_name\(&self, name: &str\) -> String \{[\s\S]*?\n    \}\n\n    /// Register one descriptor-backed handler\.',
    '''    fn descriptor_name(&self, name: &str) -> String {
        name.to_string()
    }

    /// Register one descriptor-backed handler.''',
    "router descriptor name",
)
write(path, text)

# High-level service facade likewise keeps authored descriptor identity. Raw
# provider/NATS/request inspection remains test-support-only.
path = "rust/crates/trellis/src/service/runtime_facade.rs"
text = read(path)
text = text.replace(
    '#[cfg(feature = "integration-test-scoping")]\nuse crate::integration_test_scoping::IntegrationTestScope;\n',
    "",
)
text = regex_sub(
    text,
    r'\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n    integration_test_scope: Option<IntegrationTestScope>,',
    "",
    "remove service option scope field",
)
text = regex_sub(
    text,
    r'\n            #\[cfg\(feature = "integration-test-scoping"\)\]\n            integration_test_scope: None,',
    "",
    "remove service option scope initializer",
)
text = regex_sub(
    text,
    r'\n    /// Apply an immutable integration-test contract namespace to this connection\.\n    #\[cfg\(feature = "integration-test-scoping"\)\]\n    #\[doc\(hidden\)\]\n    pub fn with_integration_test_scope\(mut self, scope: IntegrationTestScope\) -> Self \{\n        self\.integration_test_scope = Some\(scope\);\n        self\n    \}\n',
    "\n",
    "remove service scope setter",
)
text = text.replace(
    '''        #[cfg(feature = "integration-test-scoping")]
        let mut router = router;
        #[cfg(feature = "integration-test-scoping")]
        router.set_integration_test_scope(client.integration_test_scope().cloned());
''',
    "",
)
text = regex_sub(
    text,
    r'\n                #\[cfg\(feature = "integration-test-scoping"\)\]\n                integration_test_scope: options\.integration_test_scope\.clone\(\),',
    "",
    "remove service connect scope propagation",
)
old = '''    #[cfg(feature = "integration-test-scoping")]
    let (event_api_id, event_name, publish_capabilities) = match client.integration_test_scope() {
        Some(scope) => (
            scope.contract_id(&event_api_id),
            scope.logical_name(&event_name),
            publish_capabilities
                .into_iter()
                .map(|capability| scope.capability(&capability))
                .collect(),
        ),
        None => (event_api_id, event_name, publish_capabilities),
    };
'''
text = replace_once(text, old, "", "remove event verifier scope mutation")
# The only remaining feature gates in this file are raw integration helpers.
text = text.replace('feature = "integration-test-scoping"', 'feature = "test-support"')
write(path, text)

# Generated ABI test helpers should not expose no-op namespace compatibility
# methods. Keep raw NATS access as a test-support hook.
path = "rust/crates/trellis/src/generated.rs"
text = read(path)
text = regex_sub(
    text,
    r'\n    /// Resolve a descriptor subject through this connection\'s integration-test scope\.[\s\S]*?\n    /// Return the connected NATS client for live transport-boundary tests\.',
    '\n    /// Return the connected NATS client for live transport-boundary tests.',
    "remove generated scope helper methods",
)
text = text.replace(
    '#[cfg(all(feature = "integration-test-scoping", feature = "test-support"))]',
    '#[cfg(feature = "test-support")]',
)
text = regex_sub(
    text,
    r'\n    #\[cfg\(feature = "integration-test-scoping"\)\] integration_test_scope: Option<\n        crate::integration_test_scoping::IntegrationTestScope,\n    >,',
    "",
    "remove generated service scope parameter",
)
text = regex_sub(
    text,
    r'\n            #\[cfg\(feature = "integration-test-scoping"\)\]\n            integration_test_scope,',
    "",
    "remove generated service scope propagation",
)
write(path, text)

# Shared-runtime protocol keeps only the actual per-test deployment namespace.
path = "ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts"
text = read(path)
text = replace_once(
    text,
    '  /** Immutable contract namespace tokens passed to this case\'s connections. */\n  readonly scope: { readonly runToken: string; readonly caseToken: string };\n',
    "",
    "remove TS shared scope",
)
text = replace_once(text, "  readonly version: 4;", "  readonly version: 5;", "shared manifest v5")
write(path, text)

path = "ts/packages/trellis-test/src/integration/shared_runtime_host.ts"
text = read(path)
text = replace_once(
    text,
    'import { caseScopeToken, integrationSlug } from "./names.ts";',
    'import { integrationSlug } from "./names.ts";',
    "remove case scope import",
)
text = regex_sub(
    text,
    r'\n      scope: \{\n        runToken: caseScopeToken\(runId\),\n        caseToken: caseScopeToken\(assignment\.id\),\n      \},',
    "",
    "remove shared host scope assignment",
)
text = replace_once(text, "      version: 4,", "      version: 5,", "shared host manifest v5")
write(path, text)

# Rust test harness uses assignment.namespace solely for deployment names. It no
# longer mutates contract/API/capability/subject identity.
path = "rust/crates/trellis-test/src/lib.rs"
text = read(path)
text = replace_once(
    text,
    '''struct SharedRuntimeAssignment {
    mode: String,
    namespace: String,
    tenant_id: String,
    scope: SharedRuntimeScope,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct SharedRuntimeScope {
    run_token: String,
    case_token: String,
}
''',
    '''struct SharedRuntimeAssignment {
    mode: String,
    namespace: String,
    tenant_id: String,
}
''',
    "remove Rust shared scope model",
)
text = replace_once(
    text,
    '''    /// Integration-test namespace construction failed.
    #[error(transparent)]
    IntegrationScope(#[from] trellis_rs::integration_test_scoping::IntegrationTestScopeError),

''',
    "",
    "remove scope error",
)
text = replace_once(text, "    if manifest.version != 4 {", "    if manifest.version != 5 {", "Rust shared manifest v5")
text = replace_all(
    text,
    "integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,",
    "namespace: Option<String>,",
    "replace harness scope fields",
    minimum=3,
)
text = replace_once(
    text,
    '''        let integration_test_scope = shared_runtime
            .as_ref()
            .map(|(_, _, assignment)| {
                trellis_rs::integration_test_scoping::IntegrationTestScope::new(
                    assignment.scope.run_token.clone(),
                    assignment.scope.case_token.clone(),
                )
            })
            .transpose()?;

''',
    "",
    "remove scope construction",
)
text = replace_once(
    text,
    '                    default_deployment: format!("{}-deployment", assignment.namespace),',
    '                    default_deployment: "deployment".to_owned(),',
    "shared default deployment",
)
text = replace_once(
    text,
    "                    integration_test_scope,\n                    control_plane_path: shared.control_plane_sqlite_path.clone(),",
    "                    namespace: Some(assignment.namespace.clone()),\n                    control_plane_path: shared.control_plane_sqlite_path.clone(),",
    "shared runtime namespace",
)
text = replace_once(
    text,
    "            integration_test_scope,\n            control_plane_path,",
    "            namespace: None,\n            control_plane_path,",
    "local runtime namespace",
)
# Remove the two runtime identity-mutating convenience methods.
text = regex_sub(
    text,
    r'\n    /// Resolve a descriptor subject through this case\'s integration-test scope\.[\s\S]*?\n    /// Return direct SQLite access for the runtime-owned Trellis control plane\.',
    '\n    /// Return direct SQLite access for the runtime-owned Trellis control plane.',
    "remove runtime scope helpers",
)
text = replace_once(
    text,
    "            integration_test_scope: self.integration_test_scope.clone(),",
    "            namespace: self.namespace.clone(),",
    "admin namespace",
)
# Service options no longer receive a hidden namespace.
text = regex_sub(
    text,
    r'        let options = trellis_rs::service::ServiceConnectOptions::new\(([\s\S]*?)\n        \)\n        \.with_session_key_seed\(random_session_seed\(\)\)\n        \.with_timeout_ms\(30_000\);\n        match self\.integration_test_scope\.clone\(\) \{\n            Some\(scope\) => options\.with_integration_test_scope\(scope\),\n            None => options,\n        \}',
    r'''        trellis_rs::service::ServiceConnectOptions::new(\1
        )
        .with_session_key_seed(random_session_seed())
        .with_timeout_ms(30_000)''',
    "service connect options without scope",
)
text = replace_once(
    text,
    "            integration_test_scope: options.integration_test_scope,",
    "            namespace: options.namespace,",
    "admin init namespace",
)
old = '''        let deployment_name = deployment.unwrap_or(&self.default_deployment);
        let deployment_name = self.integration_test_scope.as_ref().map_or_else(
            || deployment_name.to_string(),
            |scope| scope.identifier(deployment_name),
        );
'''
new = '''        let deployment_name = deployment.unwrap_or(&self.default_deployment);
        let deployment_name = self.namespace.as_ref().map_or_else(
            || deployment_name.to_string(),
            |namespace| format!("{namespace}-{deployment_name}"),
        );
'''
text = replace_once(text, old, new, "namespace deployment names")
text = text.replace("        let contract = contract.scoped(self.integration_test_scope.as_ref())?;\n", "")
text = text.replace("            integration_test_scope: self.integration_test_scope.clone(),\n", "")
text = text.replace("    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,\n", "")
text = text.replace("            self.integration_test_scope.clone(),\n", "")
text = text.replace("            reconnect.integration_test_scope.clone(),\n", "")
text = text.replace("        key.integration_test_scope.clone(),\n", "")
# Captured admission no longer wraps options in a namespace mutator.
text = regex_sub(
    text,
    r'        let options = match self\.integration_test_scope\.clone\(\) \{\n            Some\(scope\) => options\.with_integration_test_scope\(scope\),\n            None => options,\n        \};\n',
    "",
    "captured admission scope",
)
# Remove the contract artifact mutation implementation wholesale, preserving the
# TrellisTestContract impl closing brace.
start = text.find("    fn scoped(\n")
end = text.find("fn builtin_api_artifacts()", start)
if start == -1 or end == -1:
    raise RuntimeError("cannot locate test contract scoping implementation")
text = text[:start] + "}\n\n" + text[end:]
# connect_bound_user carries no namespace after contract identity is stable.
text = regex_sub(
    text,
    r'    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,\n',
    "",
    "connect bound scope parameter",
    minimum=1,
)
text = regex_sub(
    text,
    r'    let options = match integration_test_scope \{\n        Some\(scope\) => options\.with_integration_test_scope\(scope\),\n        None => options,\n    \};\n',
    "",
    "connect bound scope application",
)
write(path, text)

# Test call sites should assert the actual product subject/capability, not a
# compatibility helper. Keep replacements intentionally narrow and let the
# residual gate catch anything more complicated.
for path in Path("rust/crates/trellis/tests/integration").glob("*.rs"):
    text = path.read_text()
    text = re.sub(
        r'\b[A-Za-z_][A-Za-z0-9_]*\.integration_test_descriptor_subject\(([^()\n]+)\)',
        r'(\1).to_string()',
        text,
    )
    text = re.sub(
        r'\b[A-Za-z_][A-Za-z0-9_]*\.integration_test_descriptor_capability\(([^()\n]+)\)',
        r'(\1).to_string()',
        text,
    )
    path.write_text(text)

# Generated facade: remove namespace compatibility helpers and the scope
# argument from the ad hoc test service connector.
path = "rust/crates/trellis/src/generated.rs"
text = read(path)
text = text.replace('feature = "integration-test-scoping"', 'feature = "test-support"')
write(path, text)

# Runner: keep adaptive concurrency, but schedule fixture processes in parallel
# and force tests within each fixture to run serially. This removes the need to
# mutate protocol identity while retaining parallelism across independent fixtures.
path = "rust/crates/trellis-test/integration_runner.ts"
text = read(path)
old = '''    const runs: TestRun[] = [];
    if (sharedTests.length > 0) {
      runs.push(
        await runTests(executable, [
          ...testArgs,
          ...unregisteredTests.flatMap((name) => ["--skip", name]),
          ...isolatedTests.flatMap((name) => ["--skip", name]),
          `--test-threads=${jobs}`,
          "--format=pretty",
        ], env),
      );
    }
    for (const name of isolatedTests) {
'''
new = '''    const runs: TestRun[] = [];
    const fixtureGroups = groupRustTestsByFixture(sharedTests);
    for (let offset = 0; offset < fixtureGroups.length; offset += jobs) {
      const batch = fixtureGroups.slice(offset, offset + jobs);
      runs.push(...await Promise.all(batch.map((group) => {
        const otherSharedTests = sharedTests.filter((name) => !group.tests.includes(name));
        return runTests(executable, [
          ...testArgs,
          ...unregisteredTests.flatMap((name) => ["--skip", name]),
          ...isolatedTests.flatMap((name) => ["--skip", name]),
          ...otherSharedTests.flatMap((name) => ["--skip", name]),
          "--test-threads=1",
          "--format=pretty",
        ], env);
      })));
    }
    for (const name of isolatedTests) {
'''
text = replace_once(text, old, new, "fixture-level Rust runner")
anchor = '''export function partitionRustTests(
  testIds: readonly string[],
  classifications: ReadonlyMap<string, string>,
): { readonly sharedTests: string[]; readonly isolatedTests: string[] } {
'''
helper = '''export function groupRustTestsByFixture(
  testIds: readonly string[],
): Array<{ readonly fixture: string; readonly tests: string[] }> {
  const groups = new Map<string, string[]>();
  for (const id of testIds) {
    const separator = id.indexOf("::");
    if (separator <= 0) {
      throw new Error(`Rust integration test '${id}' has no fixture module prefix`);
    }
    const fixture = id.slice(0, separator);
    const tests = groups.get(fixture) ?? [];
    tests.push(id);
    groups.set(fixture, tests);
  }
  return [...groups.entries()]
    .toSorted(([left], [right]) => left.localeCompare(right))
    .map(([fixture, tests]) => ({ fixture, tests }));
}

'''
text = replace_once(text, anchor, helper + anchor, "fixture grouping helper")
write(path, text)

path = "rust/crates/trellis-test/integration_runner_test.ts"
text = read(path)
text = replace_once(
    text,
    "  loadIntegrationLiveArtifacts,\n  parseIntegrationRunnerArgs,",
    "  groupRustTestsByFixture,\n  loadIntegrationLiveArtifacts,\n  parseIntegrationRunnerArgs,",
    "import fixture grouping helper",
)
anchor = '''Deno.test("Rust integration runner separates isolated process cases", () => {
'''
test = '''Deno.test("Rust integration runner groups shared cases by fixture", () => {
  assertEquals(
    groupRustTestsByFixture([
      "rpc::second",
      "events::one",
      "rpc::first",
      "state::one",
    ]),
    [
      { fixture: "events", tests: ["events::one"] },
      { fixture: "rpc", tests: ["rpc::second", "rpc::first"] },
      { fixture: "state", tests: ["state::one"] },
    ],
  );
  assertThrows(
    () => groupRustTestsByFixture(["missing-prefix"]),
    Error,
    "fixture module prefix",
  );
});

'''
text = replace_once(text, anchor, test + anchor, "fixture grouping unit test")
write(path, text)

# Internal test manifest is intentionally version-locked; no compatibility path.
# The old scope tokens are now dead imports/data.

# Fail fast if behavior-mutation machinery survived anywhere in normal source or
# test harness code. CLEANUP.md may still describe historical work and is excluded.
residual_tokens = (
    "integration-test-scoping",
    "IntegrationTestScope",
    "integration_test_scope",
    "with_integration_test_scope",
    "integration_test_descriptor_subject",
    "integration_test_descriptor_capability",
)
for root in [Path("rust/crates/trellis"), Path("rust/crates/trellis-test"), Path("ts/packages/trellis-test/src")]:
    for candidate in root.rglob("*"):
        if not candidate.is_file() or candidate.suffix not in {".rs", ".toml", ".ts"}:
            continue
        content = candidate.read_text()
        for token in residual_tokens:
            if token in content:
                raise RuntimeError(f"stale test-scoping token {token!r} in {candidate}")
