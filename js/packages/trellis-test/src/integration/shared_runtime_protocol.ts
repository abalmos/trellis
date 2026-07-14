import type { LocalNatsBootstrapManifest } from "../nats_bootstrap.ts";

/** Environment variable containing the shared NATS manifest path for workers. */
export const TRELLIS_TEST_SHARED_RUNTIME_ENV = "TRELLIS_TEST_SHARED_RUNTIME";

/** Shared NATS server and isolated account assignments for parallel tests. */
export type TrellisIntegrationSharedRuntimeManifest = {
  /** Manifest format version. */
  readonly version: 2;
  /** Native NATS endpoint shared by every tenant. */
  readonly natsUrl: string;
  /** Websocket NATS endpoint shared by every tenant. */
  readonly websocketUrl: string;
  /** Workdir containing NATS credentials and configuration. */
  readonly workdir: string;
  /** Isolated NATS account pair keyed by integration case id. */
  readonly tenants: Record<string, LocalNatsBootstrapManifest>;
};
