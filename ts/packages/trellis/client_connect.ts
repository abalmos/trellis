import {
  type Authenticator,
  type NatsConnection,
  wsconnect,
} from "@nats-io/nats-core";
import {
  CONTRACT_STATE_METADATA,
  type ContractStateMetadata,
} from "./contract_support/mod.ts";
import { getContractRuntime } from "./contract_support/contract_runtime.ts";
import type { NativeProtocolContract } from "./contract_support/protocol_artifacts.ts";
import { resolveNativeProtocolPresentation } from "./contract_support/protocol_resolution.ts";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import {
  base64urlDecode,
  base64urlEncode,
  BrowserAuthorizationContextStore,
  getOrCreateSessionKey,
  getPublicSessionKey,
  type SessionKeyOptions,
  setSessionId,
} from "./auth/browser.ts";
import { createAuth, type TrellisAuth } from "./auth/session_auth.ts";
import {
  AuthorizationContextBundleSchema,
  AuthorizationContextCache,
  AuthorizationContextRefreshError,
  type AuthorizationContextStore,
  AuthorizationProviderCache,
  MemoryAuthorizationContextStore,
  refreshAuthorizationContextWithMetadata,
  startAuthorizationContextRefresh,
} from "./auth/authorization_context.ts";
import {
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigest,
} from "./auth/session_proof.ts";
import { estimateMidpointClockOffsetMs } from "./auth/time.ts";
import { toArrayBuffer } from "./auth/browser.ts";
import {
  importEd25519PrivateKeyFromSeedBase64url,
  publicKeyBase64urlFromSeed,
} from "./auth/keys.ts";
import type { ClientOpts } from "./client.ts";
import type { RuntimeApi } from "./contract_support/runtime.ts";
import {
  DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
  type RuntimeTransport,
} from "./runtime_transport.ts";
import {
  type RuntimeStateStores,
  Trellis,
  type TrellisOpts,
} from "./session.ts";
import { TransportError } from "./errors/index.ts";
import {
  AsyncResult,
  BaseError,
  Result,
  UnexpectedError,
} from "@qlever-llc/result";
import { type StaticDecode, Type } from "typebox";
import { Value } from "typebox/value";
import { ulid } from "ulid";
import { type SessionKeyHandle, signBytes } from "./auth/browser/session.ts";
import {
  observeNatsTrellisConnection,
  type TrellisConnection,
} from "./connection.ts";
import { recordTrellisDuration } from "./telemetry/mod.ts";

type ClientContract = NativeProtocolContract & {
  readonly [CONTRACT_STATE_METADATA]?: ContractStateMetadata;
};

function createConnectedClient(args: {
  name: string;
  nc: NatsConnection;
  connection: TrellisConnection;
  inboxPrefix: string;
  sessionKey: string;
  sign(data: Uint8Array): Promise<Uint8Array>;
  contextDigest: string | (() => string);
  authorizationProviderCache: AuthorizationProviderCache;
  opts: {
    log: ClientOpts["log"];
    timeout: ClientOpts["timeout"];
    stream: ClientOpts["stream"];
    noResponderRetry: ClientOpts["noResponderRetry"];
    api: RuntimeApi;
    state: TrellisOpts<RuntimeApi>["state"];
    onSessionNotFound?: TrellisOpts<RuntimeApi>["onSessionNotFound"];
  };
}): Trellis<RuntimeApi, "client", RuntimeStateStores> {
  const trellis = new Trellis<RuntimeApi, "client", RuntimeStateStores>(
    args.name,
    args.nc,
    {
      sessionKey: args.sessionKey,
      sign: args.sign,
      contextDigest: args.contextDigest,
      authorizationProviderCache: args.authorizationProviderCache,
    },
    {
      ...args.opts,
      connection: args.connection,
    },
    args.inboxPrefix,
  );

  return trellis;
}

function clientConnectResult<T>(
  promise: Promise<T>,
): AsyncResult<T, TransportError | UnexpectedError | ClientAuthHandledError> {
  return AsyncResult.from(
    promise.then(
      (value): Result<
        T,
        TransportError | UnexpectedError | ClientAuthHandledError
      > => Result.ok(value),
      (
        cause,
      ): Result<T, TransportError | UnexpectedError | ClientAuthHandledError> =>
        Result.err(
          cause instanceof TransportError ||
            cause instanceof ClientAuthHandledError
            ? cause
            : new UnexpectedError({ cause }),
        ),
    ),
  );
}

type BrowserClientAuthOptions = {
  mode?: "browser";
  handle?: SessionKeyHandle;
  provider?: string;
  redirectTo?: string | (() => string);
  landingPath?: string;
  context?: unknown;
  currentUrl?: URL | string | (() => URL | string);
  flowId?: string;
  sessionKey?: SessionKeyOptions;
  authorizationContextStore?: AuthorizationContextStore;
};

type SessionKeyClientAuthOptionsBase = {
  mode: "session_key";
  sessionKeySeed: string;
  sessionId?: string;
  provider?: string;
  redirectTo: string;
  context?: unknown;
  flowId?: string;
};

type SessionKeyClientAuthOptions =
  & SessionKeyClientAuthOptionsBase
  & (
    | {
      authorizationContextStore: AuthorizationContextStore;
      authorizationContextEphemeral?: never;
    }
    | {
      authorizationContextStore?: never;
      authorizationContextEphemeral: true;
    }
  );

export type ClientAuthOptions =
  | BrowserClientAuthOptions
  | SessionKeyClientAuthOptions;

export type ClientAuthRequiredContext = {
  loginUrl: string;
  sessionKey: string;
  mode: "browser" | "session_key";
};

export type ClientAuthContinuation =
  | { status: "bound"; flowId: string }
  | { status: "handled" }
  | void;

export type ClientAuthHandledErrorData = {
  id: string;
  type: "ClientAuthHandledError";
  message: string;
  context?: Record<string, unknown>;
  traceId?: string;
};

/**
 * Error raised when client authentication was delegated to caller-owned routing.
 */
export class ClientAuthHandledError
  extends BaseError<ClientAuthHandledErrorData> {
  override readonly name = "ClientAuthHandledError" as const;

  constructor() {
    super("Client authentication was handled by the caller");
  }

  override toSerializable(): ClientAuthHandledErrorData {
    return this.baseSerializable() as ClientAuthHandledErrorData;
  }
}

type ClientConnectArgsFor<TContract extends ClientContract> =
  & ClientOpts
  & {
    trellisUrl: string;
    contract: TContract;
    participant: {
      id: string;
      artifactDigest: string;
    };
    auth?: ClientAuthOptions;
    onAuthRequired?: (
      ctx: ClientAuthRequiredContext,
    ) => Promise<ClientAuthContinuation> | ClientAuthContinuation;
  };

export type TrellisClientConnectArgs<
  TContract extends ClientContract = ClientContract,
> = ClientConnectArgsFor<TContract>;

type ClientRuntimeIdentity = {
  mode: "browser" | "session_key";
  sessionKey: string;
  sessionNkey: string;
  seed: Uint8Array;
  sessionId?: string;
  setSessionId(sessionId: string): Promise<void>;
  auth: TrellisAuth;
  sign(data: Uint8Array): Promise<Uint8Array>;
};

const ClientTransportEndpointsSchema = Type.Object({
  natsServers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
});

const ClientTransportsSchema = Type.Object({
  native: Type.Optional(ClientTransportEndpointsSchema),
  websocket: Type.Optional(ClientTransportEndpointsSchema),
});
type RuntimeTransports = StaticDecode<typeof ClientTransportsSchema>;

type ClientConnectDeps = {
  loadTransport(): Promise<RuntimeTransport>;
  now(): number;
  authorizationContextStore?: AuthorizationContextStore;
};

const ClientBootstrapReadySchema = Type.Object({
  status: Type.Literal("ready"),
  serverNow: Type.Integer(),
  connectInfo: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    participantId: Type.String({ minLength: 1 }),
    participantDigest: Type.String({ minLength: 1 }),
    transports: ClientTransportsSchema,
    transport: Type.Object({
      inboxPrefix: Type.String({ minLength: 1 }),
      jwt: Type.String({ minLength: 1 }),
      jwtExpiresAt: Type.Integer({ minimum: 1 }),
    }),
    authorizationContext: AuthorizationContextBundleSchema,
  }),
}, { additionalProperties: true });

type ClientBootstrapReady = StaticDecode<typeof ClientBootstrapReadySchema>;
type ClientBootstrapAuthRequired = {
  status: "auth_required";
  serverNow: number;
};
type ClientBootstrapNotReady = {
  status: "not_ready";
  reason: string;
  serverNow: number;
};
type ClientBootstrapResponse =
  | ClientBootstrapReady
  | ClientBootstrapAuthRequired
  | ClientBootstrapNotReady;
type ClockOffsetState = { serverClockOffsetMs: number };
type BrowserGlobalThis = typeof globalThis & {
  document?: unknown;
  window?: unknown;
};

function isBrowserRuntime(): boolean {
  const browserGlobal = globalThis as BrowserGlobalThis;
  return typeof browserGlobal.window !== "undefined" &&
    typeof browserGlobal.document !== "undefined";
}

function selectClientRuntimeTransportServers(
  transports: RuntimeTransports,
): string[] {
  if (isBrowserRuntime()) {
    if (transports.websocket?.natsServers?.length) {
      return transports.websocket.natsServers;
    }
    throw new Error(
      "Browser authorization runtime has no WebSocket NATS endpoints",
    );
  }
  if (transports.native?.natsServers?.length) {
    return transports.native.natsServers;
  }

  throw new Error("Authorization runtime has no native NATS endpoints");
}

const defaultDeps: ClientConnectDeps = {
  loadTransport: async () => {
    if (isBrowserRuntime()) {
      return { connect: wsconnect };
    }

    const mod = await import("./runtime_transport.ts");
    return await mod.loadDefaultRuntimeTransport();
  },
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

const BindWireSchema = Type.Object({
  serverNow: Type.Integer(),
  session: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    principalId: Type.String({ minLength: 1 }),
    principalKind: Type.Union([
      Type.Literal("user"),
      Type.Literal("service"),
      Type.Literal("device"),
    ]),
    participantId: Type.String({ minLength: 1 }),
    participantKind: Type.Union([
      Type.Literal("app"),
      Type.Literal("agent"),
      Type.Literal("device"),
      Type.Literal("service"),
    ]),
    inboxPrefix: Type.String({ minLength: 1 }),
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
  nats: Type.Object({
    jwt: Type.String({ minLength: 1 }),
    jwtExpiresAt: Type.Integer({ minimum: 1 }),
    transports: ClientTransportsSchema,
  }, { additionalProperties: false }),
  authorizationContext: AuthorizationContextBundleSchema,
}, { additionalProperties: false });

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

function normalizeTrellisUrl(trellisUrl: string): string {
  return new URL(trellisUrl).toString().replace(/\/$/, "");
}

function resolveCurrentUrl(auth?: BrowserClientAuthOptions): URL | null {
  const currentUrl = typeof auth?.currentUrl === "function"
    ? auth.currentUrl()
    : auth?.currentUrl;
  if (currentUrl instanceof URL) return currentUrl;
  if (typeof currentUrl === "string") return new URL(currentUrl);
  return null;
}

function resolveRedirectTo(
  auth: BrowserClientAuthOptions,
  currentUrl: URL,
): string {
  const redirectTo = typeof auth.redirectTo === "function"
    ? auth.redirectTo()
    : auth.redirectTo;
  if (redirectTo) {
    return new URL(redirectTo, currentUrl.origin).toString();
  }

  const queryRedirect = currentUrl.searchParams.get("redirectTo");
  if (queryRedirect) {
    return new URL(queryRedirect, currentUrl.origin).toString();
  }

  if (auth.landingPath) {
    return new URL(auth.landingPath, currentUrl.origin).toString();
  }

  return currentUrl.toString();
}

function resolveConfiguredRedirectTo(
  redirectTo: string | (() => string) | undefined,
): string | undefined {
  return typeof redirectTo === "function" ? redirectTo() : redirectTo;
}

async function createSessionKeyRuntimeIdentity(
  sessionKeySeed: string,
  sessionId?: string,
): Promise<ClientRuntimeIdentity> {
  const seed = base64urlDecode(sessionKeySeed);
  const privateKey = await importEd25519PrivateKeyFromSeedBase64url(
    sessionKeySeed,
  );
  const sessionKey = publicKeyBase64urlFromSeed(seed);
  const auth = await createAuth({ sessionKeySeed });
  const sign = async (data: Uint8Array): Promise<Uint8Array> => {
    const signature = await crypto.subtle.sign(
      "Ed25519",
      privateKey,
      toArrayBuffer(data),
    );
    return new Uint8Array(signature);
  };

  const identity: ClientRuntimeIdentity = {
    mode: "session_key",
    sessionKey,
    sessionNkey: auth.sessionNkey,
    seed,
    auth,
    sessionId,
    setSessionId: (value) => {
      identity.sessionId = value;
      return Promise.resolve();
    },
    sign,
  };
  return identity;
}

async function resolveClientIdentity(
  auth: ClientAuthOptions | undefined,
  storageScope: string,
): Promise<ClientRuntimeIdentity> {
  if (auth?.mode === "session_key") {
    return await createSessionKeyRuntimeIdentity(
      auth.sessionKeySeed,
      auth.sessionId,
    );
  }

  const handle = auth?.handle ?? await getOrCreateSessionKey({
    ...auth?.sessionKey,
    storageScope,
  });
  const sessionAuth = await createAuth({
    sessionKeySeed: base64urlEncode(handle.seed),
  });
  return {
    mode: "browser",
    sessionKey: getPublicSessionKey(handle),
    sessionNkey: sessionAuth.sessionNkey,
    seed: handle.seed,
    sessionId: handle.sessionId,
    auth: sessionAuth,
    setSessionId: (sessionId) =>
      Promise.resolve(setSessionId(handle, sessionId)),
    sign: (data) => signBytes(handle, data),
  };
}

async function bindClientFlow(args: {
  trellisUrl: string;
  origin: string;
  flowId: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
}): Promise<ClientBootstrapReady> {
  const startedAt = performance.now();
  const requestId = ulid();
  const issuedAt = Date.now();
  const unsigned = {
    requestId,
    issuedAt,
    proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
  };
  const requestDigest = await sessionProofRequestDigest(unsigned);
  const response = await fetch(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(args.flowId)}/bind`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: args.origin,
      },
      body: JSON.stringify({
        ...unsigned,
        proof: await args.identity.auth.signSessionProof({
          purpose: "userAuthBind",
          requestId,
          issuedAt,
          flowId: args.flowId,
          sessionPublicKey: args.identity.sessionKey,
          requestDigest,
        }),
      }),
    },
  );
  if (!response.ok) {
    const reason = await response.text();
    throw createTransportError({
      code: "bind_failed",
      message: "Trellis could not finish the sign-in step.",
      hint: "Start the sign-in flow again.",
      context: { status: response.status, trellisUrl: args.trellisUrl, reason },
    });
  }

  const payload = await readJsonResponse(response, {
    code: "bind_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Start the sign-in flow again.",
    context: { flowId: args.flowId },
  });
  if (
    payload && typeof payload === "object" &&
    (payload as { status?: unknown }).status === "expired"
  ) {
    throw createTransportError({
      code: "flow_expired",
      message: "The Trellis sign-in step expired.",
      hint: "Start the sign-in flow again.",
      context: { flowId: args.flowId },
    });
  }
  let parsed: StaticDecode<typeof BindWireSchema>;
  try {
    parsed = Value.Parse(BindWireSchema, payload) as StaticDecode<
      typeof BindWireSchema
    >;
  } catch (cause) {
    throw createTransportError({
      code: "bind_invalid_response",
      message: "Trellis returned an invalid sign-in response.",
      hint: "Start the sign-in flow again.",
      cause,
      context: { flowId: args.flowId },
    });
  }
  await args.identity.setSessionId(parsed.session.sessionId);
  recordTrellisDuration(
    "trellis.connect.duration",
    performance.now() - startedAt,
    {
      phase: "bootstrap",
      participantKind: "client",
      outcome: "ok",
    },
  );
  return {
    status: "ready",
    serverNow: parsed.serverNow / 1_000,
    connectInfo: {
      sessionId: parsed.session.sessionId,
      participantId: parsed.session.participantId,
      participantDigest: parsed.session.participantArtifactDigest,
      transports: parsed.nats.transports,
      transport: {
        inboxPrefix: parsed.session.inboxPrefix,
        jwt: parsed.nats.jwt,
        jwtExpiresAt: parsed.nats.jwtExpiresAt,
      },
      authorizationContext: parsed.authorizationContext,
    },
  };
}

async function recoverClientBootstrapWithRetry(args: {
  trellisUrl: string;
  identity: ClientRuntimeIdentity;
  cache: AuthorizationContextCache;
  deps: ClientConnectDeps;
  offsetState: ClockOffsetState;
}): Promise<ClientBootstrapResponse> {
  if (!args.identity.sessionId) {
    return {
      status: "auth_required",
      serverNow: args.deps.now() / 1_000,
    };
  }

  try {
    await args.cache.restore();
    args.cache.sessionBinding();
    args.offsetState.serverClockOffsetMs = args.cache.serverClockOffsetMs();
    args.identity.auth.setServerClockOffsetMs(
      args.offsetState.serverClockOffsetMs + args.deps.now() - Date.now(),
    );
  } catch (error) {
    if (
      error instanceof Error &&
      error.message === "no authorization session is installed"
    ) {
      return {
        status: "auth_required",
        serverNow: args.deps.now() / 1_000,
      };
    }
    throw error;
  }

  for (let attempt = 0; attempt < 2; attempt += 1) {
    const attemptStartedAt = performance.now();
    const requestStartedAtMs = args.deps.now();
    try {
      const result = await refreshAuthorizationContextWithMetadata({
        trellisUrl: args.trellisUrl,
        sessionId: args.identity.sessionId,
        auth: args.identity.auth,
        cache: args.cache,
      });
      const responseReceivedAtMs = args.deps.now();
      const serverClockOffsetMs = estimateMidpointClockOffsetMs({
        requestStartedAtMs,
        responseReceivedAtMs,
        serverNowSeconds: result.response.serverNow / 1_000,
      });
      args.offsetState.serverClockOffsetMs = serverClockOffsetMs;
      const session = result.response.session;
      const nats = result.response.nats;
      if (
        !nats.transports.native && !nats.transports.websocket
      ) {
        throw createTransportError({
          code: "trellis.bootstrap.invalid_response",
          message: "Trellis returned incomplete client recovery metadata.",
          hint: "Retry the connection. If it keeps happening, check Trellis.",
          context: { trellisUrl: args.trellisUrl },
        });
      }
      recordTrellisDuration(
        "trellis.connect.duration",
        performance.now() - attemptStartedAt,
        {
          phase: "bootstrap",
          participantKind: "client",
          outcome: "ok",
        },
      );
      return {
        status: "ready",
        serverNow: result.response.serverNow / 1_000,
        connectInfo: {
          sessionId: session.sessionId,
          participantId: session.participantId,
          participantDigest: session.participantArtifactDigest,
          transports: nats.transports,
          transport: {
            inboxPrefix: session.inboxPrefix,
            jwt: nats.jwt,
            jwtExpiresAt: nats.jwtExpiresAt,
          },
          authorizationContext: result.response.authorizationContext,
        },
      };
    } catch (error) {
      if (
        error instanceof AuthorizationContextRefreshError &&
        error.terminal
      ) {
        return {
          status: "auth_required",
          serverNow: args.deps.now() / 1_000,
        };
      }
      if (attempt === 0) {
        continue;
      }
      throw error;
    }
  }

  throw createTransportError({
    code: "trellis.bootstrap.time_sync_failed",
    message: "Trellis could not confirm the client time window.",
    hint:
      "Retry the connection. If it keeps happening, check the client and Trellis clocks.",
    context: { trellisUrl: args.trellisUrl },
  });
}

async function createRuntimeUserAuthenticator(args: {
  identity: ClientRuntimeIdentity;
  sessionId: string;
  contextDigest: string | (() => string);
  jwt: string | (() => string);
}): Promise<{ authenticators: Authenticator[]; stop: () => void }> {
  const options = await args.identity.auth.natsConnectOptions({
    sessionId: args.sessionId,
    contextDigest: args.contextDigest,
    jwt: args.jwt,
  });
  return {
    authenticators: Array.isArray(options.authenticator)
      ? options.authenticator
      : [options.authenticator],
    stop: () => {},
  };
}

function cleanupBrowserCallbackUrl(currentUrl: URL): void {
  if (!isBrowserRuntime()) return;
  if (
    !currentUrl.searchParams.has("flowId") &&
    !currentUrl.searchParams.has("authError")
  ) {
    return;
  }

  currentUrl.searchParams.delete("flowId");
  currentUrl.searchParams.delete("authError");
  globalThis.history.replaceState(
    {},
    "",
    currentUrl.pathname + currentUrl.search + currentUrl.hash,
  );
}

function isExpiredBindError(error: unknown): boolean {
  return error instanceof TransportError &&
    error.code === "flow_expired";
}

function needsReauth(
  bootstrap: ClientBootstrapResponse,
): bootstrap is
  | Extract<ClientBootstrapResponse, { status: "auth_required" }>
  | Extract<
    ClientBootstrapResponse,
    {
      status: "not_ready";
      reason: "contract_not_active" | "insufficient_permissions";
    }
  > {
  return bootstrap.status === "auth_required" ||
    (
      bootstrap.status === "not_ready" &&
      (bootstrap.reason === "insufficient_permissions" ||
        bootstrap.reason === "contract_not_active")
    );
}

function bootstrapTargetsRequestedContract<
  TContract extends ClientContract,
>(
  bootstrap: ClientBootstrapResponse,
  args: ClientConnectArgsFor<TContract>,
): boolean {
  return bootstrap.status === "ready" &&
    bootstrap.connectInfo.participantId === args.participant.id &&
    bootstrap.connectInfo.participantDigest === args.contract.CONTRACT_DIGEST;
}

async function buildSessionKeyLoginUrl(args: {
  trellisUrl: string;
  redirectTo: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
  contract: ClientContract;
}): Promise<
  { status: "auth_required"; loginUrl: string }
> {
  const startedAt = performance.now();
  const requestId = ulid();
  const issuedAt = Date.now();
  const presentation = await resolveNativeProtocolPresentation(args.contract);
  if (
    args.participant.id !== args.contract.CONTRACT_ID ||
    args.participant.artifactDigest !== args.contract.CONTRACT_DIGEST
  ) {
    throw new Error("Client participant identity does not match its contract");
  }
  const unsigned = {
    requestId,
    issuedAt,
    sessionPublicKey: args.identity.sessionKey,
    sessionNkey: args.identity.sessionNkey,
    participantId: args.participant.id,
    participantArtifactDigest: args.participant.artifactDigest,
    participantArtifact: presentation.participant,
    referencedApiArtifacts: [presentation.api, ...presentation.referencedApis],
    redirectTarget: args.redirectTo,
    proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
  };
  const requestDigest = await sessionProofRequestDigest(unsigned);
  const response = await fetch(`${args.trellisUrl}/auth/requests`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      ...unsigned,
      proof: await args.identity.auth.signSessionProof({
        purpose: "userAuthRequest",
        requestId,
        issuedAt,
        sessionPublicKey: args.identity.sessionKey,
        sessionNkey: args.identity.sessionNkey,
        participantId: args.participant.id,
        participantDigest: args.participant.artifactDigest,
        redirectTarget: args.redirectTo,
        requestDigest,
      }),
    }),
  });
  if (!response.ok) {
    const reason = await response.text();
    throw createTransportError({
      code: "auth_request_failed",
      message: "Trellis could not start sign-in.",
      hint:
        "Retry sign-in. If it keeps failing, check Trellis availability and access.",
      context: { status: response.status, reason, trellisUrl: args.trellisUrl },
    });
  }

  const payload = await readJsonResponse(response, {
    code: "auth_request_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Retry sign-in. If it keeps happening, start the sign-in flow again.",
    context: { trellisUrl: args.trellisUrl },
  });
  const start = Value.Parse(
    Type.Object({
      flowId: Type.String({ minLength: 1 }),
      loginUrl: Type.String({ minLength: 1 }),
    }, { additionalProperties: false }),
    payload,
  );
  if (start) {
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - startedAt,
      {
        phase: "bootstrap",
        participantKind: "client",
        outcome: "ok",
      },
    );
    return {
      status: "auth_required",
      loginUrl: start.loginUrl,
    };
  }
  throw createTransportError({
    code: "auth_request_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Retry sign-in. If it keeps happening, start the sign-in flow again.",
    context: { trellisUrl: args.trellisUrl },
  });
}

export async function connectClientWithDeps<
  TContract extends ClientContract,
>(
  args: ClientConnectArgsFor<TContract>,
  deps: ClientConnectDeps,
): Promise<CallerRuntime<TContract>> {
  const totalStartedAt = performance.now();
  const trellisUrl = normalizeTrellisUrl(args.trellisUrl);
  const trustScope = `${
    new URL(trellisUrl).origin
  }:${args.participant.id}:${args.participant.artifactDigest}`;
  const identity = await resolveClientIdentity(args.auth, trustScope);
  const currentUrl = args.auth?.mode === "session_key"
    ? null
    : resolveCurrentUrl(args.auth);
  const browserAuth = args.auth?.mode === "session_key" ? undefined : args.auth;
  const callbackFlowId = args.auth?.mode === "session_key"
    ? args.auth.flowId
    : browserAuth?.flowId ?? currentUrl?.searchParams.get("flowId") ??
      undefined;
  const callbackAuthError = args.auth?.mode === "session_key"
    ? undefined
    : currentUrl?.searchParams.get("authError") ?? undefined;
  const offsetState: ClockOffsetState = { serverClockOffsetMs: 0 };

  if (
    args.auth?.mode === "session_key" &&
    !args.auth.authorizationContextStore &&
    args.auth.authorizationContextEphemeral !== true
  ) {
    throw new Error(
      "session-key clients require persistent authorization context storage or explicit ephemeral mode",
    );
  }
  const contextStore = args.auth?.authorizationContextStore ??
    deps.authorizationContextStore ??
    (args.auth?.mode === "session_key"
      ? new MemoryAuthorizationContextStore()
      : new BrowserAuthorizationContextStore(trustScope));
  const authorizationContexts = new AuthorizationContextCache(
    trellisUrl,
    `installation:${trustScope}`,
    contextStore,
    (input, init) => globalThis.fetch(input, init),
    deps.now,
  );

  if (callbackAuthError) {
    if (currentUrl) cleanupBrowserCallbackUrl(currentUrl);
    throw createTransportError({
      code: callbackAuthError,
      message: "Trellis sign-in did not complete.",
      hint: "Start sign-in again if you want to approve access.",
      context: { reason: callbackAuthError, trellisUrl },
    });
  }

  let callbackBootstrap: ClientBootstrapReady | undefined;
  if (callbackFlowId) {
    try {
      callbackBootstrap = await bindClientFlow({
        trellisUrl,
        origin: currentUrl?.origin ?? new URL(trellisUrl).origin,
        flowId: callbackFlowId,
        identity,
        participant: args.participant,
      });
      if (currentUrl) cleanupBrowserCallbackUrl(currentUrl);
    } catch (error) {
      if (currentUrl && isExpiredBindError(error)) {
        cleanupBrowserCallbackUrl(currentUrl);
      } else {
        throw error;
      }
    }
  }

  const initialBootstrapStartedAt = performance.now();
  const initialBootstrap = callbackBootstrap ??
    await recoverClientBootstrapWithRetry({
      trellisUrl,
      identity,
      cache: authorizationContexts,
      deps,
      offsetState,
    });
  recordTrellisDuration(
    "trellis.connect.duration",
    performance.now() - initialBootstrapStartedAt,
    {
      phase: "bootstrap",
      participantKind: "client",
      outcome: "ok",
    },
  );

  const authStartedAt = performance.now();
  const bootstrap = needsReauth(initialBootstrap) ||
      !bootstrapTargetsRequestedContract(initialBootstrap, args)
    ? await resolveAuthRequired(args, identity, currentUrl)
    : initialBootstrap;
  recordTrellisDuration(
    "trellis.connect.duration",
    performance.now() - authStartedAt,
    {
      phase: "auth_resolution",
      participantKind: "client",
      outcome: "ok",
    },
  );

  if (bootstrap.status !== "ready") {
    if (bootstrap.status === "not_ready") {
      throw createTransportError({
        code: "trellis.bootstrap.not_ready",
        message: "Trellis is not ready to connect this client.",
        hint:
          "Wait for the requested app access to become available, then try again.",
        context: { reason: bootstrap.reason },
      });
    }
    throw createTransportError({
      code: "trellis.bootstrap.auth_required",
      message: "Trellis still requires sign-in before connecting this client.",
      hint: "Complete sign-in, then try again.",
    });
  }

  const transport = await deps.loadTransport();
  identity.auth.setServerClockOffsetMs(
    offsetState.serverClockOffsetMs + deps.now() - Date.now(),
  );
  authorizationContexts.setServerClockOffsetMs(
    offsetState.serverClockOffsetMs,
  );
  await authorizationContexts.install(
    bootstrap.connectInfo.authorizationContext,
    {
      bootstrapJwt: bootstrap.connectInfo.transport.jwt,
      bootstrapJwtExpiresAt: bootstrap.connectInfo.transport.jwtExpiresAt,
    },
  );
  const runtimeState = {
    participantDigest: bootstrap.connectInfo.participantDigest,
    sessionId: bootstrap.connectInfo.sessionId,
    jwt: () => authorizationContexts.routingJwt(),
    contextDigest: () => authorizationContexts.current().contextDigest,
  };
  const handleSessionNotFound = identity.mode === "browser"
    ? async () => {
      const latestCurrentUrl = resolveCurrentUrl(browserAuth);
      try {
        await resolveAuthRequired(
          args,
          identity,
          latestCurrentUrl,
        );
      } catch (error) {
        if (error instanceof ClientAuthHandledError) {
          return;
        }
        throw error;
      }
    }
    : undefined;
  const runtimeAuth = await createRuntimeUserAuthenticator({
    identity,
    sessionId: runtimeState.sessionId,
    contextDigest: runtimeState.contextDigest,
    jwt: runtimeState.jwt,
  });
  let nc: NatsConnection | undefined;
  let authorizationProviderCache: AuthorizationProviderCache | undefined;
  try {
    const natsStartedAt = performance.now();
    nc = await transport.connect({
      servers: selectClientRuntimeTransportServers(
        bootstrap.connectInfo.transports,
      ),
      maxReconnectAttempts: DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
      ignoreAuthErrorAbort: true,
      timeout: args.timeout ?? 10_000,
      inboxPrefix: bootstrap.connectInfo.transport.inboxPrefix,
      authenticator: runtimeAuth.authenticators,
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
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - natsStartedAt,
      {
        phase: "nats_connect",
        participantKind: "client",
        outcome: "ok",
      },
    );
  } catch (error) {
    authorizationProviderCache?.stop();
    if (nc && !nc.isClosed()) await nc.close();
    runtimeAuth.stop();
    throw createTransportError({
      code: "trellis.runtime.connect_failed",
      message: "Trellis could not open the runtime connection.",
      hint:
        "Retry the connection. If it keeps failing, check Trellis transport availability.",
      cause: error,
      context: { trellisUrl },
    });
  }
  if (!nc || !authorizationProviderCache) {
    throw new Error("Trellis client runtime connection was not established");
  }
  void nc.closed().finally(() => runtimeAuth.stop());

  const clientOpts: ClientOpts = {
    ...(typeof args.name === "string" ? { name: args.name } : {}),
    ...(args.log ? { log: args.log } : {}),
    ...(typeof args.timeout === "number" ? { timeout: args.timeout } : {}),
    ...(typeof args.stream === "string" ? { stream: args.stream } : {}),
    ...(args.noResponderRetry
      ? { noResponderRetry: args.noResponderRetry }
      : {}),
  };
  const connection = observeNatsTrellisConnection({
    kind: "client",
    nc,
    log: false,
    ...(args.log
      ? {
        lifecycleLog: {
          log: args.log,
          context: { client: clientOpts.name ?? "client" },
        },
      }
      : {}),
  });
  connection.subscribe((status) =>
    authorizationProviderCache.observeConnectionPhase(status.phase)
  );
  const stopContextRefresh = startAuthorizationContextRefresh({
    trellisUrl: args.trellisUrl,
    sessionId: runtimeState.sessionId,
    auth: identity.auth,
    cache: authorizationContexts,
    onRefresh: () => {
      nc.setServers(
        selectClientRuntimeTransportServers(
          authorizationContexts.runtimeBinding().transports,
        ),
      );
      return nc.reconnect();
    },
    onTerminalFailure: async () => {
      if (!nc.isClosed()) await nc.drain();
    },
  });
  void nc.closed().then(stopContextRefresh, stopContextRefresh);

  const api = getContractRuntime(args.contract).usedApi as RuntimeApi;
  const state = args.contract[CONTRACT_STATE_METADATA] as TrellisOpts<
    RuntimeApi
  >["state"];

  const client = createConnectedClient({
    name: clientOpts.name ?? "client",
    nc,
    connection,
    inboxPrefix: bootstrap.connectInfo.transport.inboxPrefix,
    sessionKey: identity.sessionKey,
    sign: identity.sign,
    contextDigest: () => authorizationContexts.current().contextDigest,
    authorizationProviderCache,
    opts: {
      log: clientOpts.log,
      timeout: clientOpts.timeout,
      stream: clientOpts.stream,
      noResponderRetry: clientOpts.noResponderRetry,
      api,
      state,
      onSessionNotFound: handleSessionNotFound,
    },
  });
  recordTrellisDuration(
    "trellis.connect.duration",
    performance.now() - totalStartedAt,
    {
      phase: "total",
      participantKind: "client",
      outcome: "ok",
    },
  );
  return createCallerRuntime(client, args.contract);
}

async function resolveAuthRequired<
  TContract extends ClientContract,
>(
  args: ClientConnectArgsFor<TContract>,
  identity: ClientRuntimeIdentity,
  currentUrl: URL | null,
): Promise<ClientBootstrapResponse> {
  const browserAuth: BrowserClientAuthOptions =
    args.auth?.mode === "session_key" ? {} : args.auth ?? {};
  const redirectTo = args.auth?.mode === "session_key"
    ? args.auth.redirectTo
    : currentUrl
    ? resolveRedirectTo(browserAuth, currentUrl)
    : resolveConfiguredRedirectTo(browserAuth.redirectTo);
  if (!redirectTo) {
    throw new Error("Client authentication requires a redirectTo URL");
  }

  const authStart = await buildSessionKeyLoginUrl({
    trellisUrl: normalizeTrellisUrl(args.trellisUrl),
    redirectTo,
    identity,
    participant: args.participant,
    contract: args.contract,
  });

  const loginUrl = authStart.loginUrl;

  const continuationStartedAt = performance.now();
  const continuation = await args.onAuthRequired?.({
    loginUrl,
    sessionKey: identity.sessionKey,
    mode: identity.mode,
  });
  recordTrellisDuration(
    "trellis.connect.duration",
    performance.now() - continuationStartedAt,
    {
      phase: "bootstrap",
      participantKind: "client",
      outcome: "ok",
    },
  );
  if (continuation && continuation.status === "handled") {
    throw new ClientAuthHandledError();
  }

  if (continuation && continuation.status === "bound") {
    const bindStartedAt = performance.now();
    const bootstrap = await bindClientFlow({
      trellisUrl: normalizeTrellisUrl(args.trellisUrl),
      origin: new URL(redirectTo).origin,
      flowId: continuation.flowId,
      identity,
      participant: args.participant,
    });
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - bindStartedAt,
      {
        phase: "bootstrap",
        participantKind: "client",
        outcome: "ok",
      },
    );
    return bootstrap;
  }

  if (isBrowserRuntime()) {
    globalThis.location.href = loginUrl;
    throw new ClientAuthHandledError();
  }

  throw new Error(
    "Client authentication required and no auth continuation was provided",
  );
}

/** Connects user-facing participants to the Trellis caller runtime. */
export class TrellisClient {
  static connect<
    TContract extends ClientContract,
  >(
    args: ClientConnectArgsFor<TContract>,
  ): AsyncResult<
    CallerRuntime<TContract>,
    TransportError | UnexpectedError | ClientAuthHandledError
  >;
  static connect(
    args: TrellisClientConnectArgs,
  ): AsyncResult<
    unknown,
    TransportError | UnexpectedError | ClientAuthHandledError
  > {
    return clientConnectResult(connectClientWithDeps(args, defaultDeps));
  }
}
