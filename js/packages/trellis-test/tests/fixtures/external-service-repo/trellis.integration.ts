import { dirname, fromFileUrl, join, resolve } from "@std/path";
import type { TrellisIntegrationRuntimeOptions } from "@qlever-llc/trellis-test/integration";
import type { TrellisIntegrationRunnerConfig } from "@qlever-llc/trellis-test/integration/runner";

export const externalServiceRepoRoot = dirname(fromFileUrl(import.meta.url));
export const externalServiceRepoJsRoot = resolve(
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
