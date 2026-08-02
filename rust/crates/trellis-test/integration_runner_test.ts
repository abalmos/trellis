import {
  assertEquals,
  assertRejects,
  assertStringIncludes,
  assertThrows,
} from "@std/assert";
import { isAbsolute, join, relative } from "@std/path";
import {
  assertRustExecutionInventory,
  buildIntegrationTest,
  buildRuntimeBinaries,
  loadIntegrationLiveArtifacts,
  parseIntegrationRunnerArgs,
  partitionRustTests,
  prebuiltDirtyTreeError,
  rustTestResults,
  testNamesFromList,
  writeIntegrationLiveArtifacts,
} from "./integration_runner.ts";

Deno.test("Rust integration runner parses worker and libtest arguments", () => {
  assertEquals(
    parseIntegrationRunnerArgs(["--jobs", "3", "--", "--nocapture"]),
    { jobs: 3, testArgs: ["--nocapture"] },
  );
  assertThrows(
    () => parseIntegrationRunnerArgs(["--jobs=0"]),
    Error,
    "positive integer",
  );
});

Deno.test("Rust integration runner separates isolated process cases", () => {
  assertEquals(
    partitionRustTests(
      ["rpc::shared", "runtime::isolated", "support"],
      new Map([["runtime::isolated", "isolated-process"]]),
    ),
    {
      sharedTests: ["rpc::shared", "support"],
      isolatedTests: ["runtime::isolated"],
    },
  );
});

Deno.test("Rust integration runner extracts test tenant names", () => {
  assertEquals(
    testNamesFromList(
      "rpc::success: test\nfeeds::success: test\n\n2 tests, 0 benchmarks\n",
    ),
    ["rpc::success", "feeds::success"],
  );
});

Deno.test("Rust integration runner extracts executed test results", () => {
  assertEquals(
    rustTestResults(
      "running 3 tests\ntest rpc::success ... ok\ntest rpc::failure ... FAILED\ntest rpc::pending ... ignored\n",
    ),
    [
      { name: "rpc::success", status: "passed" },
      { name: "rpc::failure", status: "failed" },
      { name: "rpc::pending", status: "ignored" },
    ],
  );
});

Deno.test("Rust integration runner validates every selected result", () => {
  const passed = (name: string) => ({ name, status: "passed" as const });
  assertThrows(
    () =>
      assertRustExecutionInventory(["rpc::one", "rpc::two"], [
        passed("rpc::one"),
      ]),
    Error,
    "missing [rpc::two]",
  );
  assertThrows(
    () =>
      assertRustExecutionInventory(["rpc::one"], [{
        name: "rpc::one",
        status: "ignored",
      }]),
    Error,
    "were ignored",
  );
  assertThrows(
    () =>
      assertRustExecutionInventory(["rpc::one"], [
        passed("rpc::one"),
        passed("rpc::extra"),
      ]),
    Error,
    "unexpected [rpc::extra]",
  );
});

Deno.test("prebuilt artifacts reject dirty working trees", () => {
  const error = prebuiltDirtyTreeError(" M rust/crates/trellis/src/lib.rs\n");
  assertStringIncludes(error ?? "", "--prebuilt-only");
  assertStringIncludes(error ?? "", "--build-only");
  assertStringIncludes(error ?? "", "TRELLIS_TEST_ALLOW_DIRTY_PREBUILT");
  assertEquals(prebuiltDirtyTreeError(""), undefined);
  assertEquals(prebuiltDirtyTreeError("   \n"), undefined);
});

Deno.test("prebuilt-only live artifacts fail on a dirty tree", async () => {
  const names = [
    "TRELLIS_TEST_PREBUILT_ONLY",
    "TRELLIS_TEST_ALLOW_DIRTY_PREBUILT",
  ] as const;
  const previous = new Map(names.map((name) => [name, Deno.env.get(name)]));
  const dirtyMarker = `.trellis-test-dirty-${crypto.randomUUID()}`;
  const markerPath = join(Deno.cwd(), dirtyMarker);
  await Deno.writeTextFile(markerPath, "dirty");
  try {
    Deno.env.set("TRELLIS_TEST_PREBUILT_ONLY", "1");
    Deno.env.delete("TRELLIS_TEST_ALLOW_DIRTY_PREBUILT");
    await assertRejects(
      () => loadIntegrationLiveArtifacts("/nonexistent/manifest.json"),
      Error,
      "uncommitted changes",
    );
  } finally {
    await Deno.remove(markerPath).catch(() => undefined);
    for (const [name, value] of previous) {
      if (value === undefined) Deno.env.delete(name);
      else Deno.env.set(name, value);
    }
  }
});

Deno.test("allow-dirty override skips the tree check", async () => {
  const names = [
    "TRELLIS_TEST_PREBUILT_ONLY",
    "TRELLIS_TEST_ALLOW_DIRTY_PREBUILT",
  ] as const;
  const previous = new Map(names.map((name) => [name, Deno.env.get(name)]));
  const tempDir = await Deno.makeTempDir({ prefix: "trellis-runner-dirty-" });
  try {
    const manifestPath = join(tempDir, "manifest.json");
    await Deno.writeTextFile(manifestPath, "{}");
    Deno.env.set("TRELLIS_TEST_PREBUILT_ONLY", "1");
    Deno.env.set("TRELLIS_TEST_ALLOW_DIRTY_PREBUILT", "1");
    // The dirty gate is skipped, so the manifest itself is validated instead.
    await assertRejects(
      () => loadIntegrationLiveArtifacts(manifestPath),
      Error,
      "unsupported integration live artifacts manifest format",
    );
  } finally {
    await Deno.remove(tempDir, { recursive: true }).catch(() => undefined);
    for (const [name, value] of previous) {
      if (value === undefined) Deno.env.delete(name);
      else Deno.env.set(name, value);
    }
  }
});

Deno.test("prebuilt-only Rust runner rejects every Cargo fallback", async () => {
  const names = [
    "TRELLIS_TEST_PREBUILT_ONLY",
    "TRELLIS_TEST_INTEGRATION_BIN",
    "TRELLIS_TEST_SERVER_BIN",
    "TRELLIS_TEST_CLI_BIN",
  ] as const;
  const previous = new Map(names.map((name) => [name, Deno.env.get(name)]));
  try {
    Deno.env.set("TRELLIS_TEST_PREBUILT_ONLY", "1");
    for (const name of names.slice(1)) Deno.env.delete(name);
    await assertRejects(
      buildIntegrationTest,
      Error,
      "refusing Cargo fallback",
    );
    await assertRejects(
      buildRuntimeBinaries,
      Error,
      "refusing Cargo fallback",
    );
  } finally {
    for (const [name, value] of previous) {
      if (value === undefined) Deno.env.delete(name);
      else Deno.env.set(name, value);
    }
  }
});

Deno.test("integration live artifacts are deterministic and validated", async () => {
  const tempDir = await Deno.makeTempDir();
  try {
    const sources = `${tempDir}/sources`;
    await Deno.mkdir(sources);
    const executables = {
      integrationTest: `${sources}/integration`,
      trellisServer: `${sources}/server`,
      trellisCli: `${sources}/cli`,
    };
    await Promise.all(
      Object.entries(executables).map(([name, path]) =>
        Deno.writeTextFile(path, name)
      ),
    );
    const manifestPath = relative(
      Deno.cwd(),
      `${tempDir}/dist/manifest.json`,
    );
    const sourceSha = "0123456789abcdef0123456789abcdef01234567";
    await writeIntegrationLiveArtifacts(manifestPath, sourceSha, executables);
    const firstManifest = await Deno.readTextFile(manifestPath);
    assertStringIncludes(
      firstManifest,
      '"format": "trellis.integration-live-artifacts.v1"',
    );
    assertStringIncludes(firstManifest, `"sourceSha": "${sourceSha}"`);
    await writeIntegrationLiveArtifacts(manifestPath, sourceSha, executables);
    assertEquals(await Deno.readTextFile(manifestPath), firstManifest);

    await Deno.chmod(`${tempDir}/dist/trellis-integration-test`, 0o644);
    const artifacts = await loadIntegrationLiveArtifacts(
      manifestPath,
      sourceSha,
    );
    assertEquals(
      ((await Deno.stat(artifacts.integrationBinary)).mode ?? 0) & 0o111,
      0o111,
    );
    assertEquals(
      Object.keys(artifacts.runtimeBinaries),
      [
        "TRELLIS_TEST_SERVER_BIN",
        "TRELLIS_TEST_CLI_BIN",
      ],
    );
    assertEquals(
      [
        artifacts.integrationBinary,
        ...Object.values(artifacts.runtimeBinaries),
      ].every(isAbsolute),
      true,
    );
    await Deno.writeTextFile(artifacts.integrationBinary, "tampered");
    await assertRejects(
      () => loadIntegrationLiveArtifacts(manifestPath, sourceSha),
      Error,
      "checksum mismatch",
    );
    await assertRejects(
      () => loadIntegrationLiveArtifacts(manifestPath, "different-source"),
      Error,
      "source SHA",
    );
  } finally {
    await Deno.remove(tempDir, { recursive: true });
  }
});
