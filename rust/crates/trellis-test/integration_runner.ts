import { dirname, fromFileUrl, join, resolve } from "@std/path";
import clientMatrix from "../../../integration/client-test-matrix.json" with {
  type: "json",
};
import runtimeMatrix from "../../../integration/rust-runtime-test-matrix.json" with {
  type: "json",
};
import {
  summarizeTrellisTestDurations,
  summarizeTrellisTestProcessStarts,
} from "../../../ts/packages/trellis-test/src/integration/metrics.ts";
import { startTrellisIntegrationSharedRuntimeHost } from "../../../ts/packages/trellis-test/src/integration/shared_runtime_host.ts";
import { TRELLIS_TEST_SHARED_RUNTIME_ENV } from "../../../ts/packages/trellis-test/src/integration/shared_runtime_protocol.ts";

const repoRoot = fromFileUrl(new URL("../../../", import.meta.url));
const INTEGRATION_BINARY_ENV = "TRELLIS_TEST_INTEGRATION_BIN";
const PREBUILT_ONLY_ENV = "TRELLIS_TEST_PREBUILT_ONLY";
const ALLOW_DIRTY_PREBUILT_ENV = "TRELLIS_TEST_ALLOW_DIRTY_PREBUILT";
export const INTEGRATION_LIVE_ARTIFACTS_MANIFEST =
  "dist/integration-runtime/manifest.json";
const INTEGRATION_LIVE_ARTIFACTS_FORMAT =
  "trellis.integration-live-artifacts.v1";
const LIVE_EXECUTABLES = {
  integrationTest: "trellis-integration-test",
  trellisServer: "trellis-server",
  trellisCli: "trellis-cli",
} as const;

type IntegrationLiveArtifacts = {
  readonly integrationBinary: string;
  readonly runtimeBinaries: Record<string, string>;
};

type IntegrationLiveArtifactsManifest = {
  readonly format: string;
  readonly sourceSha: string;
  readonly executables: Record<keyof typeof LIVE_EXECUTABLES, {
    readonly path: string;
    readonly sha256: string;
  }>;
};

type CargoArtifact = {
  readonly reason?: string;
  readonly executable?: string | null;
  readonly target?: { readonly name?: string };
  readonly profile?: { readonly test?: boolean };
};

if (import.meta.main) {
  Deno.exit(await main(Deno.args));
}

async function main(args: readonly string[]): Promise<number> {
  const { jobs, testArgs } = parseIntegrationRunnerArgs(args);
  const tempDir = fromFileUrl(
    new URL("../../target/trellis-test-tmp/", import.meta.url),
  );
  await Deno.mkdir(tempDir, { recursive: true });
  Deno.env.set("TMPDIR", tempDir);
  const executable = Deno.env.get(INTEGRATION_BINARY_ENV) ??
    await buildIntegrationTest();
  const runtimeBinaries = await buildRuntimeBinaries();
  const compiled = await listTests(executable, []);
  assertCompiledInventory(compiled);
  const tenantIds = await listTests(executable, testArgs);
  const classifications = rustTestClassifications();
  const { sharedTests, isolatedTests } = partitionRustTests(
    tenantIds,
    classifications,
  );
  const inheritedManifest = Deno.env.get(TRELLIS_TEST_SHARED_RUNTIME_ENV);
  const host = inheritedManifest === undefined
    ? await startTrellisIntegrationSharedRuntimeHost({
      runtime: {
        trellis: {
          command: {
            cmd: runtimeBinaries.TRELLIS_TEST_SERVER_BIN,
            args: ["--config", "{config}", "all"],
          },
        },
      },
      assignments: tenantIds.map((id) => ({
        id,
        namespacePrefix: "rs",
        classification: classifications.get(id) ?? "shared",
      })),
    })
    : {
      env: { [TRELLIS_TEST_SHARED_RUNTIME_ENV]: inheritedManifest },
      metrics: () => [],
      output: () => "shared host is owned by the live orchestrator",
      stop: async () => {},
    };

  try {
    const env = { ...host.env, ...runtimeBinaries, TMPDIR: tempDir };
    const runs: TestRun[] = [];
    if (sharedTests.length > 0) {
      runs.push(
        await runTests(executable, [
          ...testArgs,
          ...isolatedTests.flatMap((name) => ["--skip", name]),
          `--test-threads=${jobs}`,
          "--format=pretty",
        ], env),
      );
    }
    for (const name of isolatedTests) {
      runs.push(
        await runTests(executable, [
          name,
          "--exact",
          "--test-threads=1",
          "--format=pretty",
        ], env),
      );
    }
    const results = runs.flatMap((run) => rustTestResults(run.stdout));
    const success = runs.every((run) => run.success);
    if (success) assertRustExecutionInventory(tenantIds, results);
    console.log(JSON.stringify({
      event: "rust-integration-results",
      registered: expectedRustTests().length,
      compiled: compiled.length,
      selected: tenantIds.length,
      passed: results.filter((result) => result.status === "passed").length,
      failed: results.filter((result) => result.status === "failed").length,
      ignored: results.filter((result) => result.status === "ignored").length,
      tests: results,
    }));
    if (!success) {
      console.error(
        `shared Trellis output:\n${host.output?.() ?? "<unavailable>"}`,
      );
    }
    return runs.find((run) => !run.success)?.code ?? 0;
  } finally {
    try {
      if (inheritedManifest === undefined) {
        const metrics = host.metrics === undefined ? [] : await host.metrics();
        console.log(JSON.stringify({
          event: "integration-process-summary",
          starts: summarizeTrellisTestProcessStarts(metrics),
          slowest: summarizeTrellisTestDurations(metrics),
        }));
      }
    } finally {
      await host.stop();
    }
  }
}

type TestRun = {
  readonly success: boolean;
  readonly code: number;
  readonly stdout: string;
};

async function runTests(
  executable: string,
  args: readonly string[],
  env: Readonly<Record<string, string>>,
): Promise<TestRun> {
  const child = new Deno.Command(executable, {
    args: [...args],
    cwd: repoRoot,
    env,
    stdin: "inherit",
    stdout: "piped",
    stderr: "piped",
  }).spawn();
  const [status, stdout] = await Promise.all([
    child.status,
    teeOutput(child.stdout, Deno.stdout),
    teeOutput(child.stderr, Deno.stderr),
  ]).then(([status, stdout]) => [status, stdout] as const);
  return { success: status.success, code: status.code, stdout };
}

export function parseIntegrationRunnerArgs(args: readonly string[]): {
  readonly jobs: number;
  readonly testArgs: readonly string[];
} {
  let jobs = 4;
  const testArgs: string[] = [];
  for (let index = 0; index < args.length; index += 1) {
    const arg = args[index];
    if (arg === "--jobs") {
      jobs = positiveInteger(args[index + 1], arg);
      index += 1;
    } else if (arg.startsWith("--jobs=")) {
      jobs = positiveInteger(arg.slice("--jobs=".length), "--jobs");
    } else if (arg === "--") {
      testArgs.push(...args.slice(index + 1));
      break;
    } else {
      testArgs.push(arg);
    }
  }
  return { jobs, testArgs };
}

export function partitionRustTests(
  testIds: readonly string[],
  classifications: ReadonlyMap<string, string>,
): { readonly sharedTests: string[]; readonly isolatedTests: string[] } {
  const isolatedTests = testIds.filter((id) =>
    classifications.get(id) === "isolated-process"
  );
  return {
    sharedTests: testIds.filter((id) => !isolatedTests.includes(id)),
    isolatedTests,
  };
}

function positiveInteger(value: string | undefined, flag: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${flag} requires a positive integer`);
  }
  return parsed;
}

export async function buildIntegrationTest(): Promise<string> {
  rejectCargoFallback("Rust integration test executable");
  const output = await new Deno.Command("cargo", {
    args: [
      "test",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "trellis-rs",
      "--test",
      "integration",
      "--no-run",
      "--message-format=json",
    ],
    cwd: repoRoot,
    stdin: "null",
    stdout: "piped",
    stderr: "inherit",
  }).output();
  if (!output.success) {
    throw new Error(`cargo failed with status ${output.code}`);
  }

  let executable: string | undefined;
  for (const line of new TextDecoder().decode(output.stdout).split("\n")) {
    if (line === "") continue;
    const artifact = JSON.parse(line) as CargoArtifact;
    if (
      artifact.reason === "compiler-artifact" &&
      artifact.target?.name === "integration" &&
      artifact.profile?.test === true &&
      typeof artifact.executable === "string"
    ) {
      executable = artifact.executable;
    }
  }
  if (executable === undefined) {
    throw new Error(
      "cargo did not report the Rust integration test executable",
    );
  }
  return executable;
}

export async function buildRuntimeBinaries(): Promise<Record<string, string>> {
  const server = Deno.env.get("TRELLIS_TEST_SERVER_BIN");
  const cli = Deno.env.get("TRELLIS_TEST_CLI_BIN");
  if (server !== undefined && cli !== undefined) {
    return {
      TRELLIS_TEST_SERVER_BIN: server,
      TRELLIS_TEST_CLI_BIN: cli,
    };
  }
  rejectCargoFallback("Rust integration runtime binaries");

  const output = await new Deno.Command("cargo", {
    args: [
      "build",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "trellis-runtime",
      "-p",
      "trellis-cli",
      "--bins",
      "--message-format=json",
    ],
    cwd: repoRoot,
    stdin: "null",
    stdout: "piped",
    stderr: "inherit",
  }).output();
  if (!output.success) {
    throw new Error(
      `building Rust integration runtime binaries failed with status ${output.code}`,
    );
  }

  const binaries = new Map<string, string>();
  for (const line of new TextDecoder().decode(output.stdout).split("\n")) {
    if (line === "") continue;
    const artifact = JSON.parse(line) as CargoArtifact;
    if (
      artifact.reason === "compiler-artifact" &&
      typeof artifact.target?.name === "string" &&
      typeof artifact.executable === "string"
    ) {
      binaries.set(artifact.target.name, artifact.executable);
    }
  }
  const serverBinary = server ?? binaries.get("trellis-server");
  const cliBinary = cli ?? binaries.get("trellis");
  if (serverBinary === undefined || cliBinary === undefined) {
    throw new Error(
      "cargo did not report all Rust integration runtime binaries",
    );
  }
  return {
    TRELLIS_TEST_SERVER_BIN: serverBinary,
    TRELLIS_TEST_CLI_BIN: cliBinary,
  };
}

export async function buildIntegrationLiveArtifacts(
  manifestPath = INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
): Promise<IntegrationLiveArtifacts> {
  const integrationBinary = await buildIntegrationTest();
  const runtimeBinaries = await buildRuntimeBinaries();
  return await writeIntegrationLiveArtifacts(
    manifestPath,
    await currentSourceSha(),
    {
      integrationTest: integrationBinary,
      trellisServer: runtimeBinaries.TRELLIS_TEST_SERVER_BIN,
      trellisCli: runtimeBinaries.TRELLIS_TEST_CLI_BIN,
    },
  );
}

export async function writeIntegrationLiveArtifacts(
  manifestPath: string,
  sourceSha: string,
  executables: Readonly<Record<keyof typeof LIVE_EXECUTABLES, string>>,
): Promise<IntegrationLiveArtifacts> {
  const artifactDir = dirname(manifestPath);
  await Deno.mkdir(artifactDir, { recursive: true });
  const manifestExecutables =
    {} as IntegrationLiveArtifactsManifest["executables"];
  for (
    const name of Object.keys(LIVE_EXECUTABLES) as Array<
      keyof typeof LIVE_EXECUTABLES
    >
  ) {
    const path = LIVE_EXECUTABLES[name];
    const destination = join(artifactDir, path);
    await Deno.copyFile(executables[name], destination);
    await Deno.chmod(destination, 0o755);
    manifestExecutables[name] = {
      path,
      sha256: await sha256(destination),
    };
  }
  const manifest: IntegrationLiveArtifactsManifest = {
    format: INTEGRATION_LIVE_ARTIFACTS_FORMAT,
    sourceSha,
    executables: manifestExecutables,
  };
  await Deno.writeTextFile(
    manifestPath,
    `${JSON.stringify(manifest, null, 2)}\n`,
  );
  return liveArtifactPaths(artifactDir);
}

/** @internal Returns the prebuilt-artifacts error when the working tree has uncommitted changes. */
export function prebuiltDirtyTreeError(porcelain: string): string | undefined {
  return porcelain.trim() === ""
    ? undefined
    : "the working tree has uncommitted changes while --prebuilt-only is set; " +
      "rebuild the live artifacts with --build-only, or set " +
      "TRELLIS_TEST_ALLOW_DIRTY_PREBUILT=1 to use them anyway";
}

async function gitPorcelain(): Promise<string> {
  const output = await new Deno.Command("git", {
    args: ["status", "--porcelain"],
    cwd: repoRoot,
    stdin: "null",
    stdout: "piped",
    stderr: "inherit",
  }).output();
  if (!output.success) {
    throw new Error("failed to inspect the working tree status");
  }
  return new TextDecoder().decode(output.stdout);
}

async function assertPrebuiltTreeClean(): Promise<void> {
  if (Deno.env.get(PREBUILT_ONLY_ENV) !== "1") return;
  if (Deno.env.get(ALLOW_DIRTY_PREBUILT_ENV) === "1") return;
  const error = prebuiltDirtyTreeError(await gitPorcelain());
  if (error !== undefined) throw new Error(error);
}

export async function loadIntegrationLiveArtifacts(
  manifestPath = INTEGRATION_LIVE_ARTIFACTS_MANIFEST,
  expectedSourceSha?: string,
): Promise<IntegrationLiveArtifacts> {
  // Prebuilt binaries may be stale when the working tree has uncommitted
  // changes; the commit SHA alone cannot catch that.
  await assertPrebuiltTreeClean();
  const manifest = JSON.parse(
    await Deno.readTextFile(manifestPath),
  ) as Partial<IntegrationLiveArtifactsManifest>;
  if (manifest.format !== INTEGRATION_LIVE_ARTIFACTS_FORMAT) {
    throw new Error("unsupported integration live artifacts manifest format");
  }
  const sourceSha = expectedSourceSha ?? await currentSourceSha();
  if (manifest.sourceSha !== sourceSha) {
    throw new Error(
      `integration live artifacts source SHA ${manifest.sourceSha} does not match ${sourceSha}`,
    );
  }
  const artifactDir = dirname(manifestPath);
  for (
    const name of Object.keys(LIVE_EXECUTABLES) as Array<
      keyof typeof LIVE_EXECUTABLES
    >
  ) {
    const entry = manifest.executables?.[name];
    if (
      entry?.path !== LIVE_EXECUTABLES[name] ||
      !/^[0-9a-f]{64}$/.test(entry.sha256)
    ) {
      throw new Error(`invalid integration live artifact entry ${name}`);
    }
    const path = join(artifactDir, entry.path);
    if (await sha256(path) !== entry.sha256) {
      throw new Error(
        `integration live artifact checksum mismatch for ${name}`,
      );
    }
    await Deno.chmod(path, 0o755);
  }
  return liveArtifactPaths(artifactDir);
}

function liveArtifactPaths(artifactDir: string): IntegrationLiveArtifacts {
  const resolvedArtifactDir = resolve(artifactDir);
  return {
    integrationBinary: join(
      resolvedArtifactDir,
      LIVE_EXECUTABLES.integrationTest,
    ),
    runtimeBinaries: {
      TRELLIS_TEST_SERVER_BIN: join(
        resolvedArtifactDir,
        LIVE_EXECUTABLES.trellisServer,
      ),
      TRELLIS_TEST_CLI_BIN: join(
        resolvedArtifactDir,
        LIVE_EXECUTABLES.trellisCli,
      ),
    },
  };
}

async function currentSourceSha(): Promise<string> {
  const githubSha = Deno.env.get("GITHUB_SHA");
  if (githubSha !== undefined) return githubSha;
  const output = await new Deno.Command("git", {
    args: ["rev-parse", "HEAD"],
    cwd: repoRoot,
    stdin: "null",
    stdout: "piped",
    stderr: "inherit",
  }).output();
  if (!output.success) throw new Error("failed to resolve source SHA");
  return new TextDecoder().decode(output.stdout).trim();
}

async function sha256(path: string): Promise<string> {
  const digest = await crypto.subtle.digest(
    "SHA-256",
    await Deno.readFile(path),
  );
  return Array.from(
    new Uint8Array(digest),
    (byte) => byte.toString(16).padStart(2, "0"),
  ).join("");
}

async function listTests(
  executable: string,
  testArgs: readonly string[],
): Promise<string[]> {
  const output = await new Deno.Command(executable, {
    args: [...testArgs, "--list", "--format=terse"],
    cwd: repoRoot,
    stdin: "null",
    stdout: "piped",
    stderr: "inherit",
  }).output();
  if (!output.success) {
    throw new Error(
      `listing Rust integration tests failed with status ${output.code}`,
    );
  }
  const tests = testNamesFromList(new TextDecoder().decode(output.stdout));
  if (tests.length === 0) {
    throw new Error("Rust integration test binary reported no tests");
  }
  return tests;
}

export function testNamesFromList(output: string): string[] {
  return output.split("\n")
    .filter((line) => line.endsWith(": test"))
    .map((line) => line.slice(0, -": test".length));
}

type RustTestResult = {
  readonly name: string;
  readonly status: "passed" | "failed" | "ignored";
};

export function rustTestResults(output: string): RustTestResult[] {
  const results: RustTestResult[] = [];
  for (const line of output.split("\n")) {
    const match = /^test (.+) \.\.\. (ok|FAILED|ignored)$/.exec(line.trim());
    if (match === null) continue;
    results.push({
      name: match[1],
      status: match[2] === "ok"
        ? "passed"
        : match[2] === "FAILED"
        ? "failed"
        : "ignored",
    });
  }
  return results;
}

export function expectedRustTests(): string[] {
  return [...clientMatrix.cases, ...runtimeMatrix.cases]
    .filter((entry) => entry.completion.rust === "implemented")
    .map((entry) => {
      const implementation = entry.implementations?.rust;
      if (implementation === undefined) {
        throw new Error(
          `implemented Rust matrix case ${entry.id} has no mapping`,
        );
      }
      return `${implementation.module}::${implementation.function}`;
    })
    .toSorted();
}

export function rustTestClassifications(): ReadonlyMap<
  string,
  "shared" | "isolated-process"
> {
  return new Map(
    [...clientMatrix.cases, ...runtimeMatrix.cases]
      .filter((entry) => entry.completion.rust === "implemented")
      .map((entry) => {
        const implementation = entry.implementations?.rust;
        if (implementation === undefined) {
          throw new Error(
            `implemented Rust matrix case ${entry.id} has no mapping`,
          );
        }
        const classification = entry.classification ?? "shared";
        if (
          classification !== "shared" &&
          classification !== "isolated-process"
        ) {
          throw new Error(
            `Rust matrix case ${entry.id} has invalid classification ${classification}`,
          );
        }
        return [
          `${implementation.module}::${implementation.function}`,
          classification,
        ] as const;
      }),
  );
}

export async function verifyCompiledRustInventory(
  executable: string,
): Promise<void> {
  assertCompiledInventory(await listTests(executable, []));
}

function assertCompiledInventory(compiled: readonly string[]): void {
  assertSameTests(
    "registered Rust cases",
    expectedRustTests(),
    compiled.toSorted(),
  );
}

export function assertRustExecutionInventory(
  expected: readonly string[],
  results: readonly RustTestResult[],
): void {
  const ignored = results.filter((result) => result.status === "ignored");
  if (ignored.length > 0) {
    throw new Error(
      `selected Rust integration tests were ignored: ${
        ignored.map((result) => result.name).join(", ")
      }`,
    );
  }
  const executed = results
    .map((result) => result.name)
    .toSorted();
  assertSameTests("executed Rust cases", expected.toSorted(), executed);
}

function assertSameTests(
  label: string,
  expected: readonly string[],
  actual: readonly string[],
): void {
  const missing = expected.filter((name) => !actual.includes(name));
  const unexpected = actual.filter((name) => !expected.includes(name));
  if (
    expected.length !== actual.length || missing.length > 0 ||
    unexpected.length > 0
  ) {
    throw new Error(
      `${label} differ from expected inventory: missing [${
        missing.join(", ")
      }], ` +
        `unexpected [${unexpected.join(", ")}], ` +
        `expected ${expected.length}, actual ${actual.length}`,
    );
  }
}

function rejectCargoFallback(label: string): void {
  if (Deno.env.get(PREBUILT_ONLY_ENV) === "1") {
    throw new Error(
      `${label} is missing while ${PREBUILT_ONLY_ENV}=1; refusing Cargo fallback`,
    );
  }
}

async function teeOutput(
  stream: ReadableStream<Uint8Array>,
  output: { write(data: Uint8Array): Promise<number> },
): Promise<string> {
  const chunks: Uint8Array[] = [];
  for await (const chunk of stream) {
    chunks.push(chunk);
    await output.write(chunk);
  }
  const size = chunks.reduce((total, chunk) => total + chunk.length, 0);
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.length;
  }
  return new TextDecoder().decode(bytes);
}
