import { type Authenticator, type NatsConnection } from "@nats-io/nats-core";
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
import { compileProtocolArtifacts } from "./contract_support/protocol_artifacts.ts";

import {
  deriveDeviceIdentity,
  verifyDeviceConfirmationCode,
} from "./auth/device_activation.ts";
import {
  importEd25519PrivateKeyFromSeedBase64url,
  signEd25519SeedSha256,
} from "./auth/keys.ts";
import {
  base64urlDecode,
  base64urlEncode,
  sha256,
  toArrayBuffer,
  utf8,
} from "./auth/utils.ts";
import {
  correctedIatSeconds,
  estimateMidpointClockOffsetMs,
} from "./auth/time.ts";
import {
  buildNatsConnectSignaturePayload,
  createAuth,
} from "./auth/session_auth.ts";
import {
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigestV1,
} from "./auth/session_proof.ts";
import type { RuntimeApi } from "./contract_support/runtime.ts";
import {
  DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
  loadDefaultRuntimeTransport,
  selectRuntimeTransportServers,
} from "./runtime_transport.ts";
import { type ServiceHealth, ServiceHealthRuntime } from "./server/health.ts";
import { publishHealthHeartbeatSample } from "./health_transport.ts";
import { type RuntimeStateStoresForContract, Trellis } from "./session.ts";
import { logger as noopLogger, type LoggerLike } from "./globals.ts";
import { TransferError, TransportError } from "./errors/index.ts";
import { type StaticDecode, Type } from "typebox";
import { Value } from "typebox/value";
import { observeNatsTrellisConnection } from "./connection.ts";
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
  CONTRACT_ID: string;
  CONTRACT_DIGEST: string;
  CONTRACT: TContract & {
    displayName?: string;
  };
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
  }): Promise<NatsConnection>;
};

type DeviceConnectDeps = {
  loadTransport(): Promise<DeviceConnectTransport>;
  now(): number;
};

const ClientTransportEndpointsSchema = Type.Object({
  natsServers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
});

const ClientTransportsSchema = Type.Object({
  native: Type.Optional(ClientTransportEndpointsSchema),
  websocket: Type.Optional(ClientTransportEndpointsSchema),
});

export type TrellisDevicePendingActivationState = {
  status: "pending";
  contractDigest: string;
  publicIdentityKey: string;
  instanceId: string;
  deploymentId: string;
  flowId: string;
  nonce: string;
  activationUrl: string;
};

export type TrellisDeviceActivatedActivationState = {
  status: "activated";
  contractDigest: string;
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
};

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
    servers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
  }),
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
    Type.Literal("manifest_required"),
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
    contractId: string;
    contractDigest: string;
    transports: { websocket: { natsServers: string[] } };
    transport: { jwt: string; inboxPrefix: string };
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

async function signIdentityBytes(
  identitySeed: Uint8Array,
  data: Uint8Array,
): Promise<Uint8Array> {
  const privateKey = await importEd25519PrivateKeyFromSeedBase64url(
    base64urlEncode(identitySeed),
  );
  return new Uint8Array(
    await crypto.subtle.sign("Ed25519", privateKey, toArrayBuffer(data)),
  );
}

function createDeviceNatsAuthTokenAuthenticator(args: {
  publicIdentityKey: string;
  identitySeed: Uint8Array;
  contractDigest: string;
  now: () => number;
  getServerClockOffsetMs: () => number;
}): Authenticator {
  return () => {
    const iat = correctedIatSeconds(
      args.now(),
      args.getServerClockOffsetMs(),
    );
    const sig = signEd25519SeedSha256(
      args.identitySeed,
      new TextEncoder().encode(
        `nats-connect:${
          buildNatsConnectSignaturePayload(iat, args.contractDigest)
        }`,
      ),
    );
    return {
      auth_token: JSON.stringify({
        v: 1,
        sessionKey: args.publicIdentityKey,
        iat,
        sig: base64urlEncode(new Uint8Array(sig)),
        contractDigest: args.contractDigest,
      }),
    };
  };
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
  contractId: string;
  contractDigest: string;
  connectInfo: ResolvedDeviceConnectInfo;
}): void {
  if (
    args.connectInfo.contractId !== args.contractId ||
    args.connectInfo.contractDigest !== args.contractDigest
  ) {
    throw createTransportError({
      code: "trellis.bootstrap.contract_mismatch",
      message:
        "Trellis returned connection details for a different device contract.",
      hint:
        "Retry the connection. If it keeps happening, check the requested device contract and Trellis activation state.",
      context: {
        requestedContractId: args.contractId,
        requestedContractDigest: args.contractDigest,
        returnedContractId: args.connectInfo.contractId,
        returnedContractDigest: args.connectInfo.contractDigest,
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

function parseResponseRecord(text: string): Record<string, unknown> | null {
  if (text.length === 0) {
    return null;
  }
  try {
    const parsed = JSON.parse(text);
    return parsed && typeof parsed === "object" && !Array.isArray(parsed)
      ? parsed as Record<string, unknown>
      : null;
  } catch {
    return null;
  }
}

function resolveDeviceLogger(log?: LoggerLike | false): LoggerLike {
  if (log === false) {
    return noopLogger;
  }

  return log ?? noopLogger;
}

async function readResponseReason(response: Response): Promise<string | null> {
  const text = await response.text();
  if (!text) {
    return null;
  }

  try {
    const parsed = JSON.parse(text) as Record<string, unknown>;
    if (typeof parsed.reason === "string" && parsed.reason.length > 0) {
      return parsed.reason;
    }
  } catch {
    return text;
  }

  return text;
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
  contractDigest: string;
}): void {
  if (args.localState.contractDigest !== args.contractDigest) {
    throw createTransportError({
      code: "trellis.device.activation_state_contract_mismatch",
      message:
        "Local device activation state does not match the requested device contract.",
      hint:
        "Use activation state for the same device contract, or start activation again for this contract digest.",
      context: {
        stateContractDigest: args.localState.contractDigest,
        contractDigest: args.contractDigest,
      },
    });
  }
}

async function createActivationSession<
  TLocalState extends TrellisDeviceLocalActivationState,
>(args: {
  trellisUrl: string;
  contractDigest: string;
  identity: Awaited<ReturnType<typeof deriveDeviceIdentity>>;
  provisioned: TrellisDeviceProvisionedIdentity;
  contract: DeviceContract;
  now: () => number;
  localState: TLocalState;
}): Promise<TrellisDeviceActivationSession<TLocalState>> {
  assertActivationStateMatchesIdentity({
    localState: args.localState,
    publicIdentityKey: args.identity.publicIdentityKey,
  });
  assertActivationStateMatchesContract({
    localState: args.localState,
    contractDigest: args.contractDigest,
  });

  const activatedState = createActivatedLocalState(args.localState);
  return {
    activationUrl: args.localState.activationUrl,
    localState: args.localState,
    waitForOnlineApproval: async (opts?: { signal?: AbortSignal }) => {
      if (args.localState.status === "activated") {
        return activatedState;
      }

      while (!opts?.signal?.aborted) {
        const bootstrap = await fetchDeviceBootstrap({
          trellisUrl: args.trellisUrl,
          deviceIdentity: args.identity,
          provisioned: args.provisioned,
          contract: args.contract,
          now: args.now,
          offsetState: { serverClockOffsetMs: 0 },
          reviewId: args.localState.flowId,
          signal: opts?.signal,
        });
        if (bootstrap.status === "ready") return activatedState;
        if (bootstrap.status === "not_ready") {
          throw createTransportError({
            code: "trellis.auth.device_activation_rejected",
            message: "Device activation was rejected.",
            hint: "Request activation again.",
            context: { reviewId: args.localState.flowId },
          });
        }
      }
      throw opts?.signal?.reason ?? new DOMException("Aborted", "AbortError");
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
  reviewId?: string;
  signal?: AbortSignal;
}): Promise<DeviceBootstrapResponse> {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    const requestStartedAtMs = args.now();
    const issuedAt = Math.trunc(
      requestStartedAtMs + args.offsetState.serverClockOffsetMs,
    );
    const requestId = ulid();
    const identityAuth = await createAuth({
      sessionKeySeed: base64urlEncode(args.deviceIdentity.identitySeed),
    });
    const sessionAuth = await createAuth({
      sessionKeySeed: base64urlEncode(
        crypto.getRandomValues(new Uint8Array(32)),
      ),
    });
    const deviceIdentityKeyId = base64urlEncode(
      await sha256(base64urlDecode(identityAuth.sessionKey)),
    );
    const challengeDigest = base64urlEncode(
      await sha256(utf8(`${args.provisioned.principalId}:${requestId}`)),
    );
    const presentation = await compileProtocolArtifacts(args.contract);
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
      participantArtifactDigest: args.provisioned.participantArtifactDigest,
      participantNeedsDigest: args.provisioned.participantNeedsDigest,
      participantArtifact: presentation.participant,
      referencedApiArtifacts: [
        presentation.api,
        ...presentation.referencedApis,
      ],
      challengeDigest,
      proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
    };
    const requestDigest = await sessionProofRequestDigestV1(unsigned);
    const response = await fetch(
      new URL(
        args.reviewId === undefined
          ? "/bootstrap/device"
          : "/auth/devices/activate/wait",
        args.trellisUrl,
      ),
      {
        method: "POST",
        signal: args.signal,
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify(
          args.reviewId === undefined
            ? {
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
            }
            : {
              reviewId: args.reviewId,
              waitMs: 30_000,
              bootstrap: {
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
              },
            },
        ),
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
      sessionAuth.setServerClockOffsetMs(
        estimateMidpointClockOffsetMs({
          requestStartedAtMs,
          responseReceivedAtMs,
          serverNowSeconds: ready.serverNow / 1_000,
        }),
      );
      return {
        status: "ready",
        sessionAuth,
        connectInfo: {
          sessionId: ready.session.sessionId,
          instanceId: args.provisioned.instanceId,
          deploymentId: args.provisioned.deploymentId,
          contractId: args.provisioned.participantId,
          contractDigest: ready.authorization.participantArtifactDigest,
          transports: { websocket: { natsServers: ready.nats.servers } },
          transport: {
            jwt: ready.nats.jwt,
            inboxPrefix: ready.session.inboxPrefix,
          },
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
  const activation = await fetchDeviceBootstrap({
    trellisUrl: args.trellisUrl,
    deviceIdentity: identity,
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    offsetState: { serverClockOffsetMs: 0 },
  });
  if (activation.status !== "activation_required") {
    throw createTransportError({
      code: "trellis.auth.device_activation_unavailable",
      message: "The device does not require activation.",
      hint: "Connect the device directly.",
      context: { status: activation.status },
    });
  }
  const nonce = ulid();

  return await createActivationSession({
    trellisUrl: args.trellisUrl,
    contractDigest: args.contract.CONTRACT_DIGEST,
    identity,
    provisioned: args.identity,
    contract: args.contract,
    now: deps.now,
    localState: {
      status: "pending",
      contractDigest: args.contract.CONTRACT_DIGEST,
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
    contractDigest: args.contract.CONTRACT_DIGEST,
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
  const contractDigest = args.contract.CONTRACT_DIGEST;
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
    contractId: args.contract.CONTRACT_ID,
    contractDigest,
    connectInfo,
  });

  const transport = await deps.loadTransport();
  const sessionOptions = await bootstrap.sessionAuth.natsConnectOptions({
    sessionId: connectInfo.sessionId,
    participantDigest: connectInfo.contractDigest,
    jwt: connectInfo.transport.jwt,
  });
  let nc: NatsConnection;
  try {
    nc = await transport.connect({
      servers: selectRuntimeTransportServers(connectInfo.transports),
      maxReconnectAttempts: DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
      inboxPrefix: connectInfo.transport.inboxPrefix,
      authenticator: sessionOptions.authenticator,
    });
  } catch (cause) {
    throw createTransportError({
      code: "trellis.runtime.connect_failed",
      message: "Trellis could not open the device runtime connection.",
      hint:
        "Retry the connection. If it keeps failing, check Trellis transport availability.",
      cause,
      context: { contractId: args.contract.CONTRACT_ID },
    });
  }

  const connection = observeNatsTrellisConnection({
    kind: "device",
    nc,
    log: false,
    lifecycleLog: {
      log,
      context: { contractId: args.contract.CONTRACT_ID },
    },
  });

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
    serviceName: args.contract.CONTRACT?.displayName ??
      args.contract.CONTRACT_ID,
    kind: "device",
    instanceId: connectInfo.instanceId,
    contractId: connectInfo.contractId,
    contractDigest: connectInfo.contractDigest,
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
          contractId: connectInfo.contractId,
          contractDigest: connectInfo.contractDigest,
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
