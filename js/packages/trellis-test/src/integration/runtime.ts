import { TrellisTestRuntime } from "../runtime.ts";
import {
  hasSharedRuntimeManifest,
  readSharedRuntimeManifest,
} from "./shared_runtime_client.ts";
import type {
  TrellisIntegrationRuntime,
  TrellisIntegrationRuntimeOptions,
  TrellisIntegrationScope,
  TrellisIntegrationTestOptions,
} from "./types.ts";

/** Returns an isolated runtime scope for a single direct integration test. */
export function runtimeScopeIsolated(): TrellisIntegrationScope {
  return { kind: "isolated" };
}

/**
 * Returns a case-scoped shared runtime scope for parallel-safe tests.
 *
 * In shared-runtime mode, the case id determines the default deployment via the
 * shared run id. Tests must still use case-scoped contracts, subjects, names,
 * state keys, and resource keys for full parallel isolation.
 */
export function runtimeScopeForCase(caseId: string): TrellisIntegrationScope {
  return { kind: "shared-case", caseId };
}

/** Runs a test body with deterministic Trellis runtime cleanup. */
export async function withTrellisIntegrationRuntime<T>(
  fn: (runtime: TrellisIntegrationRuntime) => Promise<T>,
  options: TrellisIntegrationRuntimeOptions,
): Promise<T> {
  const runtime = await TrellisTestRuntime.start(options);
  try {
    return await fn(runtime);
  } catch (cause) {
    throw new Error(
      `${
        cause instanceof Error ? cause.message : String(cause)
      }\n${runtime.controlPlaneOutput()}`,
      { cause },
    );
  } finally {
    await runtime.stop();
  }
}

/**
 * Registers a Deno integration test backed by Trellis test runtime support.
 *
 * Direct mode starts a new runtime and requires caller-supplied runtime options,
 * including the Trellis command. Shared-runtime mode is activated by the
 * `TRELLIS_TEST_SHARED_RUNTIME` manifest environment variable and attaches
 * `runtimeScopeForCase(...)` tests to the shared coordinator.
 */
export function trellisIntegrationTest(
  args: TrellisIntegrationTestOptions,
): void {
  const { name, scope, fn } = args;
  const useSharedRuntime = scope.kind === "shared-case" &&
    hasSharedRuntimeManifest();
  const liveRuntimeSanitizers = scope.kind === "shared-case";

  Deno.test({
    name,
    sanitizeResources: args.sanitizeResources ??
      (liveRuntimeSanitizers ? false : undefined),
    sanitizeOps: args.sanitizeOps ??
      (liveRuntimeSanitizers ? false : undefined),
    async fn() {
      if (useSharedRuntime) {
        const manifest = await readSharedRuntimeManifest();
        const tenant = manifest.tenants[scope.caseId];
        if (tenant === undefined) {
          throw new Error(
            `missing NATS tenant for integration case ${scope.caseId}`,
          );
        }
        if (args.runtime === undefined) {
          throw new Error("parallel integration tests require runtime options");
        }
        await withTrellisIntegrationRuntime(fn, {
          ...args.runtime,
          nats: {
            workdir: manifest.workdir,
            natsUrl: manifest.natsUrl,
            websocketUrl: manifest.websocketUrl,
            manifest: tenant,
          },
        });
        return;
      }

      if (args.runtime === undefined) {
        throw new Error(
          "trellisIntegrationTest requires runtime options unless a shared runtime manifest is present",
        );
      }
      await withTrellisIntegrationRuntime(fn, args.runtime);
    },
  });
}
