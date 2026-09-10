import type { NatsConnection } from "@nats-io/nats-core";
export { checkDeviceActivation } from "./device/activation.ts";
export type {
  CheckDeviceActivationArgs,
  TrellisDeviceActivatedStatus,
  TrellisDeviceActivationRequiredStatus,
  TrellisDeviceActivationStatus,
  TrellisDeviceNotReadyStatus,
} from "./device/activation.ts";
import {
  AsyncResult,
  type BaseError,
  Result,
  UnexpectedError,
} from "@qlever-llc/result";
import { ulid } from "ulid";
import { decodeTrellisHttpError, TrellisHttpError } from "./auth/http_error.ts";
import {
  type ContractResourceBindings,
  ContractResourceBindingsSchema,
} from "./participant.ts";

import {
  deriveDeviceConfirmationCode,
  deriveDeviceIdentity,
  requestDeviceEnrollment,
  verifyDeviceConfirmationCode,
  waitForDeviceActivation,
} from "./auth/device_activation.ts";
import {
  base64urlDecode,
  base64urlEncode,
  sha256,
  utf8,
} from "./auth/utils.ts";
import { estimateMidpointClockOffsetMs } from "./auth/time.ts";
import { createAuth } from "./auth/session_auth.ts";
import { AuthorizationContextRefreshResponseSchema } from "./auth/authorization/types.ts";
import type { RuntimeApi } from "./participant_runtime/api.ts";
import {
  DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
  loadDefaultRuntimeTransport,
  selectRuntimeTransportServers,
} from "./runtime_transport.ts";
import {
  type ServiceHealth,
  ServiceHealthRuntime,
} from "./service/runtime/health.ts";
import { publishHealthHeartbeatSample } from "./health_transport.ts";
import { type RuntimeStateStoresForContract, Trellis } from "./session.ts";
import { logger as noopLogger, type LoggerLike } from "./globals.ts";
import { TransportError } from "./errors/index.ts";
import { Type } from "typebox";
import { Value } from "typebox/value";
import {
  installConnectionAvailability,
  observeNatsTrellisConnection,
} from "./connection.ts";
import {
  type AuthorizationContextBundle,
  AuthorizationContextCache,
  AuthorizationContextRefreshError,
  AuthorizationProviderCache,
  startAuthorizationContextRefresh,
} from "./auth/authorization_context.ts";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import {
  type GeneratedParticipant,
  getParticipantRuntime,
  participantAvailability,
  participantEvidence,
} from "./participant_runtime/participant.ts";

type DeviceContract = GeneratedParticipant;

type RuntimeStateShape = Record<
  string,
  { kind: "value" | "map"; value: unknown }
>;
type BroadStateStore = {
  get(...args: unknown[]): AsyncResult<unknown, BaseError>;
  put(...args: unknown[]): AsyncResult<unknown, BaseError>;
  list(...args: unknown[]): AsyncResult<unknown, BaseError>;
  delete(...args: unknown[]): AsyncResult<unknown, BaseError>;
};
type BroadStateFacade = Record<string, BroadStateStore>;

function deviceConnectResult<T>(
  promise: Promise<T>,
): AsyncResult<T, TransportError | UnexpectedError> {
  return AsyncResult.from(
    promise.then(
      (
        value,
      ): Result<T, TransportError | UnexpectedError> => Result.ok(value),
      (
        cause,
      ): Result<T, TransportError | UnexpectedError> =>
        Result.err(
          cause instanceof TransportError
            ? cause
            : new UnexpectedError({ cause }),
        ),
    ),
  );
}

export type TrellisDeviceConnection<
  TContract extends DeviceContract = DeviceContract,
> = CallerRuntime<TContract> & {
  readonly health: ServiceHealth;
};

type DeviceConnectTransport = {
  connect(options: {
    servers: string | string[];
    token?: string;
    authenticator?: unknown;
    inboxPrefix?: string;
    maxReconnectAttempts?: number;
    ignoreAuthErrorAbort?: boolean;
    timeout?: number;
  }): Promise<NatsConnection>;
};

type DeviceConnectDeps = {
  loadTransport(): Promise<DeviceConnectTransport>;
  now(): number;
};

export type TrellisDevicePendingActivationState = {
  status: "pending";
  participantId: string;
  publicIdentityKey: string;
  instanceId: string;
  deploymentId: string;
  flowId: string;
  nonce: string;
  activationUrl: string;
};

export type TrellisDeviceActivatedActivationState = {
  status: "activated";
  participantId: string;
  publicIdentityKey: string;
  instanceId: string;
  deploymentId: string;
  flowId: string;
  nonce: string;
  activationUrl: string;
};

export type TrellisDeviceLocalActivationState =
  | TrellisDevicePendingActivationState
  | TrellisDeviceActivatedActivationState;

export type TrellisDeviceActivationSession<
  TState extends TrellisDeviceLocalActivationState =
    TrellisDeviceLocalActivationState,
> = {
  activationUrl: string;
  localState: TState;
  waitForOnlineApproval(opts?: {
    signal?: AbortSignal;
  }): Promise<TrellisDeviceActivatedActivationState>;
  acceptConfirmationCode(
    code: string,
  ): Promise<TrellisDeviceActivatedActivationState>;
};

export type TrellisDeviceActivationArgs<
  TContract extends DeviceContract = DeviceContract,
> = {
  trellisUrl: string;
  participant: TContract;
  rootSecret: Uint8Array | string;
  provisioningSecret?: string;
};

/** Provisioned device identity and exact deployment participant binding. */
export type TrellisDeviceProvisionedIdentity = {
  deploymentId: string;
  instanceId: string;
  principalId: string;
  participantId: string;
  participantDigest: string;
  participantNeedsDigest: string;
  provisioningSecret?: string;
  expectedSecretVersion?: number;
};

export type TrellisDeviceResumeActivationArgs<
  TContract extends DeviceContract = DeviceContract,
> = TrellisDeviceActivationArgs<TContract> & {
  localState: TrellisDeviceLocalActivationState;
};

export type TrellisDeviceConnectArgs<
  TContract extends DeviceContract = DeviceContract,
> = {
  trellisUrl: string;
  participant: TContract;
  rootSecret: Uint8Array | string;
  log?: LoggerLike | false;
};

const DeviceBootstrapReadySchema = Type.Object({
  ...AuthorizationContextRefreshResponseSchema.properties,
  authorization: Type.Object({
    participantId: Type.String({ minLength: 1 }),
    participantDigest: Type.String({ minLength: 1 }),
    resourceRuntime: ContractResourceBindingsSchema,
  }),
});

type DeviceBootstrapReady = {
  status: "ready";
  connectInfo: {
    connectionId: string;
    participantId: string;
    participantDigest: string;
    transports: {
      native?: { natsServers: string[] };
      websocket?: { natsServers: string[] };
    };
    transport: { jwt: string; jwtExpiresAt: number; inboxPrefix: string };
    authorizationContext: AuthorizationContextBundle;
    apiBindings: Readonly<Record<string, unknown>>;
    resourceBindings: ContractResourceBindings;
  };
  sessionAuth: Awaited<ReturnType<typeof createAuth>>;
};
type DeviceBootstrapResponse = DeviceBootstrapReady;
type ResolvedDeviceConnectInfo = DeviceBootstrapReady["connectInfo"];
type DeviceClockOffsetState = {
  serverClockOffsetMs: number;
};

function normalizeRootSecret(rootSecret: Uint8Array | string): Uint8Array {
  if (typeof rootSecret === "string") {
    const decoded = base64urlDecode(rootSecret.trim());
    if (decoded.length === 0) throw new Error("rootSecret must not be empty");
    return decoded;
  }
  if (rootSecret.length === 0) throw new Error("rootSecret must not be empty");
  return rootSecret;
}

const defaultDeps: DeviceConnectDeps = {
  loadTransport: loadDefaultRuntimeTransport,
  now: () => Date.now(),
};

function transportCauseContext(cause: unknown): Record<string, unknown> {
  if (cause instanceof Error) {
    return { causeName: cause.name, causeMessage: cause.message };
  }

  return { cause: String(cause) };
}

function createTransportError(args: {
  code: string;
  message: string;
  hint: string;
  context?: Record<string, unknown>;
  cause?: unknown;
}): TransportError {
  return new TransportError({
    code: args.code,
    message: args.message,
    hint: args.hint,
    cause: args.cause,
    context: {
      ...(args.context ?? {}),
      ...(args.cause === undefined ? {} : transportCauseContext(args.cause)),
    },
  });
}

async function readJsonResponse(
  response: Response,
  args: {
    code: string;
    message: string;
    hint: string;
    context?: Record<string, unknown>;
  },
): Promise<unknown> {
  try {
    return await response.json();
  } catch (cause) {
    throw createTransportError({
      ...args,
      cause,
    });
  }
}

function resolveDeviceLogger(log?: LoggerLike | false): LoggerLike {
  if (log === false) {
    return noopLogger;
  }

  return log ?? noopLogger;
}

function createInvalidConfirmationCodeTransportError(
  context?: Record<string, unknown>,
) {
  return createTransportError({
    code: "trellis.device.invalid_confirmation_code",
    message: "The device confirmation code is invalid.",
    hint:
      "Retry with the current confirmation code for this activation, or restart activation if the code is no longer valid.",
    context,
  });
}

function createActivatedLocalState(
  localState: TrellisDeviceLocalActivationState,
): TrellisDeviceActivatedActivationState {
  return {
    ...localState,
    status: "activated",
  };
}

function assertActivationStateMatchesIdentity(args: {
  localState: TrellisDeviceLocalActivationState;
  publicIdentityKey: string;
}): void {
  if (args.localState.publicIdentityKey !== args.publicIdentityKey) {
    throw createTransportError({
      code: "trellis.device.activation_state_mismatch",
      message:
        "Local device activation state does not match the provided root secret.",
      hint:
        "Use the activation state for the same device identity, or start a new activation for this root secret.",
      context: {
        statePublicIdentityKey: args.localState.publicIdentityKey,
        publicIdentityKey: args.publicIdentityKey,
      },
    });
  }
}

function assertActivationStateMatchesContract(args: {
  localState: TrellisDeviceLocalActivationState;
  participantId: string;
}): void {
  if (args.localState.participantId !== args.participantId) {
    throw createTransportError({
      code: "trellis.device.activation_state_contract_mismatch",
      message:
        "Local device activation state does not match the requested device participant.",
      hint:
        "Use activation state for the same device contract, or start activation again for this contract digest.",
      context: {
        stateParticipantId: args.localState.participantId,
        participantId: args.participantId,
      },
    });
  }
}

function createActivationSession<
  TLocalState extends TrellisDeviceLocalActivationState,
>(args: {
  trellisUrl: string;
  participantId: string;
  identity: Awaited<ReturnType<typeof deriveDeviceIdentity>>;
  provisioned: Pick<
    TrellisDeviceProvisionedIdentity,
    "participantId" | "provisioningSecret"
  >;
  participant: DeviceContract;
  now: () => number;
  localState: TLocalState;
  sessionIdentity?: Awaited<ReturnType<typeof createAuth>>;
  connectionId?: string;
}): TrellisDeviceActivationSession<TLocalState> {
  assertActivationStateMatchesIdentity({
    localState: args.localState,
    publicIdentityKey: args.identity.publicIdentityKey,
  });
  assertActivationStateMatchesContract({
    localState: args.localState,
    participantId: args.participantId,
  });

  const activatedState = createActivatedLocalState(args.localState);
  return {
    activationUrl: args.localState.activationUrl,
    localState: args.localState,
    waitForOnlineApproval: async (opts?: { signal?: AbortSignal }) => {
      if (args.localState.status === "activated") {
        return activatedState;
      }

      await waitForDeviceActivation({
        trellisUrl: args.trellisUrl,
        publicIdentityKey: args.identity.publicIdentityKey,
        identitySeed: args.identity.identitySeed,
        activationKey: args.identity.activationKey,
        participantId: args.provisioned.participantId,
        ...participantEvidence(args.participant),
        provisioningSecret: args.provisioned.provisioningSecret ?? undefined,
        nonce: args.localState.nonce,
        signal: opts?.signal,
        sessionIdentity: args.sessionIdentity,
        connectionId: args.connectionId,
      });
      return activatedState;
    },
    acceptConfirmationCode: async (code: string) => {
      if (args.localState.status === "activated") {
        return activatedState;
      }

      const ok = await verifyDeviceConfirmationCode({
        activationKey: args.identity.activationKey,
        publicIdentityKey: args.identity.publicIdentityKey,
        nonce: args.localState.nonce,
        confirmationCode: code,
      });
      if (!ok) {
        throw createInvalidConfirmationCodeTransportError({
          publicIdentityKey: args.identity.publicIdentityKey,
          instanceId: args.localState.instanceId,
          deploymentId: args.localState.deploymentId,
        });
      }
      return activatedState;
    },
  };
}

async function fetchDeviceBootstrap(args: {
  trellisUrl: string;
  deviceIdentity: Awaited<ReturnType<typeof deriveDeviceIdentity>>;
  participant: DeviceContract;
  now: () => number;
  offsetState: DeviceClockOffsetState;
  signal?: AbortSignal;
  sessionAuth?: Awaited<ReturnType<typeof createAuth>>;
  connectionId?: string;
}): Promise<DeviceBootstrapResponse> {
  const sessionAuth = args.sessionAuth ?? await createAuth({
    sessionKeySeed: base64urlEncode(
      crypto.getRandomValues(new Uint8Array(32)),
    ),
  });
  const requestStartedAtMs = args.now();
  const issuedAt = Math.trunc(
    requestStartedAtMs + args.offsetState.serverClockOffsetMs,
  );
  const requestId = ulid();
  const identityAuth = await createAuth({
    sessionKeySeed: base64urlEncode(args.deviceIdentity.identitySeed),
  });
  const deviceIdentityKeyId = base64urlEncode(
    await sha256(base64urlDecode(identityAuth.sessionKey)),
  );
  const unsigned = {
    identityKeyId: deviceIdentityKeyId,
    sessionKey: sessionAuth.sessionKey,
    connectionId: args.connectionId ?? ulid(),
    requestId,
    iat: issuedAt,
    name: args.participant.identity,
    ...participantEvidence(args.participant),
  };
  const response = await fetch(
    new URL("/bootstrap/device", args.trellisUrl),
    {
      method: "POST",
      signal: args.signal,
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        ...unsigned,
        proof: await identityAuth.signSessionProof({
          purpose: "deviceBootstrap",
          origin: new URL(args.trellisUrl).origin,
          unsignedRequest: unsigned,
        }),
      }),
    },
  );
  const responseReceivedAtMs = args.now();
  if (!response.ok) throw await decodeTrellisHttpError(response);
  const payload = await readJsonResponse(response, {
    code: "trellis.bootstrap.invalid_response",
    message: "Trellis returned an invalid device bootstrap response.",
    hint: "Retry the connection or complete device activation.",
    context: { trellisUrl: args.trellisUrl },
  });
  const ready = Value.Parse(DeviceBootstrapReadySchema, payload);
  args.offsetState.serverClockOffsetMs = estimateMidpointClockOffsetMs({
    requestStartedAtMs,
    responseReceivedAtMs,
    serverNowSeconds: ready.serverNow / 1_000,
  });
  sessionAuth.setServerClockOffsetMs(
    args.offsetState.serverClockOffsetMs + args.now() - Date.now(),
  );
  return {
    status: "ready",
    sessionAuth,
    connectInfo: {
      connectionId: ready.runtime.connectionId,
      participantId: ready.runtime.participantId,
      participantDigest: ready.authorization.participantDigest,
      transports: ready.transports,
      transport: {
        jwt: ready.routing.bootstrapJwt,
        jwtExpiresAt: ready.routing.bootstrapJwtExpiresAt,
        inboxPrefix: ready.runtime.inboxPrefix,
      },
      authorizationContext: ready.authorizationContext,
      apiBindings: ready.apiBindings,
      resourceBindings: ready.authorization.resourceRuntime,
    },
  };
}

/**
 * @internal Exported for focused tests and platform-specific wrappers.
 */
export async function startDeviceActivationWithDeps<
  TContract extends DeviceContract,
>(
  args: TrellisDeviceActivationArgs<TContract>,
  deps: Pick<DeviceConnectDeps, "now">,
): Promise<
  TrellisDeviceActivationSession<TrellisDevicePendingActivationState>
> {
  const rootSecret = normalizeRootSecret(args.rootSecret);
  const identity = await deriveDeviceIdentity(rootSecret);
  const nonce = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const sessionIdentity = await createAuth({
    sessionKeySeed: base64urlEncode(
      crypto.getRandomValues(new Uint8Array(32)),
    ),
  });
  const connectionId = ulid();
  const activation = await requestDeviceEnrollment({
    trellisUrl: args.trellisUrl,
    publicIdentityKey: identity.publicIdentityKey,
    identitySeed: identity.identitySeed,
    sessionIdentity,
    connectionId,
    participantId: args.participant.identity,
    ...participantEvidence(args.participant),
    provisioningSecret: args.provisioningSecret,
    challengeDigest: base64urlEncode(await sha256(utf8(nonce))),
    confirmationCode: await deriveDeviceConfirmationCode({
      activationKey: identity.activationKey,
      publicIdentityKey: identity.publicIdentityKey,
      nonce,
    }),
  });
  const activationState = activation.activation as
    | Record<string, unknown>
    | null;
  if (
    activation.state !== "pending" ||
    typeof activationState?.reviewId !== "string" ||
    typeof activationState.activationUrl !== "string" ||
    typeof activationState.principalId !== "string" ||
    typeof activationState.deploymentId !== "string" ||
    typeof activationState.instanceId !== "string"
  ) {
    throw createTransportError({
      code: "device_activation_unavailable",
      message: "The device does not require activation.",
      hint: "Connect the device directly.",
      context: { status: activation.state },
    });
  }
  return await createActivationSession({
    trellisUrl: args.trellisUrl,
    participantId: args.participant.identity,
    identity,
    provisioned: {
      participantId: args.participant.identity,
      provisioningSecret: args.provisioningSecret,
    },
    participant: args.participant,
    now: deps.now,
    localState: {
      status: "pending",
      participantId: args.participant.identity,
      publicIdentityKey: identity.publicIdentityKey,
      instanceId: activationState.instanceId,
      deploymentId: activationState.deploymentId,
      flowId: activationState.reviewId,
      nonce,
      activationUrl: activationState.activationUrl,
    },
    sessionIdentity,
    connectionId,
  });
}

/**
 * @internal Exported for focused tests and platform-specific wrappers.
 */
export async function resumeDeviceActivationWithDeps<
  TLocalState extends TrellisDeviceLocalActivationState,
  TContract extends DeviceContract,
>(
  args:
    & TrellisDeviceResumeActivationArgs<TContract>
    & {
      localState: TLocalState;
    },
  deps: Pick<DeviceConnectDeps, "now">,
): Promise<TrellisDeviceActivationSession<TLocalState>> {
  const rootSecret = normalizeRootSecret(args.rootSecret);
  const identity = await deriveDeviceIdentity(rootSecret);

  return await createActivationSession({
    trellisUrl: args.trellisUrl,
    participantId: args.participant.identity,
    identity,
    provisioned: { participantId: args.participant.identity },
    participant: args.participant,
    now: deps.now,
    localState: args.localState,
  });
}

/**
 * @internal Exported for focused tests; applications should use
 * `TrellisDevice.connect`.
 */
export async function connectDeviceWithDeps<
  TContract extends DeviceContract,
>(
  args: TrellisDeviceConnectArgs<TContract>,
  deps: DeviceConnectDeps,
): Promise<
  TrellisDeviceConnection<TContract>
> {
  const log = resolveDeviceLogger(args.log);
  const rootSecret = normalizeRootSecret(args.rootSecret);
  const identity = await deriveDeviceIdentity(rootSecret);
  const offsetState: DeviceClockOffsetState = { serverClockOffsetMs: 0 };
  const bootstrap = await fetchDeviceBootstrap({
    trellisUrl: args.trellisUrl,
    deviceIdentity: identity,
    participant: args.participant,
    now: deps.now,
    offsetState,
  });

  const connectInfo = bootstrap.connectInfo;

  const transport = await deps.loadTransport();
  const authorizationContexts = new AuthorizationContextCache(
    args.trellisUrl,
    (input, init) => globalThis.fetch(input, init),
    deps.now,
  );
  authorizationContexts.setServerClockOffsetMs(
    offsetState.serverClockOffsetMs,
  );
  await authorizationContexts.install(connectInfo.authorizationContext, {
    bootstrapJwt: connectInfo.transport.jwt,
    bootstrapJwtExpiresAt: connectInfo.transport.jwtExpiresAt,
  });
  const verifiedContext = authorizationContexts.current();
  if (verifiedContext.context.participantId !== args.participant.identity) {
    throw new Error(
      "device authorization context belongs to another participant",
    );
  }
  const deploymentId = verifiedContext.context.deploymentId;
  const instanceId = verifiedContext.context.instanceId;
  if (!deploymentId || !instanceId) {
    throw new Error(
      "device authorization context is missing its deployment assignment",
    );
  }
  const sessionOptions = await bootstrap.sessionAuth.natsConnectOptions({
    sessionId: connectInfo.connectionId,
    contextDigest: () => authorizationContexts.current().contextDigest,
    jwt: () => authorizationContexts.routingJwt(),
  });
  let nc: NatsConnection | undefined;
  let authorizationProviderCache: AuthorizationProviderCache | undefined;
  try {
    nc = await transport.connect({
      servers: selectRuntimeTransportServers(connectInfo.transports),
      maxReconnectAttempts: DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
      ignoreAuthErrorAbort: true,
      timeout: 10_000,
      inboxPrefix: connectInfo.transport.inboxPrefix,
      authenticator: sessionOptions.authenticator,
    });
    const connectedNats = nc;
    authorizationProviderCache = await AuthorizationProviderCache.attach(
      connectedNats,
      authorizationContexts.bundle().authorizationRegistry,
      connectInfo.transport.inboxPrefix,
      authorizationContexts,
    );
    authorizationProviderCache.start();
    await authorizationProviderCache.waitReady();
    void connectedNats.closed().finally(() => {
      authorizationProviderCache?.stop();
    });
  } catch (cause) {
    authorizationProviderCache?.stop();
    if (nc && !nc.isClosed()) await nc.close();
    throw createTransportError({
      code: "trellis.runtime.connect_failed",
      message: "Trellis could not open the device runtime connection.",
      hint:
        "Retry the connection. If it keeps failing, check Trellis transport availability.",
      cause,
      context: { participantId: args.participant.identity },
    });
  }

  if (!nc || !authorizationProviderCache) {
    throw new Error("Trellis device runtime connection was not established");
  }

  const connection = observeNatsTrellisConnection({
    kind: "device",
    nc,
    availability: participantAvailability(
      args.participant,
      connectInfo.apiBindings,
      connectInfo.resourceBindings,
    ),
    log: false,
    lifecycleLog: {
      log,
      context: { participantId: args.participant.identity },
    },
  });
  connection.subscribe((status) =>
    authorizationProviderCache.observeConnectionPhase(status.phase)
  );
  const stopContextRefresh = startAuthorizationContextRefresh({
    trellisUrl: args.trellisUrl,
    sessionId: connectInfo.connectionId,
    auth: bootstrap.sessionAuth,
    cache: authorizationContexts,
    refresh: async (shouldInstall) => {
      try {
        const next = await fetchDeviceBootstrap({
          trellisUrl: args.trellisUrl,
          deviceIdentity: identity,
          participant: args.participant,
          now: deps.now,
          offsetState,
          sessionAuth: bootstrap.sessionAuth,
          connectionId: connectInfo.connectionId,
        });
        authorizationContexts.setServerClockOffsetMs(
          offsetState.serverClockOffsetMs,
        );
        const context = await authorizationContexts.install(
          next.connectInfo.authorizationContext,
          {
            bootstrapJwt: next.connectInfo.transport.jwt,
            bootstrapJwtExpiresAt: next.connectInfo.transport.jwtExpiresAt,
          },
          undefined,
          shouldInstall,
        );
        installConnectionAvailability(
          connection,
          participantAvailability(
            args.participant,
            next.connectInfo.apiBindings,
            next.connectInfo.resourceBindings,
          ),
        );
        return context;
      } catch (error) {
        if (error instanceof TrellisHttpError) {
          throw new AuthorizationContextRefreshError(error.status, error.code);
        }
        throw error;
      }
    },
    onRefresh: () =>
      connection.status.phase !== "connected" ? nc.reconnect() : undefined,
    onTerminalFailure: async () => {
      if (!nc.isClosed()) await nc.drain();
    },
  });
  void nc.closed().then(stopContextRefresh, stopContextRefresh);

  const trellis = new Trellis<
    RuntimeApi,
    "client",
    RuntimeStateStoresForContract<TContract>
  >(
    args.participant.identity,
    nc,
    {
      sessionKey: bootstrap.sessionAuth.sessionKey,
      sign: bootstrap.sessionAuth.sign,
      contextDigest: () => authorizationContexts.current().contextDigest,
      authorizationProviderCache,
    },
    {
      log,
      api: getParticipantRuntime(args.participant).api as RuntimeApi,
      state: getParticipantRuntime(args.participant).state,
      connection,
    },
    connectInfo.transport.inboxPrefix,
  );

  const health = new ServiceHealthRuntime({
    serviceName: args.participant.identity,
    kind: "device",
    instanceId,
    contractId: connectInfo.participantId,
    contractDigest: connectInfo.participantDigest,
    publishIntervalMs: 30_000,
  });
  health.setInfo({
    info: {
      deploymentId,
    },
  });
  health.add("nats", () => ({
    status: nc.isClosed() ? "failed" : "ok",
    ...(nc.isClosed() ? { summary: "NATS connection closed" } : {}),
  }));

  let heartbeatTimer: ReturnType<typeof setInterval> | undefined;
  let publishingHeartbeat = false;
  const stopHeartbeat = () => {
    if (heartbeatTimer !== undefined) {
      clearInterval(heartbeatTimer);
      heartbeatTimer = undefined;
    }
  };
  const publishHeartbeat = async (): Promise<void> => {
    if (publishingHeartbeat) {
      return;
    }

    publishingHeartbeat = true;
    try {
      await publishHealthHeartbeatSample({
        nc,
        identity: {
          sessionKey: bootstrap.sessionAuth.sessionKey,
          participantKind: "device",
          contractId: connectInfo.participantId,
          contractDigest: connectInfo.participantDigest,
          deploymentId,
          instanceId,
        },
        sample: await health.sample(),
      });
    } catch (error) {
      log.warn({ error }, "Failed to build or publish health heartbeat");
    } finally {
      publishingHeartbeat = false;
    }
  };

  await publishHeartbeat();
  heartbeatTimer = setInterval(() => {
    void publishHeartbeat();
  }, health.publishIntervalMs);
  void nc.closed().finally(stopHeartbeat);

  return Object.assign(createCallerRuntime(trellis, args.participant), {
    health,
  });
}

export const TrellisDevice = {
  connect<
    TContract extends DeviceContract,
  >(
    args: TrellisDeviceConnectArgs<TContract>,
  ): AsyncResult<
    TrellisDeviceConnection<TContract>,
    TransportError | UnexpectedError
  > {
    return deviceConnectResult(connectDeviceWithDeps(args, defaultDeps));
  },
};
