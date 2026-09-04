import type { NatsConnection } from "@nats-io/nats-core";
import { Type } from "typebox";
import { Value } from "typebox/value";
import { ulid } from "ulid";
import {
  base64urlDecode,
  base64urlEncode,
  estimateMidpointClockOffsetMs,
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigest,
  sha256,
  type TrellisAuth as SessionAuth,
} from "../../auth.ts";
import {
  type AuthorizationContextBundle,
  AuthorizationContextBundleSchema,
} from "../../auth/authorization_context.ts";
import { decodeTrellisHttpError } from "../../auth/http_error.ts";
import { ContractResourceBindingsSchema } from "../../participant.ts";
import type { RuntimeApi } from "../../participant_runtime/api.ts";
import { resolveParticipantPresentation } from "../../participant_runtime/resolution.ts";
import { TransportError } from "../../errors/index.ts";
import type { LoggerLike } from "../../globals.ts";
import { loadDefaultRuntimeTransport } from "../../runtime_transport.ts";
import { initTelemetry } from "../../telemetry/init.ts";
import type { TrellisServiceRuntimeDeps } from "./runtime.ts";
import type {
  GeneratedServiceParticipant,
  ResourceBindings,
  TrellisServiceConnectOpts,
} from "./service.ts";

type ServiceBootstrapConnectInfo = {
  sessionId: string;
  participantDigest: string;
  instanceId: string;
  deploymentId: string;
  contractId: string;
  contractDigest: string;
  transports: {
    native?: { natsServers: string[] };
    websocket?: { natsServers: string[] };
  };
  jwt: string;
  jwtExpiresAt: number;
  authorizationContext: AuthorizationContextBundle;
};

/** Validated bootstrap material used to create a service runtime session. */
export type ServiceBootstrapResponse = {
  status: "ready";
  serverNow: number;
  serverClockOffsetMs: number;
  connectInfo: ServiceBootstrapConnectInfo;
  binding: {
    contractId: string;
    digest: string;
    resources: ResourceBindings;
  };
};

type ServiceBootstrapFailure = {
  reason: string;
  message?: string;
  serverNow?: number;
  requestId?: string;
  planId?: string;
  deploymentId?: string;
  dependencyAlias?: string;
  dependencyContractId?: string;
  dependencySurface?: string;
  dependencyReason?: string;
  dependencyKey?: string;
  dependencyMessage?: string;
};

const DEFAULT_BOOTSTRAP_PENDING_RETRY_MS = 5_000;
const MAX_BOOTSTRAP_PENDING_RETRY_MS = 60_000;
const DEFAULT_BOOTSTRAP_UNAVAILABLE_INITIAL_RETRY_MS = 1_000;
const MAX_BOOTSTRAP_UNAVAILABLE_RETRY_MS = 30_000;

function dependencyWaitLogMessage(failure: ServiceBootstrapFailure): string {
  if (failure.dependencyMessage) {
    return `Service contract activation pending; ${failure.dependencyMessage}`;
  }
  if (failure.dependencyContractId) {
    const dependency = failure.dependencyAlias
      ? `dependency '${failure.dependencyAlias}' (${failure.dependencyContractId})`
      : `dependency ${failure.dependencyContractId}`;
    if (failure.dependencyReason === "dependency_not_active") {
      return `Service contract activation pending; waiting for ${dependency} to have an active running implementation`;
    }
    if (failure.dependencyReason === "unknown") {
      return `Service contract activation pending; waiting for ${dependency} to be installed or approved`;
    }
    if (failure.dependencyKey) {
      return `Service contract activation pending; waiting for ${dependency} to provide required ${
        failure.dependencySurface ?? "surface"
      } '${failure.dependencyKey}'`;
    }
    return `Service contract activation pending; waiting for ${dependency}`;
  }
  return failure.message ??
    "Service contract activation pending; waiting for dependency closure";
}

function getErrorCauseMessage(error: unknown): string {
  if (error && typeof error === "object") {
    const context = (error as { context?: Record<string, unknown> }).context;
    if (
      typeof context?.causeMessage === "string" &&
      context.causeMessage.length > 0
    ) {
      return context.causeMessage;
    }
  }

  return error instanceof Error ? error.message : String(error);
}

function delay(ms: number): Promise<void> {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function bootstrapRetryDelayMs(response: Response): number {
  const retryAfter = response.headers.get("retry-after");
  if (retryAfter === null) return DEFAULT_BOOTSTRAP_PENDING_RETRY_MS;

  const seconds = Number(retryAfter);
  if (Number.isFinite(seconds) && seconds >= 0) {
    return Math.min(seconds * 1_000, MAX_BOOTSTRAP_PENDING_RETRY_MS);
  }

  const retryAt = Date.parse(retryAfter);
  if (Number.isNaN(retryAt)) return DEFAULT_BOOTSTRAP_PENDING_RETRY_MS;
  return Math.min(
    Math.max(0, retryAt - Date.now()),
    MAX_BOOTSTRAP_PENDING_RETRY_MS,
  );
}

function bootstrapUnavailableRetryDelayMs(attempt: number): number {
  const exponent = Math.min(attempt, 10);
  return Math.min(
    DEFAULT_BOOTSTRAP_UNAVAILABLE_INITIAL_RETRY_MS * 2 ** exponent,
    MAX_BOOTSTRAP_UNAVAILABLE_RETRY_MS,
  );
}

class ServiceBootstrapEndpointUnavailableError extends Error {
  constructor(cause: unknown) {
    super("Service bootstrap endpoint is unavailable.", { cause });
    this.name = "ServiceBootstrapEndpointUnavailableError";
  }
}

/** Loads the default transport and telemetry dependencies for service connection. */
export async function loadDefaultServiceRuntimeDeps(): Promise<
  TrellisServiceRuntimeDeps
> {
  const transport = await loadDefaultRuntimeTransport();
  return {
    initTelemetry,
    connect: (
      { servers, token, authenticator, inboxPrefix, ...extraOptions },
    ) =>
      transport.connect({
        servers,
        ...extraOptions,
        ...(token ? { token } : {}),
        ...(authenticator ? { authenticator: authenticator as never } : {}),
        ...(inboxPrefix ? { inboxPrefix } : {}),
      }),
  };
}

const ServiceBootstrapReadySchema = Type.Object({
  serverNow: Type.Integer(),
  state: Type.Literal("ready"),
  session: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    inboxPrefix: Type.String({ minLength: 1 }),
    principalId: Type.String({ minLength: 1 }),
    principalKind: Type.Literal("service"),
    participantId: Type.String({ minLength: 1 }),
    participantKind: Type.Literal("service"),
    participantArtifactDigest: Type.String({ minLength: 1 }),
    participantNeedsDigest: Type.String({ minLength: 1 }),
    sessionPublicKey: Type.String({ minLength: 1 }),
    sessionKeyId: Type.String({ minLength: 1 }),
    state: Type.Literal("active"),
    createdAt: Type.Integer(),
    lastSeenAt: Type.Integer(),
    expiresAt: Type.Integer(),
    revokedAt: Type.Union([Type.Integer(), Type.Null()]),
    version: Type.Integer({ minimum: 1 }),
  }, { additionalProperties: false }),
  authorization: Type.Object({
    participantId: Type.String({ minLength: 1 }),
    participantArtifactDigest: Type.String({ minLength: 1 }),
    participantNeedsDigest: Type.String({ minLength: 1 }),
    participantJson: Type.String({ minLength: 1 }),
    effectiveGrants: Type.Unknown(),
    resourceBindings: Type.Array(Type.Unknown()),
    resourceRuntime: ContractResourceBindingsSchema,
    effectiveAuthorityExpiresAt: Type.Union([Type.Integer(), Type.Null()]),
  }, { additionalProperties: false }),
  nats: Type.Object({
    jwt: Type.String({ minLength: 1 }),
    jwtExpiresAt: Type.Integer({ minimum: 1 }),
    transports: Type.Object({
      native: Type.Optional(Type.Object({
        natsServers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
      }, { additionalProperties: false })),
      websocket: Type.Optional(Type.Object({
        natsServers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
      }, { additionalProperties: false })),
    }, { additionalProperties: false }),
  }, { additionalProperties: false }),
  authorizationContext: AuthorizationContextBundleSchema,
  activation: Type.Union([Type.Unknown(), Type.Null()]),
  proposal: Type.Union([Type.Unknown(), Type.Null()]),
}, { additionalProperties: false });

const ServiceBootstrapFailureSchema = Type.Object({
  reason: Type.String({ minLength: 1 }),
  message: Type.Optional(Type.String({ minLength: 1 })),
  serverNow: Type.Optional(Type.Integer()),
}, { additionalProperties: false });

async function fetchServiceBootstrapInfoOnce(args: {
  bootstrapUrl: URL;
  contractId: string;
  contractDigest: string;
  contract: GeneratedServiceParticipant<RuntimeApi, RuntimeApi | undefined>;
  identityAuth: SessionAuth;
  sessionAuth: SessionAuth;
  identity: TrellisServiceConnectOpts["identity"];
}): Promise<{
  response: Response;
  payload: unknown;
  requestStartedAtMs: number;
  responseReceivedAtMs: number;
}> {
  const requestStartedAtMs = Date.now();
  const requestId = ulid();
  const issuedAt = args.identityAuth.currentIat() * 1_000;
  const provisionedIdentityKeyId = base64urlEncode(
    await sha256(base64urlDecode(args.identityAuth.sessionKey)),
  );
  const presentation = await resolveParticipantPresentation(args.contract);
  if (
    args.identity.participantId !== args.contract.id ||
    args.identity.participantArtifactDigest !== args.contract.digest ||
    args.identity.participantNeedsDigest !==
      presentation.participantNeedsDigest
  ) {
    throw new Error("Service participant identity does not match its contract");
  }
  const unsigned = {
    requestId,
    issuedAt,
    deploymentId: args.identity.deploymentId,
    instanceId: args.identity.instanceId,
    provisionedIdentityKeyId,
    newSessionPublicKey: args.sessionAuth.sessionKey,
    newSessionNkey: args.sessionAuth.sessionNkey,
    participantId: args.identity.participantId,
    participantArtifactDigest: args.contract.digest,
    participantNeedsDigest: presentation.participantNeedsDigest,
    participantArtifact: presentation.participant,
    referencedApiArtifacts: [presentation.api, ...presentation.referencedApis],
    proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
  };
  const requestDigest = await sessionProofRequestDigest(unsigned);
  const body = JSON.stringify({
    ...unsigned,
    proof: await args.identityAuth.signSessionProof({
      purpose: "serviceBootstrap",
      requestId,
      issuedAt,
      deploymentId: args.identity.deploymentId,
      instanceId: args.identity.instanceId,
      provisionedIdentityKeyId,
      newSessionPublicKey: args.sessionAuth.sessionKey,
      newSessionNkey: args.sessionAuth.sessionNkey,
      participantId: args.identity.participantId,
      participantDigest: args.identity.participantArtifactDigest,
      requestDigest,
    }),
  });
  let response: Response;
  try {
    response = await fetch(args.bootstrapUrl, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body,
    });
  } catch (cause) {
    throw new ServiceBootstrapEndpointUnavailableError(cause);
  }
  const responseReceivedAtMs = Date.now();

  if (!response.ok) throw await decodeTrellisHttpError(response);
  let payload: unknown;
  try {
    payload = await response.json();
  } catch {
    payload = undefined;
  }
  return {
    response,
    payload,
    requestStartedAtMs,
    responseReceivedAtMs,
  };
}

/** Fetches and validates service bootstrap material, retrying pending states. */
export async function fetchServiceBootstrapInfo(args: {
  trellisUrl: string;
  serviceName: string;
  contractId: string;
  contractDigest: string;
  contract: GeneratedServiceParticipant<RuntimeApi, RuntimeApi | undefined>;
  identityAuth: SessionAuth;
  sessionAuth: SessionAuth;
  identity: TrellisServiceConnectOpts["identity"];
  log: LoggerLike;
}): Promise<ServiceBootstrapResponse> {
  const bootstrapUrl = new URL("/bootstrap/service", args.trellisUrl);
  let unavailableAttempt = 0;
  const loggedPendingRequests = new Set<string>();
  while (true) {
    let settled: Awaited<ReturnType<typeof fetchServiceBootstrapInfoOnce>>;
    try {
      settled = await fetchServiceBootstrapInfoOnce({
        ...args,
        bootstrapUrl,
        contract: args.contract,
      });
      unavailableAttempt = 0;
    } catch (cause) {
      if (!(cause instanceof ServiceBootstrapEndpointUnavailableError)) {
        throw cause;
      }

      const retryDelayMs = bootstrapUnavailableRetryDelayMs(unavailableAttempt);
      unavailableAttempt += 1;
      args.log.warn(
        {
          service: args.serviceName,
          trellisUrl: args.trellisUrl,
          contractId: args.contractId,
          contractDigest: args.contractDigest,
          attempt: unavailableAttempt,
          retryDelayMs,
          causeMessage: getErrorCauseMessage(cause.cause),
        },
        "Service bootstrap endpoint unavailable; retrying",
      );
      await delay(retryDelayMs);
      continue;
    }

    if (
      settled.payload !== undefined &&
      Value.Check(ServiceBootstrapFailureSchema, settled.payload)
    ) {
      const failure = settled.payload as ServiceBootstrapFailure;
      if (
        failure.reason === "iat_out_of_range" &&
        typeof failure.serverNow === "number"
      ) {
        args.identityAuth.setServerClockOffsetMs(
          estimateMidpointClockOffsetMs({
            requestStartedAtMs: settled.requestStartedAtMs,
            responseReceivedAtMs: settled.responseReceivedAtMs,
            serverNowSeconds: failure.serverNow / 1_000,
          }),
        );
        continue;
      }
      if (
        failure.reason === "authority_update_required" ||
        failure.reason === "authority_migration_required" ||
        failure.reason === "authority_reconciliation_pending"
      ) {
        const retryDelayMs = bootstrapRetryDelayMs(settled.response);
        const pendingKey = failure.planId ?? failure.requestId ??
          `${failure.deploymentId ?? "unknown"}:${args.contractDigest}`;
        if (!loggedPendingRequests.has(pendingKey)) {
          loggedPendingRequests.add(pendingKey);
          args.log.info(
            {
              service: args.serviceName,
              deploymentId: failure.deploymentId,
              planId: failure.planId,
              contractId: args.contractId,
              contractDigest: args.contractDigest,
              retryDelayMs,
            },
            failure.message ??
              "Service deployment authority pending; waiting for approval or reconciliation",
          );
        }
        await delay(retryDelayMs);
        continue;
      }
      if (failure.reason === "contract_activation_pending") {
        const retryDelayMs = bootstrapRetryDelayMs(settled.response);
        const pendingKey = failure.requestId ??
          `${failure.deploymentId ?? "unknown"}:${args.contractDigest}`;
        if (!loggedPendingRequests.has(pendingKey)) {
          loggedPendingRequests.add(pendingKey);
          args.log.info(
            {
              service: args.serviceName,
              deploymentId: failure.deploymentId,
              requestId: failure.requestId,
              contractId: args.contractId,
              contractDigest: args.contractDigest,
              dependencyAlias: failure.dependencyAlias,
              dependencyContractId: failure.dependencyContractId,
              dependencySurface: failure.dependencySurface,
              dependencyReason: failure.dependencyReason,
              dependencyKey: failure.dependencyKey,
              retryDelayMs,
            },
            dependencyWaitLogMessage(failure),
          );
        }
        await delay(retryDelayMs);
        continue;
      }
      throw new TransportError({
        code: "trellis.bootstrap.failed",
        message: `Service bootstrap failed: ${
          failure.message ?? failure.reason
        }`,
        hint:
          "Retry the connection. If it keeps failing, check Trellis bootstrap availability and contract activation.",
        context: {
          trellisUrl: args.trellisUrl,
          contractId: args.contractId,
          contractDigest: args.contractDigest,
          status: settled.response.status,
          reason: failure.reason,
        },
      });
    }

    const bootstrapState =
      settled.payload && typeof settled.payload === "object"
        ? (settled.payload as { state?: unknown }).state
        : undefined;
    if (
      bootstrapState === "authority_pending" ||
      bootstrapState === "migration_required" ||
      bootstrapState === "dependency_pending" ||
      bootstrapState === "resource_pending"
    ) {
      const retryDelayMs = bootstrapRetryDelayMs(settled.response);
      args.log.info(
        {
          service: args.serviceName,
          contractId: args.contractId,
          contractDigest: args.contractDigest,
          state: bootstrapState,
          retryDelayMs,
        },
        "Service deployment authority pending",
      );
      await delay(retryDelayMs);
      continue;
    }

    if (settled.payload === undefined) {
      throw new TransportError({
        code: "trellis.bootstrap.invalid_response",
        message: "Service bootstrap returned invalid JSON.",
        hint:
          "Retry the connection. If it keeps happening, check the Trellis deployment.",
        context: {
          trellisUrl: args.trellisUrl,
          contractId: args.contractId,
          contractDigest: args.contractDigest,
        },
      });
    }

    const response = Value.Parse(
      ServiceBootstrapReadySchema,
      settled.payload,
    );
    const serverClockOffsetMs = estimateMidpointClockOffsetMs({
      requestStartedAtMs: settled.requestStartedAtMs,
      responseReceivedAtMs: settled.responseReceivedAtMs,
      serverNowSeconds: response.serverNow / 1_000,
    });
    args.identityAuth.setServerClockOffsetMs(serverClockOffsetMs);
    args.sessionAuth.setServerClockOffsetMs(serverClockOffsetMs);
    const native = response.nats.transports.native;
    if (!native) {
      throw new TransportError({
        code: "trellis.bootstrap.invalid_response",
        message: "Service bootstrap returned no native NATS transport.",
        hint: "Configure native NATS endpoints for the Trellis runtime.",
      });
    }
    return {
      status: "ready",
      serverNow: response.serverNow / 1_000,
      serverClockOffsetMs,
      connectInfo: {
        sessionId: response.session.sessionId,
        participantDigest: response.authorization.participantArtifactDigest,
        instanceId: args.identity.instanceId,
        deploymentId: args.identity.deploymentId,
        contractId: args.contractId,
        contractDigest: args.contractDigest,
        transports: { native: { natsServers: native.natsServers } },
        jwt: response.nats.jwt,
        jwtExpiresAt: response.nats.jwtExpiresAt,
        authorizationContext: response.authorizationContext,
      },
      binding: {
        contractId: args.contractId,
        digest: response.authorization.participantArtifactDigest,
        resources: {
          kv: response.authorization.resourceRuntime.kv ?? {},
          store: response.authorization.resourceRuntime.store ?? {},
          ...(response.authorization.resourceRuntime.jobs === undefined
            ? {}
            : { jobs: response.authorization.resourceRuntime.jobs }),
          ...(response.authorization.resourceRuntime.eventConsumers ===
              undefined
            ? {}
            : {
              eventConsumers:
                response.authorization.resourceRuntime.eventConsumers,
            }),
        },
      },
    };
  }
}

/** Drains a NATS connection created by an unsuccessful bootstrap attempt. */
export async function closeFailedServiceBootstrapConnection(
  nc: NatsConnection,
): Promise<void> {
  if (nc.isClosed()) {
    return;
  }

  try {
    await nc.drain();
  } catch {
    await nc.closed().catch(() => undefined);
  }
}
