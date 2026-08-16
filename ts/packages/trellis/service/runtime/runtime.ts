import type { NatsConnection } from "@nats-io/nats-core";
import type {
  AuthorizationContextCache,
  AuthorizationProviderCache,
} from "../../auth/authorization_context.ts";
import type { TrellisDurableEventConsumerBeforeReadinessCheckHook } from "../../session.ts";

// Keep the public service package runtime-neutral.
//
// Third-party service authors may use Deno or Node, so the shared service core cannot
// hard-code a transport or file system API. Environment-specific modules wire
// these adapters in from `./deno.ts` or `./node.ts`.
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
