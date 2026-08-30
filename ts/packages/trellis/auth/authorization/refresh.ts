import { Value } from "typebox/value";
import { ulid } from "ulid";

import { base64urlDecode, base64urlEncode, sha256 } from "../utils.ts";
import { sessionProofRequestDigest } from "../session_proof.ts";
import type { TrellisAuth } from "../session_auth.ts";
import type { AuthorizationContextCache } from "./client_context.ts";
import type {
  AuthorizationContextRefreshResponse,
  AuthorizationContextRefreshResult,
  AuthorizationRuntimeBinding,
  VerifiedAuthorizationContext,
} from "./types.ts";
import { AuthorizationContextRefreshResponseSchema as ResponseSchema } from "./types.ts";

/** HTTP refresh failure with terminal-state classification. */
export class AuthorizationContextRefreshError extends Error {
  readonly terminal: boolean;

  constructor(readonly status: number) {
    super(`Authorization context refresh failed with HTTP ${status}`);
    this.name = "AuthorizationContextRefreshError";
    this.terminal = status === 401 || status === 403 || status === 409;
  }
}

/** Refresh a context after proving possession of its bound session key. */
export async function refreshAuthorizationContextWithMetadata(args: {
  trellisUrl: string;
  sessionId: string;
  auth: TrellisAuth;
  cache: AuthorizationContextCache;
  fetch?: typeof globalThis.fetch;
  shouldInstall?: () => boolean;
}): Promise<AuthorizationContextRefreshResult> {
  const fetch = args.fetch ?? globalThis.fetch;
  let current: VerifiedAuthorizationContext | undefined;
  try {
    current = args.cache.current();
  } catch {
    current = undefined;
  }
  const session = args.cache.sessionBinding();
  if (session.sessionId !== args.sessionId) {
    throw new Error("authorization recovery session mismatch");
  }
  const durable = await args.cache.store.load();
  if (!durable) throw new Error("authorization trust floor unavailable");
  const requestStartedAt = args.cache.nowMilliseconds();
  const request = {
    requestId: ulid(),
    issuedAt: Math.trunc(args.auth.currentIat() * 1_000),
    sessionId: args.sessionId,
    sessionNkey: args.auth.sessionNkey,
    currentContextDigest: current?.contextDigest ?? null,
    expectedParticipantDigest: session.participantDigest,
    expectedNeedsDigest: session.needsDigest,
    knownRootKeyId: durable.trust.rootKeyId,
    minimumManifestGeneration: durable.trust.minimumManifestGeneration,
    proof: { format: "trellis.session-proof.v1", signature: "" } as const,
  };
  const requestDigest = await sessionProofRequestDigest(request);
  const proof = await args.auth.signSessionProof({
    purpose: "authorizationContextRefresh",
    requestId: request.requestId,
    issuedAt: request.issuedAt,
    sessionId: request.sessionId,
    sessionKeyId: base64urlEncode(
      await sha256(base64urlDecode(args.auth.sessionKey)),
    ),
    currentContextDigest: request.currentContextDigest,
    expectedParticipantDigest: request.expectedParticipantDigest,
    expectedNeedsDigest: request.expectedNeedsDigest,
    knownRootKeyId: request.knownRootKeyId,
    minimumManifestGeneration: request.minimumManifestGeneration,
    requestDigest,
  });
  const response = await fetch(
    new URL("/auth/context/refresh", args.trellisUrl),
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({ ...request, proof }),
    },
  );
  if (!response.ok) throw new AuthorizationContextRefreshError(response.status);
  const next = Value.Parse(
    ResponseSchema,
    await response.json(),
  ) as AuthorizationContextRefreshResponse;
  const serverClockOffsetMs = next.serverNow - Math.trunc(
    (requestStartedAt + args.cache.nowMilliseconds()) / 2,
  );
  args.cache.setServerClockOffsetMs(serverClockOffsetMs);
  args.auth.setServerClockOffsetMs(serverClockOffsetMs);
  if (args.shouldInstall?.() === false) {
    throw new Error("authorization context refresh stopped");
  }
  const runtime = runtimeBindingFromResponse(next);
  const context = await args.cache.install(
    next.authorizationContext,
    {
      bootstrapJwt: next.nats.jwt,
      bootstrapJwtExpiresAt: next.nats.jwtExpiresAt,
    },
    Math.floor(next.serverNow / 1_000),
    args.shouldInstall,
    runtime,
  );
  return { context, response: next };
}

/** Refresh a context and return only its verified projection. */
export async function refreshAuthorizationContext(args: {
  trellisUrl: string;
  sessionId: string;
  auth: TrellisAuth;
  cache: AuthorizationContextCache;
  fetch?: typeof globalThis.fetch;
  shouldInstall?: () => boolean;
}): Promise<VerifiedAuthorizationContext> {
  const result = await refreshAuthorizationContextWithMetadata(args);
  return result.context;
}

/** Start proactive refresh using the context's distributed refresh time. */
export function startAuthorizationContextRefresh(args: {
  trellisUrl: string;
  sessionId: string;
  auth: TrellisAuth;
  cache: AuthorizationContextCache;
  fetch?: typeof globalThis.fetch;
  onTerminalFailure?: (error: unknown) => void | Promise<void>;
  onTransientFailure?: (error: unknown) => void | Promise<void>;
  onExpired?: (error: unknown) => void | Promise<void>;
  onRefresh?: (context: VerifiedAuthorizationContext) => void | Promise<void>;
}): () => void {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let failures = 0;
  let running = false;
  let wakePending = false;
  const schedule = (delayMs: number) => {
    if (!stopped) timer = setTimeout(run, delayMs);
  };
  const run = async () => {
    if (running) {
      wakePending = true;
      return;
    }
    running = true;
    const clearGuard = args.cache.clearGuard();
    try {
      let before: string | undefined;
      try {
        before = args.cache.current().contextDigest;
      } catch {
        before = undefined;
      }
      const result = await refreshAuthorizationContextWithMetadata({
        ...args,
        shouldInstall: () => !stopped,
      });
      if (stopped) return;
      failures = 0;
      await args.onRefresh?.(result.context);
      schedule(
        refreshDelay(
          args.cache,
          before === result.context.contextDigest ? 5_000 : 1_000,
        ),
      );
    } catch (error) {
      if (stopped) return;
      if (error instanceof AuthorizationContextRefreshError && error.terminal) {
        if (!(await args.cache.clearIfCurrent(clearGuard))) {
          failures = 0;
          schedule(refreshDelay(args.cache, 1_000));
          return;
        }
        await args.onTerminalFailure?.(error);
        return;
      }
      failures += 1;
      let current: VerifiedAuthorizationContext | undefined;
      try {
        current = args.cache.current();
      } catch {
        current = undefined;
      }
      await args.onTransientFailure?.(error);
      const beforeExpiry = current
        ? Math.max(
          1_000,
          (current.context.expiresAt - args.cache.correctedNowSeconds()) *
            1_000,
        )
        : Number.POSITIVE_INFINITY;
      schedule(Math.min(beforeExpiry, 5_000 * 2 ** Math.min(failures - 1, 3)));
    } finally {
      running = false;
      if (wakePending && !stopped) {
        wakePending = false;
        if (timer !== undefined) clearTimeout(timer);
        schedule(0);
      }
    }
  };
  schedule(refreshDelay(args.cache));
  let unregisterRefreshRequest: () => void;
  try {
    unregisterRefreshRequest = args.cache.registerRefreshRequest(() => {
      if (running) {
        wakePending = true;
        return;
      }
      if (timer !== undefined) clearTimeout(timer);
      schedule(0);
    });
  } catch (error) {
    stopped = true;
    if (timer !== undefined) clearTimeout(timer);
    throw error;
  }
  return () => {
    stopped = true;
    unregisterRefreshRequest();
    if (timer !== undefined) clearTimeout(timer);
  };
}

function refreshDelay(
  cache: AuthorizationContextCache,
  minimumMs = 1_000,
): number {
  try {
    return Math.max(
      minimumMs,
      (cache.routingRefreshAt() - cache.correctedNowSeconds()) * 1_000,
    );
  } catch {
    return minimumMs;
  }
}

function runtimeBindingFromResponse(
  response: AuthorizationContextRefreshResponse,
): AuthorizationRuntimeBinding {
  if (
    !response.nats.transports.native &&
    !response.nats.transports.websocket
  ) {
    throw new Error("authorization refresh returned no NATS transport");
  }
  return {
    sessionId: response.session.sessionId,
    participantId: response.session.participantId,
    participantArtifactDigest: response.session.participantArtifactDigest,
    participantNeedsDigest: response.session.participantNeedsDigest,
    inboxPrefix: response.session.inboxPrefix,
    transports: {
      ...(response.nats.transports.native === undefined ? {} : {
        native: {
          natsServers: [...response.nats.transports.native.natsServers],
        },
      }),
      ...(response.nats.transports.websocket === undefined ? {} : {
        websocket: {
          natsServers: [...response.nats.transports.websocket.natsServers],
        },
      }),
    },
  };
}

export type { AuthorizationContextPersistence } from "./store.ts";
