import { assertEquals } from "@std/assert";
import { join } from "@std/path";
import {
  runTrellisIntegrationTests,
  type TrellisIntegrationRunnerOptions,
} from "@qlever-llc/trellis-test/integration/runner";
import { TRELLIS_TEST_EVENTS_ENV } from "../src/integration/runtime.ts";
import config, {
  externalServiceRepoJsRoot,
  externalServiceRepoRoot,
  externalServiceRepoRuntime,
} from "./fixtures/external-service-repo/trellis.integration.ts";

type RunnerCommand = Parameters<
  NonNullable<TrellisIntegrationRunnerOptions["commandRunner"]>
>[0];

const fixtureConfigPath = join(
  externalServiceRepoRoot,
  "trellis.integration.ts",
);
const fixtureDenoConfigPath = join(externalServiceRepoRoot, "deno.json");
const fixtureTestPath = join(
  externalServiceRepoRoot,
  "integration",
  "rpc_smoke.integration_test.ts",
);
const fixtureTestName =
  "external.rpc-smoke calls service RPC through generic runner";
const sharedRuntimeEnv = "TRELLIS_TEST_SHARED_RUNTIME";

function recordSuccessfulRun(
  command: RunnerCommand,
  commands: RunnerCommand[],
): Promise<number> {
  const path = command.env?.[TRELLIS_TEST_EVENTS_ENV];
  if (path === undefined) throw new Error("runner did not supply event path");
  Deno.writeTextFileSync(
    path,
    ["registered", "started", "passed"].map((status) =>
      JSON.stringify({
        event: "integration-case",
        language: "typescript",
        caseId: "external.rpc-smoke",
        testName: fixtureTestName,
        status,
        timestamp: "2026-07-30T00:00:00.000Z",
        ...(status === "passed" ? { durationMs: 5 } : {}),
      })
    ).join("\n") + "\n",
  );
  const { [TRELLIS_TEST_EVENTS_ENV]: _, ...env } = command.env ?? {};
  const { env: _originalEnv, ...withoutEnv } = command;
  commands.push(
    Object.keys(env).length === 0 ? withoutEnv : {
      ...withoutEnv,
      env,
    },
  );
  return Promise.resolve(0);
}

Deno.test("external service repo fixture config supplies the Trellis command", () => {
  assertEquals(externalServiceRepoRuntime.trellis.command, {
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
  });
  assertEquals(config.denoTestArgs, ["-A", "-c", fixtureDenoConfigPath]);
  assertEquals(config.cases, [
    {
      id: "external.rpc-smoke",
      fixture: "external-service-repo",
      file: "integration/rpc_smoke.integration_test.ts",
      testName: fixtureTestName,
      coverage: ["rpc", "smoke"],
    },
  ]);
});

Deno.test("external service repo fixture runs through generic runner serial mode", async () => {
  const commands: RunnerCommand[] = [];

  const code = await runTrellisIntegrationTests({
    args: ["--config", fixtureConfigPath, "--case", "external.rpc-smoke"],
    cwd: externalServiceRepoRoot,
    commandRunner: (command) => recordSuccessfulRun(command, commands),
  });

  assertEquals(code, 0);
  assertEquals(commands, [
    {
      executable: Deno.execPath(),
      cwd: externalServiceRepoRoot,
      args: [
        "test",
        "-A",
        "-c",
        fixtureDenoConfigPath,
        "--filter",
        "/^(?:external\\.rpc-smoke calls service RPC through generic runner)$/",
        fixtureTestPath,
      ],
    },
  ]);
});

Deno.test("external service repo fixture runs through generic runner parallel mode", async () => {
  const commands: RunnerCommand[] = [];
  let stopCalls = 0;

  const code = await runTrellisIntegrationTests({
    args: [
      "--config",
      fixtureConfigPath,
      "--parallel",
      "--jobs",
      "2",
      "--fixture",
      "external-service-repo",
    ],
    cwd: externalServiceRepoRoot,
    commandRunner: (command) => recordSuccessfulRun(command, commands),
    sharedRuntimeHostStarter(args) {
      assertEquals(args.runtime, externalServiceRepoRuntime);
      return Promise.resolve({
        manifestPath: "/tmp/external-service-repo-manifest.json",
        env: {
          [sharedRuntimeEnv]: "/tmp/external-service-repo-manifest.json",
        },
        stop() {
          stopCalls += 1;
          return Promise.resolve();
        },
      });
    },
  });

  assertEquals(code, 0);
  assertEquals(stopCalls, 1);
  assertEquals(commands, [
    {
      executable: Deno.execPath(),
      cwd: externalServiceRepoRoot,
      env: {
        [sharedRuntimeEnv]: "/tmp/external-service-repo-manifest.json",
        DENO_JOBS: "2",
      },
      args: [
        "test",
        "-A",
        "-c",
        fixtureDenoConfigPath,
        "--parallel",
        "--filter",
        "/^(?:external\\.rpc-smoke calls service RPC through generic runner)$/",
        fixtureTestPath,
      ],
    },
  ]);
});
