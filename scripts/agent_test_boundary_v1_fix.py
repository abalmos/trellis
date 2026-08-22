from pathlib import Path
import re

path = Path("rust/crates/trellis-test/src/lib.rs")
text = path.read_text()

for struct_name in ("TrellisTestClientReconnect", "TrellisTestServiceKey"):
    pattern = rf"(pub struct {struct_name} \{{[\s\S]*?)\n    namespace: Option<String>,([\s\S]*?\n\}})"
    text, count = re.subn(pattern, r"\1\2", text, count=1)
    if count != 1:
        raise RuntimeError(f"expected one transformed namespace field in {struct_name}, found {count}")

path.write_text(text)

client_path = Path("ts/packages/trellis-test/src/integration/shared_runtime_client.ts")
client = client_path.read_text()
old = "  return manifest.version === 4 &&"
if client.count(old) != 1:
    raise RuntimeError(f"expected one shared-runtime client version check, found {client.count(old)}")
client_path.write_text(client.replace(old, "  return manifest.version === 5 &&", 1))

# The shared-runtime manifest is private and version-locked. Every active reader
# and writer moves together; do not carry a compatibility branch.
for path in (
    Path("rust/crates/trellis-test/src/lib.rs"),
    Path("ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts"),
    Path("ts/packages/trellis-test/src/integration/shared_runtime_host.ts"),
    Path("ts/packages/trellis-test/src/integration/shared_runtime_client.ts"),
):
    content = path.read_text()
    if "version: 4" in content or "version != 4" in content or "version !== 4" in content or "version === 4" in content:
        raise RuntimeError(f"stale shared-runtime manifest v4 reference in {path}")

# Keep the existing adaptive worker budget while serializing each fixture. A
# fixed batch would strand workers whenever one fixture in a batch runs long.
runner_path = Path("rust/crates/trellis-test/integration_runner.ts")
runner = runner_path.read_text()
old = '''    const fixtureGroups = groupRustTestsByFixture(sharedTests);
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
'''
new = '''    const fixtureGroups = groupRustTestsByFixture(sharedTests);
    let nextFixture = 0;
    const fixtureRuns = await Promise.all(
      Array.from(
        { length: Math.min(jobs, fixtureGroups.length) },
        async () => {
          const workerRuns: TestRun[] = [];
          while (nextFixture < fixtureGroups.length) {
            const group = fixtureGroups[nextFixture++];
            const otherSharedTests = sharedTests.filter((name) =>
              !group.tests.includes(name)
            );
            workerRuns.push(await runTests(executable, [
              ...testArgs,
              ...unregisteredTests.flatMap((name) => ["--skip", name]),
              ...isolatedTests.flatMap((name) => ["--skip", name]),
              ...otherSharedTests.flatMap((name) => ["--skip", name]),
              "--test-threads=1",
              "--format=pretty",
            ], env));
          }
          return workerRuns;
        },
      ),
    );
    runs.push(...fixtureRuns.flat());
'''
if runner.count(old) != 1:
    raise RuntimeError(f"expected one fixed fixture batch loop, found {runner.count(old)}")
runner_path.write_text(runner.replace(old, new, 1))

# Old scoped_contract() callers explicitly asked the harness for the current
# test's mutated participant artifact. With API/action/capability/subject
# mutation deleted, case_contract() is the direct participant-only equivalent.
# auth.rs already removes its one capability-name misuse before this generic pass.
for candidate in Path("rust/crates/trellis/tests/integration").glob("*.rs"):
    content = candidate.read_text()
    if ".scoped_contract(" in content:
        candidate.write_text(content.replace(".scoped_contract(", ".case_contract("))

# No identity-mutating harness helper should survive after the implementation is
# deleted. These names are intentionally not retained as no-op compatibility shims.
for root in (Path("rust/crates/trellis-test"), Path("rust/crates/trellis/tests/integration")):
    for candidate in root.rglob("*.rs"):
        content = candidate.read_text()
        for token in (
            "scoped_contract",
            "scope_authoring_source",
            "scope_artifact",
            "scope_source_value",
            "refresh_scoped_api_digests",
        ):
            if token in content:
                raise RuntimeError(f"stale integration identity helper {token!r} in {candidate}")
