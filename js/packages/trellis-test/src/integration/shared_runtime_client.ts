import {
  TRELLIS_TEST_SHARED_RUNTIME_ENV,
  type TrellisIntegrationSharedRuntimeManifest,
} from "./shared_runtime_protocol.ts";

/** Returns whether the current process was given a shared NATS manifest. */
export function hasSharedRuntimeManifest(): boolean {
  return Deno.env.get(TRELLIS_TEST_SHARED_RUNTIME_ENV) !== undefined;
}

/** Reads and validates the shared NATS manifest from the worker environment. */
export async function readSharedRuntimeManifest(): Promise<
  TrellisIntegrationSharedRuntimeManifest
> {
  const path = Deno.env.get(TRELLIS_TEST_SHARED_RUNTIME_ENV);
  if (!path) throw new Error("missing Trellis shared runtime manifest path");
  const value: unknown = JSON.parse(await Deno.readTextFile(path));
  if (!isSharedRuntimeManifest(value)) {
    throw new Error("invalid Trellis shared runtime manifest");
  }
  return value;
}

function isSharedRuntimeManifest(
  value: unknown,
): value is TrellisIntegrationSharedRuntimeManifest {
  if (typeof value !== "object" || value === null) return false;
  const manifest = value as Record<string, unknown>;
  return manifest.version === 3 &&
    typeof manifest.runId === "string" &&
    typeof manifest.trellisUrl === "string" &&
    typeof manifest.natsUrl === "string" &&
    typeof manifest.websocketUrl === "string" &&
    typeof manifest.workdir === "string" &&
    typeof manifest.adminPassword === "string" &&
    typeof manifest.adminRpcUrl === "string" &&
    typeof manifest.adminRpcToken === "string" &&
    typeof manifest.tenants === "object" && manifest.tenants !== null &&
    typeof manifest.assignments === "object" && manifest.assignments !== null;
}
