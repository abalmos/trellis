import {
  type Authenticator,
  type NatsConnection,
  wsconnect,
} from "@nats-io/nats-core";
import {
  CONTRACT_STATE_METADATA,
  type ContractStateMetadata,
} from "./contract_support/mod.ts";
import {
  type ContractWithRuntime,
  getContractRuntime,
} from "./contract_support/contract_runtime.ts";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import {
  base64urlDecode,
  base64urlEncode,
  getOrCreateSessionKey,
  getPublicSessionKey,
  natsConnectSigForIat,
  type SessionKeyOptions,
  setSessionId,
} from "./auth/browser.ts";
import { createAuth, type TrellisAuth } from "./auth/session_auth.ts";
import {
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigestV1,
} from "./auth/session_proof.ts";
import { sha256, toArrayBuffer, utf8 } from "./auth/browser.ts";
import { estimateMidpointClockOffsetMs } from "./auth/time.ts";
import { buildNatsConnectSignaturePayload } from "./auth/session_auth.ts";
import { canonicalizeJsonValue } from "./auth/utils.ts";
import {
  importEd25519PrivateKeyFromSeedBase64url,
  publicKeyBase64urlFromSeed,
  signEd25519SeedSha256,
} from "./auth/keys.ts";
import type { ClientOpts } from "./client.ts";
import type { TrellisContractV1 } from "./contracts.ts";
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
import {
  bindFlowSig,
  oauthInitSig,
  type SessionKeyHandle,
  signBytes,
} from "./auth/browser/session.ts";
import {
  observeNatsTrellisConnection,
  type TrellisConnection,
} from "./connection.ts";
import { recordTrellisDuration } from "./telemetry/mod.ts";

type ClientContract<TContract extends TrellisContractV1 = TrellisContractV1> =
  & ContractWithRuntime
  & {
    CONTRACT: TContract;
    CONTRACT_DIGEST?: string;
    readonly [CONTRACT_STATE_METADATA]?: ContractStateMetadata;
  };

function createConnectedClient(args: {
  name: string;
  nc: NatsConnection;
  connection: TrellisConnection;
  inboxPrefix: string;
  sessionKey: string;
  sign(data: Uint8Array): Promise<Uint8Array>;
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
};

type SessionKeyClientAuthOptions = {
  mode: "session_key";
  sessionKeySeed: string;
  sessionId?: string;
  provider?: string;
  redirectTo: string;
  context?: unknown;
  flowId?: string;
};

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
      needsDigest: string;
    };
    auth?: ClientAuthOptions;
    onAuthRequired?: (
      ctx: ClientAuthRequiredContext,
    ) => Promise<ClientAuthContinuation> | ClientAuthContinuation;
  };

export type TrellisClientConnectArgs<
  TContract extends ClientContract<TrellisContractV1> = ClientContract<
    TrellisContractV1
  >,
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
  oauthInitSig(
    redirectTo: string,
    context?: unknown,
    provider?: string,
    contract?: Record<string, unknown> | string,
  ): Promise<string>;
  natsConnectSigForIat(iat: number, contractDigest: string): Promise<string>;
  bootstrapSig(iat: number): Promise<string>;
  bindFlowSig(flowId: string): Promise<string>;
  buildRuntimeAuthTokenSync?(iat: number, contractDigest: string): string;
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
};

const ClientBootstrapReadySchema = Type.Object({
  status: Type.Literal("ready"),
  serverNow: Type.Integer(),
  connectInfo: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    contractId: Type.String({ minLength: 1 }),
    contractDigest: Type.String({ minLength: 1 }),
    transports: ClientTransportsSchema,
    transport: Type.Object({
      inboxPrefix: Type.String({ minLength: 1 }),
      jwt: Type.String({ minLength: 1 }),
    }),
  }),
}, { additionalProperties: true });

const ClientBootstrapAuthRequiredSchema = Type.Object({
  status: Type.Literal("auth_required"),
  serverNow: Type.Integer(),
}, { additionalProperties: true });

const ClientBootstrapNotReadySchema = Type.Object({
  status: Type.Literal("not_ready"),
  reason: Type.String({ minLength: 1 }),
  serverNow: Type.Integer(),
}, { additionalProperties: true });

const ClientBootstrapIatOutOfRangeSchema = Type.Object({
  reason: Type.Literal("iat_out_of_range"),
  serverNow: Type.Integer(),
}, { additionalProperties: true });

type ClientBootstrapReady = StaticDecode<typeof ClientBootstrapReadySchema>;
type ClientBootstrapAuthRequired = StaticDecode<
  typeof ClientBootstrapAuthRequiredSchema
>;
type ClientBootstrapNotReady = StaticDecode<
  typeof ClientBootstrapNotReadySchema
>;
type ClientBootstrapIatOutOfRange = StaticDecode<
  typeof ClientBootstrapIatOutOfRangeSchema
>;
type ClientBootstrapResponse =
  | ClientBootstrapReady
  | ClientBootstrapAuthRequired
  | ClientBootstrapNotReady;
type ClientBootstrapAttemptResponse =
  | ClientBootstrapResponse
  | ClientBootstrapIatOutOfRange;
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
    if (transports.native?.natsServers?.length) {
      return transports.native.natsServers;
    }
  } else {
    if (transports.native?.natsServers?.length) {
      return transports.native.natsServers;
    }
    if (transports.websocket?.natsServers?.length) {
      return transports.websocket.natsServers;
    }
  }

  throw new Error("No supported NATS transport endpoints available");
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

const ClientBootstrapWireSchema = Type.Object({
  serverNow: Type.Integer(),
  sessionId: Type.String({ minLength: 1 }),
  inboxPrefix: Type.String({ minLength: 1 }),
  participantId: Type.String({ minLength: 1 }),
  participantArtifactDigest: Type.String({ minLength: 1 }),
  participantNeedsDigest: Type.String({ minLength: 1 }),
  nats: Type.Object({
    jwt: Type.String({ minLength: 1 }),
    servers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
  }),
}, { additionalProperties: true });
const BindWireSchema = Type.Object({
  serverNow: Type.Integer(),
  session: Type.Object({
    sessionId: Type.String({ minLength: 1 }),
    inboxPrefix: Type.String({ minLength: 1 }),
    participantArtifactDigest: Type.String({ minLength: 1 }),
  }, { additionalProperties: true }),
  nats: Type.Object({
    jwt: Type.String({ minLength: 1 }),
    servers: Type.Array(Type.String({ minLength: 1 }), { minItems: 1 }),
  }),
}, { additionalProperties: true });

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

async function signDomainValue(
  sign: (data: Uint8Array) => Promise<Uint8Array>,
  prefix: string,
  value: string,
): Promise<string> {
  const digest = await sha256(utf8(`${prefix}:${value}`));
  const signature = await sign(digest);
  const binary = String.fromCharCode(...signature);
  return btoa(binary).replace(/\+/g, "-").replace(/\//g, "_").replace(
    /=+$/g,
    "",
  );
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
    setSessionId: async (value) => {
      identity.sessionId = value;
    },
    sign,
    oauthInitSig: (redirectTo, context, provider, contract) =>
      signDomainValue(
        sign,
        "oauth-init",
        contract === undefined
          ? `${redirectTo}:${canonicalizeJsonValue(context ?? null)}`
          : `${redirectTo}:${provider ?? ""}:${
            canonicalizeJsonValue(contract)
          }:${canonicalizeJsonValue(context ?? null)}`,
      ),
    natsConnectSigForIat: (iat, contractDigest) =>
      signDomainValue(
        sign,
        "nats-connect",
        buildNatsConnectSignaturePayload(iat, contractDigest),
      ),
    bootstrapSig: (iat) =>
      signDomainValue(sign, "bootstrap-client", String(iat)),
    bindFlowSig: (flowId) => signDomainValue(sign, "bind-flow", flowId),
    buildRuntimeAuthTokenSync: (iat, contractDigest) => {
      const sig = signEd25519SeedSha256(
        seed,
        utf8(
          `nats-connect:${
            buildNatsConnectSignaturePayload(iat, contractDigest)
          }`,
        ),
      );
      return JSON.stringify({
        v: 1,
        sessionKey,
        iat,
        contractDigest,
        sig: base64urlEncode(new Uint8Array(sig)),
      });
    },
  };
  return identity;
}

async function resolveClientIdentity(
  auth: ClientAuthOptions | undefined,
): Promise<ClientRuntimeIdentity> {
  if (auth?.mode === "session_key") {
    return await createSessionKeyRuntimeIdentity(
      auth.sessionKeySeed,
      auth.sessionId,
    );
  }

  const handle = auth?.handle ?? await getOrCreateSessionKey(auth?.sessionKey);
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
    setSessionId: async (sessionId) => setSessionId(handle, sessionId),
    sign: (data) => signBytes(handle, data),
    oauthInitSig: (redirectTo, context, provider, contract) =>
      oauthInitSig(handle, redirectTo, context, provider, contract),
    natsConnectSigForIat: (iat, contractDigest) =>
      natsConnectSigForIat(handle, iat, contractDigest),
    bootstrapSig: (iat) =>
      signDomainValue(
        (data) => signBytes(handle, data),
        "bootstrap-client",
        String(iat),
      ),
    bindFlowSig: (flowId) => bindFlowSig(handle, flowId),
  };
}

async function bindClientFlow(args: {
  trellisUrl: string;
  flowId: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
}): Promise<ClientBootstrapReady> {
  const startedAt = performance.now();
  const response = await fetch(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(args.flowId)}/bind`,
    {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
        Origin: new URL(args.trellisUrl).origin,
      },
      body: JSON.stringify({ idempotencyKey: ulid() }),
    },
  );
  if (!response.ok) {
    const reason = await response.text();
    throw createTransportError({
      code: "trellis.auth.bind_failed",
      message: "Trellis could not finish the sign-in step.",
      hint: "Start the sign-in flow again.",
      context: { status: response.status, trellisUrl: args.trellisUrl, reason },
    });
  }

  const payload = await readJsonResponse(response, {
    code: "trellis.auth.bind_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Start the sign-in flow again.",
    context: { flowId: args.flowId },
  });
  if (
    payload && typeof payload === "object" &&
    (payload as { status?: unknown }).status === "expired"
  ) {
    throw createTransportError({
      code: "trellis.auth.bind_expired",
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
      code: "trellis.auth.bind_invalid_response",
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
      contractId: args.participant.id,
      contractDigest: parsed.session.participantArtifactDigest,
      transports: { websocket: { natsServers: parsed.nats.servers } },
      transport: {
        inboxPrefix: parsed.session.inboxPrefix,
        jwt: parsed.nats.jwt,
      },
    },
  };
}

async function fetchClientBootstrap(args: {
  trellisUrl: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
  issuedAt: number;
}): Promise<ClientBootstrapAttemptResponse> {
  const startedAt = performance.now();
  if (!args.identity.sessionId) {
    return { status: "auth_required", serverNow: args.issuedAt / 1_000 };
  }
  const requestId = ulid();
  const sessionKeyId = base64urlEncode(
    await sha256(base64urlDecode(args.identity.sessionKey)),
  );
  const unsigned = {
    requestId,
    issuedAt: args.issuedAt,
    sessionId: args.identity.sessionId,
    sessionNkey: args.identity.sessionNkey,
    expectedParticipantDigest: args.participant.artifactDigest,
    expectedNeedsDigest: args.participant.needsDigest,
    proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
  };
  const requestDigest = await sessionProofRequestDigestV1(unsigned);
  const response = await fetch(`${args.trellisUrl}/bootstrap/client`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      ...unsigned,
      proof: await args.identity.auth.signSessionProof({
        purpose: "clientBootstrap",
        requestId,
        issuedAt: args.issuedAt,
        sessionId: args.identity.sessionId,
        sessionKeyId,
        sessionPublicKey: args.identity.sessionKey,
        sessionNkey: args.identity.sessionNkey,
        expectedParticipantDigest: args.participant.artifactDigest,
        expectedNeedsDigest: args.participant.needsDigest,
        requestDigest,
      }),
    }),
  });

  if (response.status === 401 || response.status === 403) {
    return { status: "auth_required", serverNow: args.issuedAt / 1_000 };
  }

  const payload = await readJsonResponse(response, {
    code: "trellis.bootstrap.invalid_response",
    message: "Trellis returned an invalid bootstrap response.",
    hint:
      "Retry the connection. If it keeps happening, check the Trellis deployment.",
    context: { trellisUrl: args.trellisUrl },
  });
  if (!response.ok) {
    if (Value.Check(ClientBootstrapIatOutOfRangeSchema, payload)) {
      return { ...payload, serverNow: payload.serverNow / 1_000 };
    }
    const reason = payload && typeof payload === "object" &&
        typeof (payload as { reason?: unknown }).reason === "string"
      ? (payload as { reason: string }).reason
      : `http_${response.status}`;
    throw createTransportError({
      code: "trellis.bootstrap.failed",
      message: "Trellis could not prepare the client session.",
      hint:
        "Retry the connection. If it keeps failing, check Trellis availability and access.",
      context: { trellisUrl: args.trellisUrl, status: response.status, reason },
    });
  }

  if (Value.Check(ClientBootstrapWireSchema, payload)) {
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - startedAt,
      {
        phase: "bootstrap",
        participantKind: "client",
        outcome: "ok",
      },
    );
    const wire = payload as StaticDecode<typeof ClientBootstrapWireSchema>;
    return {
      status: "ready",
      serverNow: wire.serverNow / 1_000,
      connectInfo: {
        sessionId: wire.sessionId,
        contractId: args.participant.id,
        contractDigest: wire.participantArtifactDigest,
        transports: { websocket: { natsServers: wire.nats.servers } },
        transport: { inboxPrefix: wire.inboxPrefix, jwt: wire.nats.jwt },
      },
    };
  }

  throw createTransportError({
    code: "trellis.bootstrap.invalid_response",
    message: "Trellis returned an invalid bootstrap response.",
    hint:
      "Retry the connection. If it keeps happening, check the Trellis deployment.",
    context: { trellisUrl: args.trellisUrl },
  });
}

function updateClockOffsetFromServer(args: {
  offsetState: ClockOffsetState;
  requestStartedAtMs: number;
  responseReceivedAtMs: number;
  serverNowSeconds: number;
}): void {
  args.offsetState.serverClockOffsetMs = estimateMidpointClockOffsetMs({
    requestStartedAtMs: args.requestStartedAtMs,
    responseReceivedAtMs: args.responseReceivedAtMs,
    serverNowSeconds: args.serverNowSeconds,
  });
}

async function fetchClientBootstrapWithRetry(args: {
  trellisUrl: string;
  sessionKey: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
  deps: ClientConnectDeps;
  offsetState: ClockOffsetState;
}): Promise<ClientBootstrapResponse> {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    const attemptStartedAt = performance.now();
    const requestStartedAtMs = args.deps.now();
    const issuedAt = Math.trunc(
      requestStartedAtMs + args.offsetState.serverClockOffsetMs,
    );
    const response = await fetchClientBootstrap({
      trellisUrl: args.trellisUrl,
      identity: args.identity,
      participant: args.participant,
      issuedAt,
    });
    const responseReceivedAtMs = args.deps.now();
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - attemptStartedAt,
      {
        phase: "bootstrap",
        participantKind: "client",
        outcome: "ok",
      },
    );

    updateClockOffsetFromServer({
      offsetState: args.offsetState,
      requestStartedAtMs,
      responseReceivedAtMs,
      serverNowSeconds: response.serverNow,
    });

    if ("status" in response) {
      return response;
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
  participantDigest: string;
  jwt: string;
}): Promise<{ authenticators: Authenticator[]; stop: () => void }> {
  const options = await args.identity.auth.natsConnectOptions({
    sessionId: args.sessionId,
    participantDigest: args.participantDigest,
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
  window.history.replaceState(
    {},
    "",
    currentUrl.pathname + currentUrl.search + currentUrl.hash,
  );
}

function isExpiredBindError(error: unknown): boolean {
  return error instanceof TransportError &&
    error.code === "trellis.auth.bind_expired";
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
  TContract extends ClientContract<TrellisContractV1>,
>(
  bootstrap: ClientBootstrapResponse,
  args: ClientConnectArgsFor<TContract>,
): boolean {
  return bootstrap.status === "ready" &&
    bootstrap.connectInfo.contractId === args.participant.id &&
    bootstrap.connectInfo.contractDigest === args.participant.artifactDigest;
}

async function buildSessionKeyLoginUrl(args: {
  trellisUrl: string;
  redirectTo: string;
  identity: ClientRuntimeIdentity;
  participant: ClientConnectArgsFor<ClientContract>["participant"];
}): Promise<
  { status: "bound" } | { status: "flow_started"; loginUrl: string }
> {
  const startedAt = performance.now();
  const requestId = ulid();
  const issuedAt = Date.now();
  const unsigned = {
    requestId,
    issuedAt,
    sessionPublicKey: args.identity.sessionKey,
    sessionNkey: args.identity.sessionNkey,
    participantId: args.participant.id,
    participantArtifactDigest: args.participant.artifactDigest,
    participantNeedsDigest: args.participant.needsDigest,
    redirectTarget: args.redirectTo,
    proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
  };
  const requestDigest = await sessionProofRequestDigestV1(unsigned);
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
      code: "trellis.auth.login_failed",
      message: "Trellis could not start sign-in.",
      hint:
        "Retry sign-in. If it keeps failing, check Trellis availability and access.",
      context: { status: response.status, reason, trellisUrl: args.trellisUrl },
    });
  }

  const payload = await readJsonResponse(response, {
    code: "trellis.auth.login_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Retry sign-in. If it keeps happening, start the sign-in flow again.",
    context: { trellisUrl: args.trellisUrl },
  });
  if (
    payload && typeof payload === "object" &&
    (payload as { state?: unknown }).state === "flow" &&
    typeof (payload as { portalUrl?: unknown }).portalUrl === "string"
  ) {
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
      status: "flow_started",
      loginUrl: (payload as { portalUrl: string }).portalUrl,
    };
  }
  throw createTransportError({
    code: "trellis.auth.login_invalid_response",
    message: "Trellis returned an invalid sign-in response.",
    hint: "Retry sign-in. If it keeps happening, start the sign-in flow again.",
    context: { trellisUrl: args.trellisUrl },
  });
}

export async function connectClientWithDeps<
  TContract extends ClientContract<TrellisContractV1>,
>(
  args: ClientConnectArgsFor<TContract>,
  deps: ClientConnectDeps,
): Promise<CallerRuntime<TContract>> {
  const totalStartedAt = performance.now();
  const trellisUrl = normalizeTrellisUrl(args.trellisUrl);
  const identity = await resolveClientIdentity(args.auth);
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

  if (callbackAuthError) {
    if (currentUrl) cleanupBrowserCallbackUrl(currentUrl);
    throw createTransportError({
      code: `trellis.auth.${callbackAuthError}`,
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
    await fetchClientBootstrapWithRetry({
      trellisUrl,
      sessionKey: identity.sessionKey,
      identity,
      participant: args.participant,
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
    ? await resolveAuthRequired(args, identity, currentUrl, deps, offsetState)
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
  const runtimeState = {
    participantDigest: bootstrap.connectInfo.contractDigest,
    sessionId: bootstrap.connectInfo.sessionId,
    jwt: bootstrap.connectInfo.transport.jwt,
  };
  const handleSessionNotFound = identity.mode === "browser"
    ? async () => {
      const latestCurrentUrl = resolveCurrentUrl(browserAuth);
      try {
        await resolveAuthRequired(
          args,
          identity,
          latestCurrentUrl,
          deps,
          offsetState,
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
    participantDigest: runtimeState.participantDigest,
    jwt: runtimeState.jwt,
  });
  let nc: NatsConnection;
  try {
    const natsStartedAt = performance.now();
    nc = await transport.connect({
      servers: selectClientRuntimeTransportServers(
        bootstrap.connectInfo.transports,
      ),
      maxReconnectAttempts: DEFAULT_RUNTIME_MAX_RECONNECT_ATTEMPTS,
      inboxPrefix: bootstrap.connectInfo.transport.inboxPrefix,
      authenticator: runtimeAuth.authenticators,
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
  TContract extends ClientContract<TrellisContractV1>,
>(
  args: ClientConnectArgsFor<TContract>,
  identity: ClientRuntimeIdentity,
  currentUrl: URL | null,
  deps: ClientConnectDeps,
  offsetState: ClockOffsetState,
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
  });

  if (authStart.status === "bound") {
    const bootstrapStartedAt = performance.now();
    const bootstrap = await fetchClientBootstrapWithRetry({
      trellisUrl: normalizeTrellisUrl(args.trellisUrl),
      sessionKey: identity.sessionKey,
      identity,
      participant: args.participant,
      deps,
      offsetState,
    });
    recordTrellisDuration(
      "trellis.connect.duration",
      performance.now() - bootstrapStartedAt,
      {
        phase: "bootstrap",
        participantKind: "client",
        outcome: "ok",
      },
    );
    return bootstrap;
  }

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
    window.location.href = loginUrl;
    throw new ClientAuthHandledError();
  }

  throw new Error(
    "Client authentication required and no auth continuation was provided",
  );
}

/** Connects user-facing participants to the Trellis caller runtime. */
export class TrellisClient {
  static connect<
    TContract extends ClientContract<TrellisContractV1>,
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
