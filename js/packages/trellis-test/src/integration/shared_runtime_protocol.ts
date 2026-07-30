import type { LocalNatsBootstrapManifest } from "../nats_bootstrap.ts";

/** Environment variable containing the shared NATS manifest path for workers. */
export const TRELLIS_TEST_SHARED_RUNTIME_ENV = "TRELLIS_TEST_SHARED_RUNTIME";

export type TrellisIntegrationRuntimeAssignment = {
  readonly mode: "shared" | "isolated-process";
  readonly namespace: string;
  readonly tenantId: string;
  /** Immutable contract namespace tokens passed to this case's connections. */
  readonly scope: { readonly runToken: string; readonly caseToken: string };
};

/** Shared NATS/Trellis host and explicit worker assignments. */
export type TrellisIntegrationSharedRuntimeManifest = {
  /** Manifest format version. */
  readonly version: 3;
  /** Unique identifier for this host lifecycle. */
  readonly runId: string;
  /** HTTP endpoint of the shared Trellis runtime. */
  readonly trellisUrl: string;
  /** Native NATS endpoint shared by every tenant. */
  readonly natsUrl: string;
  /** Websocket NATS endpoint shared by every tenant. */
  readonly websocketUrl: string;
  /** Workdir containing NATS credentials and configuration. */
  readonly workdir: string;
  /** Local test-admin password for normal attached authentication. */
  readonly adminPassword: string;
  /** Localhost-only low-level Auth RPC adapter endpoint. */
  readonly adminRpcUrl: string;
  /** Bearer token protecting the low-level Auth RPC adapter. */
  readonly adminRpcToken: string;
  /** NATS account pair keyed by tenant assignment. */
  readonly tenants: Record<string, LocalNatsBootstrapManifest>;
  /** Shared or isolated-process assignment keyed by executable test identity. */
  readonly assignments: Record<string, TrellisIntegrationRuntimeAssignment>;
};
