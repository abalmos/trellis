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

# Shared runtime manifest no longer carries contract/subject scope tokens.
replace_once(
    "ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts",
    "  /** Immutable contract namespace tokens passed to this case's connections. */\n"
    "  readonly scope: { readonly runToken: string; readonly caseToken: string };\n",
    "",
)
replace_once(
    "ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts",
    "  readonly version: 4;",
    "  readonly version: 5;",
)
replace_once(
    "ts/packages/trellis-test/src/integration/shared_runtime_client.ts",
    "return manifest.version === 4 &&",
    "return manifest.version === 5 &&",
)

path = "ts/packages/trellis-test/src/integration/shared_runtime_host.ts"
text = read(path)
text = text.replace(
    'import { caseScopeToken, integrationSlug } from "./names.ts";',
    'import { integrationSlug } from "./names.ts";',
)
text, count = re.subn(
    r',\n      scope: \{\n'
    r'        runToken: caseScopeToken\(runId\),\n'
    r'        caseToken: caseScopeToken\(assignment\.id\),\n'
    r'      \}',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: host scope assignment changed")
text = text.replace("version: 4,", "version: 5,")
write(path, text)

# Rust harness: keep per-test deployment uniqueness, remove artifact/subject mutation.
path = "rust/crates/trellis-test/src/lib.rs"
text = read(path)

text, count = re.subn(
    r'\n    scope: SharedRuntimeScope,\n'
    r'\}\n\n'
    r'#\[derive\(Clone, Debug, Deserialize\)\]\n'
    r'#\[serde\(rename_all = "camelCase"\)\]\n'
    r'struct SharedRuntimeScope \{\n'
    r'    run_token: String,\n'
    r'    case_token: String,\n'
    r'\}\n',
    "\n}\n",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: shared scope model changed")
text = text.replace("if manifest.version != 4 {", "if manifest.version != 5 {")

text, count = re.subn(
    r'\n    /// Integration-test namespace construction failed\.\n'
    r'    #\[error\(transparent\)\]\n'
    r'    IntegrationScope\(#\[from\] trellis_rs::integration_test_scoping::IntegrationTestScopeError\),\n',
    "\n",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: IntegrationScope error changed")

# Runtime field: namespace is only for case-owned deployment aliases.
text = text.replace(
    "    attached: bool,\n"
    "    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,\n"
    "    control_plane_path: PathBuf,",
    "    attached: bool,\n"
    "    deployment_namespace: Option<String>,\n"
    "    control_plane_path: PathBuf,",
)
text, count = re.subn(
    r'        let integration_test_scope = shared_runtime\n'
    r'            \.as_ref\(\)\n'
    r'            \.map\(\|\(_, _, assignment\)\| \{\n'
    r'                trellis_rs::integration_test_scoping::IntegrationTestScope::new\(\n'
    r'                    assignment\.scope\.run_token\.clone\(\),\n'
    r'                    assignment\.scope\.case_token\.clone\(\),\n'
    r'                \)\n'
    r'            \}\)\n'
    r'            \.transpose\(\)\?;\n',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: runtime scope construction changed")
text = text.replace(
    "                    attached: true,\n"
    "                    integration_test_scope,\n"
    "                    control_plane_path: shared.control_plane_sqlite_path.clone(),",
    "                    attached: true,\n"
    "                    deployment_namespace: Some(assignment.namespace.clone()),\n"
    "                    control_plane_path: shared.control_plane_sqlite_path.clone(),",
)
text = text.replace(
    "            attached: false,\n"
    "            integration_test_scope,\n"
    "            control_plane_path,",
    "            attached: false,\n"
    "            deployment_namespace: None,\n"
    "            control_plane_path,",
)

# Delete runtime identity helpers.
text, count = re.subn(
    r"    /// Resolve a descriptor subject through this case's integration-test scope\.\n"
    r'    #\[doc\(hidden\)\]\n'
    r'    pub fn integration_test_descriptor_subject\(&self, subject: &str\) -> String \{\n.*?'
    r'    \}\n\n',
    "",
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: descriptor helper changed")
text, count = re.subn(
    r'    /// Return the exact case-scoped contract installed by this runtime\.\n'
    r'    pub fn scoped_contract\(\n'
    r'        &self,\n'
    r'        contract: &TrellisTestContract,\n'
    r'    \) -> Result<TrellisTestContract, TrellisTestError> \{\n'
    r'        contract\.scoped\(self\.integration_test_scope\.as_ref\(\)\)\n'
    r'    \}\n\n',
    "",
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: scoped_contract helper changed")

# Admin receives the deployment namespace, not protocol mutation state.
text = text.replace(
    "            integration_test_scope: self.integration_test_scope.clone(),",
    "            deployment_namespace: self.deployment_namespace.clone(),",
)
text = text.replace(
    "    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,",
    "    deployment_namespace: Option<String>,",
)
text = text.replace(
    "            integration_test_scope: options.integration_test_scope,",
    "            deployment_namespace: options.deployment_namespace,",
)

# Explicit deployment aliases stay unique within the shared control plane.
text, count = re.subn(
    r'        let deployment_name = deployment\.unwrap_or\(&self\.default_deployment\);\n'
    r'        let deployment_name = self\.integration_test_scope\.as_ref\(\)\.map_or_else\(\n'
    r'            \|\| deployment_name\.to_string\(\),\n'
    r'            \|scope\| scope\.identifier\(deployment_name\),\n'
    r'        \);',
    '        let deployment_name = deployment.unwrap_or(&self.default_deployment);\n'
    '        let deployment_name = self.deployment_namespace.as_ref().map_or_else(\n'
    '            || deployment_name.to_string(),\n'
    '            |namespace| format!("{namespace}-{deployment_name}"),\n'
    '        );',
    text,
)
if count != 1:
    raise RuntimeError(f"{path}: deployment alias block changed")

# Service connection is ordinary product connection now.
text, count = re.subn(
    r'        let options = (trellis_rs::service::ServiceConnectOptions::new\(.*?\)\n'
    r'        \.with_session_key_seed\(random_session_seed\(\)\)\n'
    r'        \.with_timeout_ms\(30_000\));\n'
    r'        match self\.integration_test_scope\.clone\(\) \{\n'
    r'            Some\(scope\) => options\.with_integration_test_scope\(scope\),\n'
    r'            None => options,\n'
    r'        \}\n',
    lambda match: f"        {match.group(1)}\n",
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: service connect option scope changed")

# Contracts are approved exactly as authored.
text = re.sub(
    r'        let contract = contract\.scoped\(self\.integration_test_scope\.as_ref\(\)\)\?;\n',
    "",
    text,
)

# Ad-hoc service/reconnect values no longer carry scope.
text = re.sub(
    r'\n    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,',
    "",
    text,
)
text = re.sub(
    r'\n            integration_test_scope: self\.integration_test_scope\.clone\(\),',
    "",
    text,
)
text = text.replace(
    "        &key.participant_needs_digest,\n"
    "        &session_seed,\n"
    "        key.integration_test_scope.clone(),\n",
    "        &key.participant_needs_digest,\n"
    "        &session_seed,\n",
)
text = text.replace(
    "            &self.participant_digest,\n"
    "            self.integration_test_scope.clone(),\n"
    "            self.authorization_context_store.clone(),\n",
    "            &self.participant_digest,\n"
    "            self.authorization_context_store.clone(),\n",
)
text = text.replace("            reconnect.integration_test_scope.clone(),\n", "")
text = re.sub(
    r'        let options = match self\.integration_test_scope\.clone\(\) \{\n'
    r'            Some\(scope\) => options\.with_integration_test_scope\(scope\),\n'
    r'            None => options,\n'
    r'        \};\n',
    "",
    text,
)

# Delete the entire artifact-scoping implementation and recursive JSON rewriter.
text, count = re.subn(
    r'    fn scoped\(\n.*?\n\}\n\nfn builtin_api_artifacts\(\)',
    '}\n\nfn builtin_api_artifacts()',
    text,
    count=1,
    flags=re.S,
)
if count != 1:
    raise RuntimeError(f"{path}: contract scoping block changed")

# connect_bound_user no longer accepts/applies scope.
text = re.sub(
    r'\n    integration_test_scope: Option<trellis_rs::integration_test_scoping::IntegrationTestScope>,',
    "",
    text,
)
text = re.sub(
    r'    let options = match integration_test_scope \{\n'
    r'        Some\(scope\) => options\.with_integration_test_scope\(scope\),\n'
    r'        None => options,\n'
    r'    \};\n',
    "",
    text,
)

for token in ("integration_test_scoping", "IntegrationTestScope", "integration_test_scope", ".scoped("):
    if token in text:
        raise RuntimeError(f"{path}: scope token survived: {token}")
write(path, text)

# Rewrite private integration callsites that only asked the harness for scoped identity.
for test_path in Path("rust/crates/trellis/tests/integration").glob("*.rs"):
    text = test_path.read_text()
    text = re.sub(
        r'[A-Za-z_][A-Za-z0-9_]*\.integration_test_descriptor_subject\(([^()\n]+)\)',
        r'(\1).to_string()',
        text,
    )
    text = re.sub(
        r'[A-Za-z_][A-Za-z0-9_]*\.integration_test_descriptor_capability\(([^()\n]+)\)',
        r'(\1).to_string()',
        text,
    )
    text = re.sub(
        r'[A-Za-z_][A-Za-z0-9_]*\.scoped_contract\(&([A-Za-z_][A-Za-z0-9_]*)\)\?',
        r'\1.clone()',
        text,
    )
    test_path.write_text(text)

# Rust runner: parallelize fixture processes; serialize tests inside each fixture.
path = "rust/crates/trellis-test/integration_runner.ts"
text = read(path)
old = '''    const env = { ...host.env, ...runtimeBinaries, TMPDIR: tempDir };
    const runs: TestRun[] = [];
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
new = '''    const env = { ...host.env, ...runtimeBinaries, TMPDIR: tempDir };
    const runs: TestRun[] = [];
    if (sharedTests.length > 0) {
      if (testArgs.length === 0) {
        const fixtures = groupRustTestsByFixture(sharedTests);
        runs.push(...await runConcurrent(fixtures, jobs, ({ fixture }) =>
          runTests(executable, [
            `${fixture}::`,
            ...unregisteredTests.flatMap((name) => ["--skip", name]),
            ...isolatedTests.flatMap((name) => ["--skip", name]),
            "--test-threads=1",
            "--format=pretty",
          ], env)
        ));
      } else {
        runs.push(
          await runTests(executable, [
            ...testArgs,
            ...unregisteredTests.flatMap((name) => ["--skip", name]),
            ...isolatedTests.flatMap((name) => ["--skip", name]),
            "--test-threads=1",
            "--format=pretty",
          ], env),
        );
      }
    }
    for (const name of isolatedTests) {
'''
if text.count(old) != 1:
    raise RuntimeError(f"{path}: shared runner block changed")
text = text.replace(old, new, 1)

helpers = r'''
export function groupRustTestsByFixture(
  testIds: readonly string[],
): Array<{ readonly fixture: string; readonly tests: string[] }> {
  const grouped = new Map<string, string[]>();
  for (const testId of testIds) {
    const separator = testId.indexOf("::");
    if (separator <= 0) {
      throw new Error(`Rust integration test id has no fixture prefix: ${testId}`);
    }
    const fixture = testId.slice(0, separator);
    const tests = grouped.get(fixture) ?? [];
    tests.push(testId);
    grouped.set(fixture, tests);
  }
  return [...grouped]
    .map(([fixture, tests]) => ({ fixture, tests: tests.toSorted() }))
    .toSorted((left, right) => left.fixture.localeCompare(right.fixture));
}

async function runConcurrent<T, R>(
  values: readonly T[],
  concurrency: number,
  run: (value: T) => Promise<R>,
): Promise<R[]> {
  const results = new Array<R>(values.length);
  let next = 0;
  await Promise.all(
    Array.from(
      { length: Math.min(concurrency, values.length) },
      async () => {
        while (true) {
          const index = next++;
          if (index >= values.length) return;
          results[index] = await run(values[index]);
        }
      },
    ),
  );
  return results;
}

'''
marker = "function positiveInteger(value: string | undefined, flag: string): number {"
if text.count(marker) != 1:
    raise RuntimeError(f"{path}: runner helper marker changed")
text = text.replace(marker, helpers + marker, 1)
write(path, text)

path = "rust/crates/trellis-test/integration_runner_test.ts"
text = read(path)
if text.count("  buildRuntimeBinaries,\n") != 1:
    raise RuntimeError(f"{path}: import anchor changed")
text = text.replace(
    "  buildRuntimeBinaries,\n",
    "  buildRuntimeBinaries,\n  groupRustTestsByFixture,\n",
    1,
)
marker = 'Deno.test("Rust integration runner extracts test tenant names", () => {\n'
fixture_test = r'''Deno.test("Rust integration runner groups shared cases by fixture", () => {
  assertEquals(
    groupRustTestsByFixture([
      "rpc::second",
      "events::only",
      "rpc::first",
    ]),
    [
      { fixture: "events", tests: ["events::only"] },
      { fixture: "rpc", tests: ["rpc::first", "rpc::second"] },
    ],
  );
  assertThrows(
    () => groupRustTestsByFixture(["missing_fixture"]),
    Error,
    "fixture prefix",
  );
});

'''
if text.count(marker) != 1:
    raise RuntimeError(f"{path}: runner test marker changed")
text = text.replace(marker, fixture_test + marker, 1)
write(path, text)

for root in (
    Path("rust/crates/trellis-test"),
    Path("rust/crates/trellis/tests/integration"),
    Path("ts/packages/trellis-test/src/integration"),
):
    for candidate in root.rglob("*"):
        if not candidate.is_file() or candidate.suffix not in {".rs", ".toml", ".ts"}:
            continue
        content = candidate.read_text()
        for token in (
            "integration-test-scoping",
            "integration_test_scoping",
            "IntegrationTestScope",
            "integration_test_scope",
            "with_integration_test_scope",
            "integration_test_descriptor_subject",
            "integration_test_descriptor_capability",
            ".scoped_contract(",
        ):
            if token in content:
                raise RuntimeError(f"stale test-scope token {token!r} in {candidate}")
