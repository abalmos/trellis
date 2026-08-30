import type { NatsConnection } from "@nats-io/nats-core";
import {
  AsyncResult,
  type BaseError,
  Result,
  UnexpectedError,
} from "@qlever-llc/result";
import { ulid } from "ulid";
import {
  CONTRACT_STATE_METADATA,
  type ContractStateMetadata,
} from "./contract_support/mod.ts";
import { resolveNativeProtocolPresentation } from "./contract_support/protocol_resolution.ts";

import {
  deriveDeviceConfirmationCode,
  deriveDeviceIdentity,
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
import {
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigest,
} from "./auth/session_proof.ts";
import type { RuntimeApi } from "./contract_support/runtime.ts";
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
import { type StaticDecode, Type } from "typebox";
import { Value } from "typebox/value";
import { observeNatsTrellisConnection } from "./connection.ts";
import {
  type AuthorizationContextBundle,
  AuthorizationContextBundleSchema,
  AuthorizationContextCache,
  type AuthorizationContextPersistence,
  AuthorizationProviderCache,
  MemoryAuthorizationContextStore,
  startAuthorizationContextRefresh,
} from "./auth/authorization_context.ts";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import {
  type ContractWithRuntime,
  getContractRuntime,
} from "./contract_support/contract_runtime.ts";

type DeviceContract<
  TContract extends {
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  } = {
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  },
> = ContractWithRuntime & {
  readonly CONTRACT_ID: string;
  readonly CONTRACT_DIGEST: string;
  readonly API: Readonly<Record<string, unknown>>;
  readonly API_DIGEST: string;
  readonly PARTICIPANT: Readonly<Record<string, unknown>>;
  readonly [CONTRACT_STATE_METADATA]?: ContractStateMetadata;
};

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
  participantDigest: string;
  publicIdentityKey: string;
  instanceId: string;
  deploymentId: string;
  flowId: string;
  nonce: string;
  activationUrl: string;
};

export type TrellisDeviceActivatedActivationState = {
  status: "activated";
  participantDigest: string;
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
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }> = DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
> = {
  trellisUrl: string;
  contract: TContract;
  rootSecret: Uint8Array | string;
  identity: TrellisDeviceProvisionedIdentity;
};

/** Provisioned device identity and exact deployment participant binding. */
export type TrellisDeviceProvisionedIdentity = {
  deploymentId: string;
  instanceId: string;
  principalId: string;
  participantId: string;
  participantArtifactDigest: string;
  participantNeedsDigest: string;
  provisioningSecret?: string;
  expectedSecretVersion?: number;
};

export type TrellisDeviceResumeActivationArgs<
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }> = DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
> = TrellisDeviceActivationArgs<TContract> & {
  localState: TrellisDeviceLocalActivationState;
};

export type TrellisDeviceConnectArgs<
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }> = DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
> = {
  trellisUrl: string;
  contract: TContract;
  rootSecret: Uint8Array | string;
  identity: TrellisDeviceProvisionedIdentity;
  log?: LoggerLike | false;
} & AuthorizationContextPersistence;

const DeviceBootstrapReadySchema = Type.Object({
  state: Type.Literal("ready"),
  serverNow: Type.Integer(),
  session: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    inboxPrefix: Type.String({ minLength: 1 }),
  }, { additionalProperties: true }),
  authorization: Type.Object({
    participantArtifactDigest: Type.String({ minLength: 1 }),
  }, { additionalProperties: true }),
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
});

const DeviceBootstrapActivationRequiredSchema = Type.Object({
  state: Type.Literal("activation_pending"),
  serverNow: Type.Integer(),
  activation: Type.Object({
    reviewId: Type.String({ minLength: 1 }),
    activationUrl: Type.String({ minLength: 1 }),
  }, { additionalProperties: true }),
});

const DeviceBootstrapNotReadySchema = Type.Object({
  state: Type.Union([
    Type.Literal("authority_pending"),
    Type.Literal("authority_rejected"),
    Type.Literal("migration_required"),
    Type.Literal("dependency_pending"),
    Type.Literal("resource_pending"),
    Type.Literal("disabled"),
    Type.Literal("activation_rejected"),
  ]),
  serverNow: Type.Integer(),
});

type DeviceBootstrapReady = {
  status: "ready";
  connectInfo: {
    sessionId: string;
    instanceId: string;
    deploymentId: string;
    participantId: string;
    participantDigest: string;
    transports: {
      native?: { natsServers: string[] };
      websocket?: { natsServers: string[] };
    };
    transport: { jwt: string; jwtExpiresAt: number; inboxPrefix: string };
    authorizationContext: AuthorizationContextBundle;
  };
  sessionAuth: Awaited<ReturnType<typeof createAuth>>;
};
type DeviceBootstrapActivationRequired = {
  status: "activation_required";
  reviewId: string;
  activationUrl: string;
};
type DeviceBootstrapNotReady = { status: "not_ready"; reason: string };
type DeviceBootstrapResponse =
  | DeviceBootstrapReady
  | DeviceBootstrapActivationRequired
  | DeviceBootstrapNotReady;
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

function assertBootstrapContractMatches(args: {
  participantId: string;
  participantDigest: string;
  connectInfo: ResolvedDeviceConnectInfo;
}): void {
  if (
    args.connectInfo.participantId !== args.participantId ||
    args.connectInfo.participantDigest !== args.participantDigest
  ) {
    throw createTransportError({
      code: "trellis.bootstrap.participant_mismatch",
      message:
        "Trellis returned connection details for a different device participant.",
      hint:
        "Retry the connection. If it keeps happening, check the requested device participant and Trellis activation state.",
      context: {
        requestedParticipantId: args.participantId,
        requestedParticipantDigest: args.participantDigest,
        returnedParticipantId: args.connectInfo.participantId,
        returnedParticipantDigest: args.connectInfo.participantDigest,
      },
    });
  }
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

function createActivationRequiredTransportError(
  context?: Record<string, unknown>,
) {
  return createTransportError({
    code: "trellis.bootstrap.activation_required",
    message: "Trellis requires device activation before connecting.",
    hint:
      "Start or resume device activation, then retry the runtime connection after activation completes.",
    context,
  });
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
  participantDigest: string;
}): void {
  if (args.localState.participantDigest !== args.participantDigest) {
    throw createTransportError({
      code: "trellis.device.activation_state_contract_mismatch",
      message:
        "Local device activation state does not match the requested device participant.",
      hint:
        "Use activation state for the same device contract, or start activation again for this contract digest.",
      context: {
        stateParticipantDigest: args.localState.participantDigest,
        participantDigest: args.participantDigest,
      },
    });
  }
}

function createActivationSession<
  TLocalState extends TrellisDeviceLocalActivationState,
>(args: {
  trellisUrl: string;
  participantDigest: string;
  identity: Awaited<ReturnType<typeof deriveDeviceIdentity>>;
  provisioned: TrellisDeviceProvisionedIdentity;
  contract: DeviceContract;
  now: () => number;
  localState: TLocalState;
}): TrellisDeviceActivationSession<TLocalState> {
  assertActivationStateMatchesIdentity({
    localState: args.localState,
    publicIdentityKey: args.identity.publicIdentityKey,
  });
  assertActivationStateMatchesContract({
    localState: args.localState,
    participantDigest: args.participantDigest,
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
        deploymentId: args.provisioned.deploymentId,
        instanceId: args.provisioned.instanceId,
        principalId: args.provisioned.principalId,
        participantId: args.provisioned.participantId,
        participantArtifactDigest: args.provisioned.participantArtifactDigest,
        participantNeedsDigest: args.provisioned.participantNeedsDigest,
        nonce: args.localState.nonce,
        signal: opts?.signal,
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
  provisioned: TrellisDeviceProvisionedIdentity;
  contract: DeviceContract;
  now: () => number;
  offsetState: DeviceClockOffsetState;
  activationNonce?: string;
  signal?: AbortSignal;
}): Promise<DeviceBootstrapResponse> {
  const presentation = await resolveNativeProtocolPresentation(args.contract);
  const sessionAuth = await createAuth({
    sessionKeySeed: base64urlEncode(
      crypto.getRandomValues(new Uint8Array(32)),
    ),
  });
  for (let attempt = 0; attempt < 2; attempt += 1) {
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
    const activationNonce = args.activationNonce ??
      `${args.provisioned.principalId}:${requestId}`;
    const challengeDigest = base64urlEncode(
      await sha256(utf8(activationNonce)),
    );
    if (
      args.provisioned.participantId !== args.contract.CONTRACT_ID ||
      args.provisioned.participantArtifactDigest !==
        args.contract.CONTRACT_DIGEST ||
      args.provisioned.participantNeedsDigest !==
        presentation.participantNeedsDigest
    ) {
      throw new Error(
        "Device participant identity does not match its contract",
      );
    }
    const unsigned = {
      requestId,
      issuedAt,
      deploymentId: args.provisioned.deploymentId,
      instanceId: args.provisioned.instanceId,
      deviceIdentityKeyId,
      principalId: args.provisioned.principalId,
      identityPublicKey: identityAuth.sessionKey,
      provisioningSecret: args.provisioned.provisioningSecret ?? null,
      expectedSecretVersion: args.provisioned.expectedSecretVersion ?? null,
      newSessionPublicKey: sessionAuth.sessionKey,
      newSessionNkey: sessionAuth.sessionNkey,
      participantId: args.provisioned.participantId,
      participantArtifactDigest: args.contract.CONTRACT_DIGEST,
      participantNeedsDigest: presentation.participantNeedsDigest,
      participantArtifact: presentation.participant,
      referencedApiArtifacts: [
        presentation.api,
        ...presentation.referencedApis,
      ],
      challengeDigest,
      confirmationCode: await deriveDeviceConfirmationCode({
        activationKey: args.deviceIdentity.activationKey,
        publicIdentityKey: args.deviceIdentity.publicIdentityKey,
        nonce: activationNonce,
      }),
      proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
    };
    const requestDigest = await sessionProofRequestDigest(unsigned);
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
            requestId,
            issuedAt,
            deploymentId: args.provisioned.deploymentId,
            instanceId: args.provisioned.instanceId,
            deviceIdentityKeyId,
            newSessionPublicKey: sessionAuth.sessionKey,
            newSessionNkey: sessionAuth.sessionNkey,
            participantId: args.provisioned.participantId,
            participantDigest: args.provisioned.participantArtifactDigest,
            challengeDigest,
            requestDigest,
          }),
        }),
      },
    );
    const responseReceivedAtMs = args.now();
    const payload = await readJsonResponse(response, {
      code: "trellis.bootstrap.invalid_response",
      message: "Trellis returned an invalid device bootstrap response.",
      hint: "Retry the connection or complete device activation.",
      context: { trellisUrl: args.trellisUrl },
    });
    if (!response.ok) {
      if (
        attempt === 0 && response.status === 401 &&
        payload && typeof payload === "object" &&
        typeof (payload as { serverNow?: unknown }).serverNow === "number"
      ) {
        args.offsetState.serverClockOffsetMs = estimateMidpointClockOffsetMs({
          requestStartedAtMs,
          responseReceivedAtMs,
          serverNowSeconds: (payload as { serverNow: number }).serverNow /
            1_000,
        });
        continue;
      }
      throw createTransportError({
        code: "trellis.bootstrap.failed",
        message: "Trellis could not prepare the device session.",
        hint: "Retry the connection or complete device activation.",
        context: { status: response.status, trellisUrl: args.trellisUrl },
      });
    }
    if (Value.Check(DeviceBootstrapReadySchema, payload)) {
      const ready = payload as StaticDecode<typeof DeviceBootstrapReadySchema>;
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
          sessionId: ready.session.sessionId,
          instanceId: args.provisioned.instanceId,
          deploymentId: args.provisioned.deploymentId,
          participantId: args.provisioned.participantId,
          participantDigest: ready.authorization.participantArtifactDigest,
          transports: ready.nats.transports,
          transport: {
            jwt: ready.nats.jwt,
            jwtExpiresAt: ready.nats.jwtExpiresAt,
            inboxPrefix: ready.session.inboxPrefix,
          },
          authorizationContext: ready.authorizationContext,
        },
      };
    }
    if (Value.Check(DeviceBootstrapActivationRequiredSchema, payload)) {
      const pending = payload as StaticDecode<
        typeof DeviceBootstrapActivationRequiredSchema
      >;
      return {
        status: "activation_required",
        reviewId: pending.activation.reviewId,
        activationUrl: pending.activation.activationUrl,
      };
    }
    if (Value.Check(DeviceBootstrapNotReadySchema, payload)) {
      return { status: "not_ready", reason: payload.state };
    }
    throw createTransportError({
      code: "trellis.bootstrap.invalid_response",
      message: "Trellis returned an invalid device bootstrap response.",
      hint: "Retry the connection or complete device activation.",
      context: { trellisUrl: args.trellisUrl },
    });
  }
  throw createTransportError({
    code: "trellis.bootstrap.time_sync_failed",
    message: "Trellis could not confirm the device time window.",
    hint: "Check the device clock and retry.",
    context: { trellisUrl: args.trellisUrl },
  });
}

/**
 * @internal Exported for focused tests and platform-specific wrappers.
 */
export async function startDeviceActivationWithDeps<
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
>(
  args: TrellisDeviceActivationArgs<TContract>,
  deps: Pick<DeviceConnectDeps, "now">,
): Promise<
  TrellisDeviceActivationSession<TrellisDevicePendingActivationState>
> {
  const rootSecret = normalizeRootSecret(args.rootSecret);
  const identity = await deriveDeviceIdentity(rootSecret);
  const nonce = ulid();
  const activation = await fetchDeviceBootstrap({
    trellisUrl: args.trellisUrl,
    deviceIdentity: identity,
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    offsetState: { serverClockOffsetMs: 0 },
    activationNonce: nonce,
  });
  if (activation.status !== "activation_required") {
    throw createTransportError({
      code: "device_activation_unavailable",
      message: "The device does not require activation.",
      hint: "Connect the device directly.",
      context: { status: activation.status },
    });
  }
  return await createActivationSession({
    trellisUrl: args.trellisUrl,
    participantDigest: args.contract.CONTRACT_DIGEST,
    identity,
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    localState: {
      status: "pending",
      participantDigest: args.contract.CONTRACT_DIGEST,
      publicIdentityKey: identity.publicIdentityKey,
      instanceId: args.identity.instanceId,
      deploymentId: args.identity.deploymentId,
      flowId: activation.reviewId,
      nonce,
      activationUrl: activation.activationUrl,
    },
  });
}

/**
 * @internal Exported for focused tests and platform-specific wrappers.
 */
export async function resumeDeviceActivationWithDeps<
  TLocalState extends TrellisDeviceLocalActivationState,
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
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
    participantDigest: args.contract.CONTRACT_DIGEST,
    identity,
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    localState: args.localState,
  });
}

/**
 * @internal Exported for focused tests; applications should use
 * `TrellisDevice.connect`.
 */
export async function connectDeviceWithDeps<
  TContract extends DeviceContract<{
    state?: Readonly<Record<string, unknown>>;
    schemas?: Readonly<Record<string, unknown>>;
  }>,
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
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    offsetState,
  });

  if (bootstrap.status === "activation_required") {
    throw createActivationRequiredTransportError({
      publicIdentityKey: identity.publicIdentityKey,
      contractId: args.contract.CONTRACT_ID,
    });
  }

  if (bootstrap.status === "not_ready") {
    throw createTransportError({
      code: "trellis.bootstrap.not_ready",
      message: "Trellis is not ready to connect this device.",
      hint:
        "Wait for the device to be activated and the requested deployment to become available, then try again.",
      context: { reason: bootstrap.reason },
    });
  }

  const connectInfo = bootstrap.connectInfo;
  assertBootstrapContractMatches({
    participantId: args.contract.CONTRACT_ID,
    participantDigest: args.identity.participantArtifactDigest,
    connectInfo,
  });

  const transport = await deps.loadTransport();
  if (
    !args.authorizationContextStore &&
    args.authorizationContextEphemeral !== true
  ) {
    throw new Error(
      "devices require persistent authorization context storage or explicit ephemeral mode",
    );
  }
  const authorizationContexts = new AuthorizationContextCache(
    args.trellisUrl,
    `device:${identity.publicIdentityKey}`,
    args.authorizationContextStore ?? new MemoryAuthorizationContextStore(),
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
  const sessionOptions = await bootstrap.sessionAuth.natsConnectOptions({
    sessionId: connectInfo.sessionId,
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
      authorizationContexts.bundle().trust.authorizationRegistry,
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
      context: { participantId: args.contract.CONTRACT_ID },
    });
  }

  if (!nc || !authorizationProviderCache) {
    throw new Error("Trellis device runtime connection was not established");
  }

  const connection = observeNatsTrellisConnection({
    kind: "device",
    nc,
    log: false,
    lifecycleLog: {
      log,
      context: { participantId: args.contract.CONTRACT_ID },
    },
  });
  connection.subscribe((status) =>
    authorizationProviderCache.observeConnectionPhase(status.phase)
  );
  const stopContextRefresh = startAuthorizationContextRefresh({
    trellisUrl: args.trellisUrl,
    sessionId: connectInfo.sessionId,
    auth: bootstrap.sessionAuth,
    cache: authorizationContexts,
    onRefresh: () =>
      connection.status.phase === "connected" ? undefined : nc.reconnect(),
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
    args.contract.CONTRACT_ID,
    nc,
    {
      sessionKey: bootstrap.sessionAuth.sessionKey,
      sign: bootstrap.sessionAuth.sign,
      contextDigest: () => authorizationContexts.current().contextDigest,
      authorizationProviderCache,
    },
    {
      log,
      api: getContractRuntime(args.contract).api as RuntimeApi,
      state: args.contract[CONTRACT_STATE_METADATA],
      connection,
    },
    connectInfo.transport.inboxPrefix,
  );

  const health = new ServiceHealthRuntime({
    serviceName: typeof args.contract.PARTICIPANT.displayName === "string"
      ? args.contract.PARTICIPANT.displayName
      : args.contract.CONTRACT_ID,
    kind: "device",
    instanceId: connectInfo.instanceId,
    contractId: connectInfo.participantId,
    contractDigest: connectInfo.participantDigest,
    publishIntervalMs: 30_000,
  });
  health.setInfo({
    info: {
      deploymentId: connectInfo.deploymentId,
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
          deploymentId: connectInfo.deploymentId,
          instanceId: connectInfo.instanceId,
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

  return Object.assign(createCallerRuntime(trellis, args.contract), { health });
}

export const TrellisDevice = {
  connect<
    TContract extends DeviceContract<{
      state?: Readonly<Record<string, unknown>>;
      schemas?: Readonly<Record<string, unknown>>;
    }>,
  >(
    args: TrellisDeviceConnectArgs<TContract>,
  ): AsyncResult<
    TrellisDeviceConnection<TContract>,
    TransportError | UnexpectedError
  > {
    return deviceConnectResult(connectDeviceWithDeps(args, defaultDeps));
  },
};
