import { TrellisTestRuntime } from "../runtime.ts";
import {
  reserveHostTestSlot,
  type TrellisTestHostSlot,
} from "../control_plane_config.ts";
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

export const TRELLIS_TEST_EVENTS_ENV = "TRELLIS_TEST_EVENTS";

type TrellisIntegrationTestStatus =
  | "registered"
  | "started"
  | "passed"
  | "failed";

/** Returns an isolated runtime scope for a single direct integration test. */
export function runtimeScopeIsolated(): TrellisIntegrationScope {
  return { kind: "isolated" };
}

/**
 * Returns a case-scoped shared runtime scope.
 *
 * In shared-runtime mode, the case id determines the default deployment via the
 * shared run id. Each case owns a NATS account and Trellis process, so fixed
 * protocol subjects remain isolated without changing their semantics.
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
 * `runtimeScopeForCase(...)` tests to the shared host.
 */
export function trellisIntegrationTest(
  args: TrellisIntegrationTestOptions,
): void {
  const { caseId, name, scope, fn } = args;
  const useSharedRuntime = scope.kind === "shared-case" &&
    hasSharedRuntimeManifest();
  const liveRuntimeSanitizers = scope.kind === "shared-case";

  emitIntegrationTestEvent(caseId, name, "registered");

  Deno.test({
    name,
    sanitizeResources: args.sanitizeResources ??
      (liveRuntimeSanitizers ? false : undefined),
    sanitizeOps: args.sanitizeOps ??
      (liveRuntimeSanitizers ? false : undefined),
    async fn() {
      const startedAt = performance.now();
      emitIntegrationTestEvent(caseId, name, "started");
      let hostSlot: TrellisTestHostSlot | undefined;
      try {
        hostSlot = await reserveHostTestSlot();
        if (useSharedRuntime) {
          const manifest = await readSharedRuntimeManifest();
          const assignment = manifest.assignments[scope.caseId];
          if (assignment === undefined) {
            throw new Error(
              `missing shared runtime assignment for integration case ${scope.caseId}`,
            );
          }
          const tenant = manifest.tenants[assignment.tenantId];
          if (tenant === undefined) {
            throw new Error(
              `missing NATS tenant ${assignment.tenantId} for integration case ${scope.caseId}`,
            );
          }
          if (assignment.mode === "shared") {
            const runtime = await TrellisTestRuntime.attach({
              trellisUrl: manifest.trellisUrl,
              natsUrl: manifest.natsUrl,
              websocketUrl: manifest.websocketUrl,
              workdir: manifest.workdir,
              manifest: tenant,
              adminPassword: manifest.adminPassword,
              adminRpcProxy: {
                url: manifest.adminRpcUrl,
                token: manifest.adminRpcToken,
              },
              deployment: assignment.namespace,
              timeouts: args.runtime?.timeouts,
            });
            try {
              await fn(runtime);
            } finally {
              await runtime.stop();
            }
          } else {
            if (args.runtime === undefined) {
              throw new Error(
                "isolated-process integration tests require runtime options",
              );
            }
            await withTrellisIntegrationRuntime(fn, {
              ...args.runtime,
              deployment: assignment.namespace,
              nats: {
                workdir: manifest.workdir,
                natsUrl: manifest.natsUrl,
                websocketUrl: manifest.websocketUrl,
                manifest: tenant,
              },
            });
          }
        } else {
          if (args.runtime === undefined) {
            throw new Error(
              "trellisIntegrationTest requires runtime options unless a shared runtime manifest is present",
            );
          }
          await withTrellisIntegrationRuntime(fn, args.runtime);
        }
        emitIntegrationTestEvent(
          caseId,
          name,
          "passed",
          performance.now() - startedAt,
        );
      } catch (error) {
        emitIntegrationTestEvent(
          caseId,
          name,
          "failed",
          performance.now() - startedAt,
        );
        throw error;
      } finally {
        hostSlot?.release();
      }
    },
  });
}

function emitIntegrationTestEvent(
  caseId: string,
  testName: string,
  status: TrellisIntegrationTestStatus,
  durationMs?: number,
): void {
  const path = Deno.env.get(TRELLIS_TEST_EVENTS_ENV);
  if (path === undefined) return;
  Deno.writeTextFileSync(
    path,
    `${
      JSON.stringify({
        event: "integration-case",
        language: "typescript",
        caseId,
        testName,
        status,
        timestamp: new Date().toISOString(),
        ...(durationMs === undefined ? {} : { durationMs }),
      })
    }\n`,
    { append: true },
  );
}
