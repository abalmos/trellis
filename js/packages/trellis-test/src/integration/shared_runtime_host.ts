import { dirname, join } from "@std/path";
import { NatsTestContainer } from "../nats_container.ts";
import {
  removeStaleMarkedDirectories,
  writeTrellisTestOwnerMarker,
} from "../cleanup.ts";
import {
  TRELLIS_TEST_SHARED_RUNTIME_ENV,
  type TrellisIntegrationSharedRuntimeManifest,
} from "./shared_runtime_protocol.ts";
import type { TrellisIntegrationRuntimeOptions } from "./types.ts";

const WORKDIR_PREFIX = "trellis-test-pool-";
const WORKDIR_OWNER_MARKER = ".trellis-test-owner";

/** Shared NATS host started for parallel Trellis integration workers. */
export type TrellisIntegrationSharedRuntimeHost = {
  /** Path to the manifest file passed to worker processes. */
  readonly manifestPath: string;
  /** Environment variables workers need to attach to their tenant. */
  readonly env: Record<string, string>;
  /** Stops the shared NATS server and removes its workdir. */
  stop(): Promise<void>;
};

/** Starts one NATS server with an isolated account pair per selected test case. */
export async function startTrellisIntegrationSharedRuntimeHost(args: {
  readonly runtime: TrellisIntegrationRuntimeOptions;
  readonly tenantIds: readonly string[];
}): Promise<TrellisIntegrationSharedRuntimeHost> {
  const workdir = await Deno.makeTempDir({ prefix: WORKDIR_PREFIX });
  await writeTrellisTestOwnerMarker(workdir, WORKDIR_OWNER_MARKER);
  await removeStaleMarkedDirectories({
    parent: dirname(workdir),
    prefix: WORKDIR_PREFIX,
    markerName: WORKDIR_OWNER_MARKER,
  });

  let nats: NatsTestContainer | undefined;
  try {
    nats = await NatsTestContainer.start(workdir, {
      startupMs: args.runtime.timeouts?.startupMs,
      tenantIds: args.tenantIds,
    });
    const manifest: TrellisIntegrationSharedRuntimeManifest = {
      version: 2,
      natsUrl: nats.natsUrl,
      websocketUrl: nats.websocketUrl,
      workdir,
      tenants: { ...nats.manifests },
    };
    const manifestPath = join(workdir, "shared-runtime-manifest.json");
    await Deno.writeTextFile(manifestPath, JSON.stringify(manifest));
    return {
      manifestPath,
      env: { [TRELLIS_TEST_SHARED_RUNTIME_ENV]: manifestPath },
      async stop() {
        await nats?.stop();
        await Deno.remove(workdir, { recursive: true }).catch(() => undefined);
      },
    };
  } catch (error) {
    await nats?.stop().catch(() => undefined);
    await Deno.remove(workdir, { recursive: true }).catch(() => undefined);
    throw error;
  }
}
