import { fromFileUrl } from "@std/path";
import { startTrellisIntegrationSharedRuntimeHost } from "../../../js/packages/trellis-test/src/integration/shared_runtime_host.ts";

const repoRoot = fromFileUrl(new URL("../../../", import.meta.url));

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
  const executable = await buildIntegrationTest();
  const runtimeBinaries = await buildRuntimeBinaries();
  const tenantIds = await listTests(executable, testArgs);
  const host = await startTrellisIntegrationSharedRuntimeHost({
    runtime: {},
    tenantIds,
  });

  try {
    const command = new Deno.Command(executable, {
      args: [...testArgs, `--test-threads=${jobs}`],
      cwd: repoRoot,
      env: { ...host.env, ...runtimeBinaries, TMPDIR: tempDir },
      stdin: "inherit",
      stdout: "inherit",
      stderr: "inherit",
    });
    return (await command.spawn().status).code;
  } finally {
    await host.stop();
  }
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

function positiveInteger(value: string | undefined, flag: string): number {
  const parsed = Number(value);
  if (!Number.isInteger(parsed) || parsed <= 0) {
    throw new Error(`${flag} requires a positive integer`);
  }
  return parsed;
}

async function buildIntegrationTest(): Promise<string> {
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

async function buildRuntimeBinaries(): Promise<Record<string, string>> {
  const jobs = Deno.env.get("TRELLIS_TEST_JOBS_SERVICE_BIN");
  const server = Deno.env.get("TRELLIS_TEST_SERVER_BIN");
  if (jobs !== undefined && server !== undefined) {
    return {
      TRELLIS_TEST_JOBS_SERVICE_BIN: jobs,
      TRELLIS_TEST_SERVER_BIN: server,
    };
  }

  const output = await new Deno.Command("cargo", {
    args: [
      "build",
      "--manifest-path",
      "rust/Cargo.toml",
      "-p",
      "trellis-service-jobs",
      "-p",
      "trellis-runtime",
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
  const jobsBinary = jobs ?? binaries.get("trellis-service-jobs");
  const serverBinary = server ?? binaries.get("trellis-server");
  if (jobsBinary === undefined || serverBinary === undefined) {
    throw new Error(
      "cargo did not report both Rust integration runtime binaries",
    );
  }
  return {
    TRELLIS_TEST_JOBS_SERVICE_BIN: jobsBinary,
    TRELLIS_TEST_SERVER_BIN: serverBinary,
  };
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
