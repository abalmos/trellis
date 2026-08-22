import { dirname, fromFileUrl, join, resolve } from "@std/path";
import type { TrellisIntegrationRuntimeOptions } from "@qlever-llc/trellis-test/integration";
import type { TrellisIntegrationRunnerConfig } from "@qlever-llc/trellis-test/integration/runner";

export const externalServiceRepoRoot = dirname(fromFileUrl(import.meta.url));
export const externalServiceRepoJsRoot = resolve(
  externalServiceRepoRoot,
  "../../../../../",
);

export const TRELLIS_TEST_SERVER_BIN_ENV = "TRELLIS_TEST_SERVER_BIN";

export function externalServiceRepoTrellisCommand(
  serverBin = Deno.env.get(TRELLIS_TEST_SERVER_BIN_ENV),
) {
  const env = {
    RUST_LOG: "info,trellis_runtime::platform::auth_callout=debug",
  };
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
  timeouts: {
    startupMs: 60_000,
    reconciliationMs: 15_000,
    waitForMs: 10_000,
    shutdownMs: 10_000,
  },
} satisfies TrellisIntegrationRuntimeOptions;

export default {
  runtime: externalServiceRepoRuntime,
  denoTestArgs: ["-A", "-c", join(externalServiceRepoRoot, "deno.json")],
  cases: [
    {
      id: "external.rpc-smoke",
      fixture: "external-service-repo",
      file: "integration/rpc_smoke.integration_test.ts",
      testName: "external.rpc-smoke calls service RPC through generic runner",
      coverage: ["rpc", "smoke"],
    },
  ],
} satisfies TrellisIntegrationRunnerConfig;
