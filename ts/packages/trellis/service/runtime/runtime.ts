import type { NatsConnection } from "@nats-io/nats-core";
import type {
  AuthorizationContextCache,
  AuthorizationProviderCache,
} from "../../auth/authorization_context.ts";
import type { TrellisDurableEventConsumerBeforeReadinessCheckHook } from "../../session.ts";

// Node and Deno share the native transport loaded by runtime_transport.ts.
export type NatsConnectOpts = {
  servers: string | string[];
  token?: string;
  inboxPrefix?: string;
  authenticator?: unknown;
  maxReconnectAttempts?: number;
  waitOnFirstConnect?: boolean;
} & Record<string, unknown>;

export type NatsConnectFn = (opts: NatsConnectOpts) => Promise<NatsConnection>;

/** Initializes telemetry for a service runtime. */
export type InitTelemetryFn = (serviceName: string) => void;

export type TrellisServiceRuntimeDeps = {
  connect: NatsConnectFn;
  initTelemetry?: InitTelemetryFn;
  /** @internal Test hook for deterministic durable event readiness interleavings. */
  durableEventConsumerBeforeReadinessCheck?:
    TrellisDurableEventConsumerBeforeReadinessCheckHook;
  /** @internal Live-test hook for provider I/O and registry permission assertions. */
  authorizationProviderReady?: (
    provider: AuthorizationProviderCache,
    connection: NatsConnection,
    context: AuthorizationContextCache,
  ) => void;
};
