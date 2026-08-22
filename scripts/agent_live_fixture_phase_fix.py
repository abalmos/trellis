from pathlib import Path


def replace_once(text: str, old: str, new: str, label: str) -> str:
    count = text.count(old)
    if count != 1:
        raise RuntimeError(
            f"{label}: expected one occurrence, found {count}: {old[:120]!r}"
        )
    return text.replace(old, new, 1)


# The ordinary trellis-test package suite owns source/unit tests. The nested
# external-repository integration case is live behavior and must not be picked
# up recursively by the package test phase.
path = Path("ts/deno.json")
text = path.read_text()
text = replace_once(
    text,
    '"test:prepared:trellis-test": "deno test -A packages/trellis-test",',
    '"test:prepared:trellis-test": "deno test -A --ignore=packages/trellis-test/tests/fixtures/external-service-repo/integration packages/trellis-test",',
    "exclude external live fixture from source phase",
)
path.write_text(text)


# Keep the external-repository fixture realistic for local use (cargo run), but
# let a live/CI phase inject an already-built server. This keeps compilation out
# of the runtime readiness deadline without teaching TrellisTestRuntime about
# Cargo or any other build system.
path = Path(
    "ts/packages/trellis-test/tests/fixtures/external-service-repo/trellis.integration.ts"
)
text = path.read_text()
text = replace_once(
    text,
    '''export const externalServiceRepoJsRoot = resolve(
  externalServiceRepoRoot,
  "../../../../../",
);

export const externalServiceRepoRuntime = {
  trellis: {
    command: {
      cmd: "cargo",
      args: [
        "run",
        "--manifest-path",
        "../rust/Cargo.toml",
        "-p",
        "trellis-runtime",
        "--bin",
        "trellis-server",
        "--",
        "--config",
        "{config}",
        "all",
      ],
      env: { RUST_LOG: "info,trellis_runtime::platform::auth_callout=debug" },
      cwd: externalServiceRepoJsRoot,
    },
  },
''',
    '''export const externalServiceRepoJsRoot = resolve(
  externalServiceRepoRoot,
  "../../../../../",
);

export const TRELLIS_TEST_SERVER_BIN_ENV = "TRELLIS_TEST_SERVER_BIN";

export function externalServiceRepoTrellisCommand(
  serverBin = Deno.env.get(TRELLIS_TEST_SERVER_BIN_ENV),
) {
  const env = { RUST_LOG: "info,trellis_runtime::platform::auth_callout=debug" };
  if (serverBin !== undefined && serverBin.length > 0) {
    return {
      cmd: serverBin,
      args: ["--config", "{config}", "all"],
      env,
      cwd: externalServiceRepoJsRoot,
    };
  }

  return {
    cmd: "cargo",
    args: [
      "run",
      "--manifest-path",
      "../rust/Cargo.toml",
      "-p",
      "trellis-runtime",
      "--bin",
      "trellis-server",
      "--",
      "--config",
      "{config}",
      "all",
    ],
    env,
    cwd: externalServiceRepoJsRoot,
  };
}

export const externalServiceRepoRuntime = {
  trellis: {
    command: externalServiceRepoTrellisCommand(),
  },
''',
    "external fixture prebuilt server override",
)
path.write_text(text)


# Guard both command paths in the fast fixture tests.
path = Path("ts/packages/trellis-test/tests/external_service_repo_fixture_test.ts")
text = path.read_text()
text = replace_once(
    text,
    '''  externalServiceRepoJsRoot,
  externalServiceRepoRoot,
  externalServiceRepoRuntime,
''',
    '''  externalServiceRepoJsRoot,
  externalServiceRepoRoot,
  externalServiceRepoRuntime,
  externalServiceRepoTrellisCommand,
''',
    "import prebuilt command helper",
)
text = replace_once(
    text,
    '''Deno.test("external service repo fixture runs through generic runner serial mode", async () => {
''',
    '''Deno.test("external service repo fixture accepts a prebuilt Trellis server", () => {
  assertEquals(externalServiceRepoTrellisCommand("/tmp/trellis-server"), {
    cmd: "/tmp/trellis-server",
    args: ["--config", "{config}", "all"],
    env: { RUST_LOG: "info,trellis_runtime::platform::auth_callout=debug" },
    cwd: externalServiceRepoJsRoot,
  });
});

Deno.test("external service repo fixture runs through generic runner serial mode", async () => {
''',
    "test prebuilt command override",
)
path.write_text(text)


# The live lane owns live process startup. Build the server before starting any
# readiness deadline, then run the external-repository smoke explicitly with
# that binary after the main live matrix.
path = Path(".github/workflows/check.yml")
text = path.read_text()
text = replace_once(
    text,
    '''      - name: Run live integration suite
        run: deno run -A -c ts/deno.json integration/live_runner.ts
''',
    '''      - name: Build live Trellis server
        run: cargo build --locked --manifest-path rust/Cargo.toml -p trellis-runtime --bin trellis-server

      - name: Run live integration suite
        run: deno run -A -c ts/deno.json integration/live_runner.ts

      - name: Run external repository smoke
        env:
          TRELLIS_TEST_SERVER_BIN: ${{ github.workspace }}/rust/target/debug/trellis-server
        run: >-
          deno run -A -c ts/deno.json
          ts/packages/trellis-test/src/integration/runner.ts
          --config
          ts/packages/trellis-test/tests/fixtures/external-service-repo/trellis.integration.ts
          --case external.rpc-smoke
''',
    "move external smoke to live check phase",
)
path.write_text(text)
