import { type Static, Type } from "typebox";
import { Value } from "typebox/value";
import { ulid } from "ulid";

import type { TrellisAuth } from "./session_auth.ts";
import { sessionProofRequestDigestV1 } from "./session_proof.ts";
import {
  base64urlDecode,
  base64urlEncode,
  canonicalizeJsonValue,
  sha256,
} from "./utils.ts";
import { verifyAuthorizationContextTokenWasm } from "./protocol_wasm.ts";

const CLIENT_STATE_FORMAT = "trellis.authorization-client-state.v1";
const CLIENT_TRUST_FORMAT = "trellis.authorization-client-trust.v1";

/** Wire schema for bootstrap authorization trust material. */
export const AuthorizationTrustBundleSchema = Type.Object({
  root: Type.Unknown(),
  issuerManifestGeneration: Type.Integer({ minimum: 1 }),
  issuerManifestDigest: Type.String({ minLength: 1 }),
  issuerManifestLocator: Type.String({ minLength: 1 }),
  issuerCertificateLocator: Type.String({ minLength: 1 }),
  policy: Type.Object({
    allowedClockSkewSeconds: Type.Integer({ minimum: 0 }),
    maximumContextLifetimeSeconds: Type.Integer({ minimum: 1 }),
    // Canonical signed-context JSON size in UTF-8 bytes.
    maximumContextBytes: Type.Integer({ minimum: 1 }),
    maximumPermissions: Type.Integer({ minimum: 1 }),
    maximumCapabilities: Type.Integer({ minimum: 1 }),
    refreshLeadSeconds: Type.Integer({ minimum: 1 }),
    refreshJitterSeconds: Type.Integer({ minimum: 0 }),
  }),
});

/** Wire schema for one published authorization context bundle. */
export const AuthorizationContextBundleSchema = Type.Object({
  context: Type.String({ minLength: 1 }),
  contextDigest: Type.String({ minLength: 1 }),
  refreshAt: Type.Integer({ minimum: 0 }),
  trust: AuthorizationTrustBundleSchema,
});

/** Proof-bound context and route-JWT renewal response. */
export const AuthorizationContextRefreshResponseSchema = Type.Object({
  serverNow: Type.Integer({ minimum: 0 }),
  authorizationContext: AuthorizationContextBundleSchema,
  bootstrapJwt: Type.String({ minLength: 1 }),
  bootstrapJwtExpiresAt: Type.Integer({ minimum: 1 }),
});

/** Bootstrap authorization trust material. */
export type AuthorizationTrustBundle = Static<
  typeof AuthorizationTrustBundleSchema
>;
/** Published signed context and the trust information needed to verify it. */
export type AuthorizationContextBundle = Static<
  typeof AuthorizationContextBundleSchema
>;

/** Route-selection JWT installed atomically with a context. */
export type AuthorizationRoutingMaterial = {
  bootstrapJwt: string;
  bootstrapJwtExpiresAt: number;
};

/** Stable session evidence retained across short-lived context expiry. */
export type AuthorizationSessionBinding = {
  sessionId: string;
  participantDigest: string;
  needsDigest: string;
};

/** Trusted context projection returned by the Rust/WASM verifier. */
export type VerifiedAuthorizationContext = {
  authority: string;
  rootKeyId: string;
  rootDigest: string;
  manifestDigest: string;
  contextDigest: string;
  manifestGeneration: number;
  refreshAt: number;
  context: Record<string, unknown> & {
    sessionId: string;
    participant: { artifactDigest: string; needsDigest: string };
    notBefore: number;
    expiresAt: number;
  };
};

/** Durable rollback floor for one authorization authority. */
export type AuthorizationTrustState = {
  format: typeof CLIENT_TRUST_FORMAT;
  authority: string;
  rootKeyId: string;
  rootDigest: string;
  minimumManifestGeneration: number;
  manifestDigestAtMinimumGeneration: string;
};

/** Atomically persisted trust floor and current context for one client binding. */
export type AuthorizationClientState = {
  format: typeof CLIENT_STATE_FORMAT;
  binding: string;
  trust: AuthorizationTrustState;
  session: AuthorizationSessionBinding;
  context: AuthorizationContextBundle | null;
  contextExpiresAt: number | null;
  routing: AuthorizationRoutingMaterial | null;
};

/** Persistence port for authorization trust and current context state. */
export type AuthorizationContextStore = {
  load(): Promise<AuthorizationClientState | undefined>;
  commit(state: AuthorizationClientState): Promise<AuthorizationClientState>;
  clearContext(
    expectedContextDigest?: string | null,
    expectedBootstrapJwt?: string | null,
  ): Promise<boolean>;
  resetTrust(): Promise<void>;
};

/** Requires durable storage unless ephemeral operation is explicitly selected. */
export type AuthorizationContextPersistence =
  | {
    authorizationContextStore: AuthorizationContextStore;
    authorizationContextEphemeral?: never;
  }
  | {
    authorizationContextStore?: never;
    authorizationContextEphemeral: true;
  };

/** Process-local context store for tests and explicitly ephemeral clients. */
export class MemoryAuthorizationContextStore
  implements AuthorizationContextStore {
  #state?: AuthorizationClientState;

  load(): Promise<AuthorizationClientState | undefined> {
    return Promise.resolve(
      this.#state ? structuredClone(this.#state) : undefined,
    );
  }

  commit(state: AuthorizationClientState): Promise<AuthorizationClientState> {
    validateAuthorizationClientStateTransition(this.#state, state);
    this.#state = structuredClone(state);
    return Promise.resolve(structuredClone(state));
  }

  clearContext(
    expectedContextDigest?: string | null,
    expectedBootstrapJwt?: string | null,
  ): Promise<boolean> {
    if (
      (expectedContextDigest !== undefined &&
        (this.#state?.context?.contextDigest ?? null) !==
          expectedContextDigest) ||
      (expectedBootstrapJwt !== undefined &&
        (this.#state?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
    ) return Promise.resolve(false);
    if (this.#state) {
      this.#state.context = null;
      this.#state.contextExpiresAt = null;
      this.#state.routing = null;
    }
    return Promise.resolve(true);
  }

  resetTrust(): Promise<void> {
    this.#state = undefined;
    return Promise.resolve();
  }
}

/** Verifies, persists, and exposes the current authorization context. */
export class AuthorizationContextCache {
  #bundle?: AuthorizationContextBundle;
  #verified?: VerifiedAuthorizationContext;
  #session?: AuthorizationSessionBinding;
  #routing?: AuthorizationRoutingMaterial;
  #certificateByLocator = new Map<string, unknown>();
  #clockOffsetMs = 0;
  #operation = 0;

  constructor(
    readonly trellisUrl: string,
    readonly binding: string,
    readonly store: AuthorizationContextStore,
    readonly fetch: typeof globalThis.fetch = globalThis.fetch,
    readonly now: () => number = Date.now,
  ) {
    if (!binding.trim()) {
      throw new Error("authorization context storage binding is empty");
    }
  }

  async restore(
    nowUnixSeconds = this.correctedNowSeconds(),
  ): Promise<boolean> {
    const state = await this.store.load();
    if (!state) return false;
    if (state.binding !== this.binding) {
      throw new Error(
        "authorization context storage belongs to another identity",
      );
    }
    if (
      (state.context === null) !== (state.contextExpiresAt === null) ||
      (state.context === null) !== (state.routing === null)
    ) {
      throw new Error("persisted context and routing material are not atomic");
    }
    if (!state.context || !state.routing) {
      this.#session = structuredClone(state.session);
      return false;
    }
    return await this.installRecoverable(
      state.context,
      state.routing,
      nowUnixSeconds,
      state.contextExpiresAt ?? undefined,
    );
  }

  async installRecoverable(
    bundle: AuthorizationContextBundle,
    routing: AuthorizationRoutingMaterial,
    nowUnixSeconds = this.correctedNowSeconds(),
    knownContextExpiresAt?: number,
  ): Promise<boolean> {
    const verificationNow = knownContextExpiresAt !== undefined &&
        knownContextExpiresAt <= nowUnixSeconds
      ? knownContextExpiresAt - 1
      : nowUnixSeconds;
    const verified = await this.install(bundle, routing, verificationNow);
    if (
      knownContextExpiresAt !== undefined &&
      verified.context.expiresAt !== knownContextExpiresAt
    ) {
      throw new Error(
        "persisted context expiry does not match its signed context",
      );
    }
    if (
      verified.context.expiresAt <= nowUnixSeconds ||
      routing.bootstrapJwtExpiresAt <= nowUnixSeconds
    ) {
      await this.clear();
      return false;
    }
    return true;
  }

  async install(
    bundle: AuthorizationContextBundle,
    routing: AuthorizationRoutingMaterial,
    nowUnixSeconds = this.correctedNowSeconds(),
    shouldInstall: () => boolean = () => true,
  ): Promise<VerifiedAuthorizationContext> {
    const operation = ++this.#operation;
    const durable = await this.store.load();
    if (durable && durable.binding !== this.binding) {
      throw new Error(
        "authorization context storage belongs to another identity",
      );
    }
    const [manifest, certificate] = await Promise.all([
      this.#fetchJson(bundle.trust.issuerManifestLocator, "manifest"),
      this.#fetchCertificate(bundle.trust.issuerCertificateLocator),
    ]);
    const verified = await verifyAuthorizationContext({
      bundle,
      manifest,
      certificate,
      nowUnixSeconds,
      minimumManifestGeneration: Math.max(
        bundle.trust.issuerManifestGeneration,
        durable?.trust.minimumManifestGeneration ?? 0,
      ),
    });
    const next: AuthorizationClientState = {
      format: CLIENT_STATE_FORMAT,
      binding: this.binding,
      trust: {
        format: CLIENT_TRUST_FORMAT,
        authority: verified.authority,
        rootKeyId: verified.rootKeyId,
        rootDigest: verified.rootDigest,
        minimumManifestGeneration: verified.manifestGeneration,
        manifestDigestAtMinimumGeneration: verified.manifestDigest,
      },
      session: {
        sessionId: verified.context.sessionId,
        participantDigest: verified.context.participant.artifactDigest,
        needsDigest: verified.context.participant.needsDigest,
      },
      context: structuredClone(bundle),
      contextExpiresAt: verified.context.expiresAt,
      routing: structuredClone(routing),
    };
    validateAuthorizationClientStateTransition(durable, next);
    if (!shouldInstall() || operation !== this.#operation) {
      throw new Error("authorization context installation stopped");
    }
    const persisted = await this.store.commit(next);
    validateAuthorizationClientStateTransition(next, persisted);
    if (canonicalizeJsonValue(persisted) !== canonicalizeJsonValue(next)) {
      throw new Error(
        "authorization context persistence did not commit exact state",
      );
    }
    if (!shouldInstall() || operation !== this.#operation) {
      throw new Error("authorization context installation stopped");
    }
    this.#bundle = structuredClone(bundle);
    this.#verified = verified;
    this.#session = structuredClone(next.session);
    this.#routing = structuredClone(routing);
    return verified;
  }

  current(
    nowUnixSeconds = this.correctedNowSeconds(),
  ): VerifiedAuthorizationContext {
    const verified = this.#verified;
    if (
      !verified || verified.context.notBefore > nowUnixSeconds ||
      verified.context.expiresAt <= nowUnixSeconds
    ) {
      throw new Error("no current authorization context");
    }
    return verified;
  }

  bundle(): AuthorizationContextBundle {
    if (!this.#bundle) throw new Error("no authorization context is installed");
    return structuredClone(this.#bundle);
  }

  shouldRefresh(nowUnixSeconds = this.correctedNowSeconds()): boolean {
    this.current(nowUnixSeconds);
    return nowUnixSeconds >= this.routingRefreshAt();
  }

  async clear(): Promise<void> {
    const operation = ++this.#operation;
    await this.store.clearContext();
    if (operation !== this.#operation) return;
    this.#bundle = undefined;
    this.#verified = undefined;
    this.#routing = undefined;
  }

  /** Capture the exact persisted material owned by an in-flight refresh. */
  clearGuard(): readonly [string | null, string | null] {
    return [
      this.#bundle?.contextDigest ?? null,
      this.#routing?.bootstrapJwt ?? null,
    ];
  }

  /** Clear terminal state only if no newer context or route JWT replaced it. */
  async clearIfCurrent(
    guard: readonly [string | null, string | null],
  ): Promise<boolean> {
    if (
      (this.#bundle?.contextDigest ?? null) !== guard[0] ||
      (this.#routing?.bootstrapJwt ?? null) !== guard[1]
    ) return false;
    const operation = ++this.#operation;
    await this.store.clearContext(guard[0], guard[1]);
    if (operation !== this.#operation) return false;
    this.#bundle = undefined;
    this.#verified = undefined;
    this.#routing = undefined;
    return true;
  }

  async resetTrust(): Promise<void> {
    this.#operation += 1;
    this.#bundle = undefined;
    this.#verified = undefined;
    this.#session = undefined;
    this.#routing = undefined;
    await this.store.resetTrust();
  }

  sessionBinding(): AuthorizationSessionBinding {
    if (!this.#session) {
      throw new Error("no authorization session is installed");
    }
    return structuredClone(this.#session);
  }

  routingJwt(): string {
    if (
      !this.#routing ||
      this.#routing.bootstrapJwtExpiresAt <= this.correctedNowSeconds()
    ) {
      throw new Error("authorization routing JWT expired");
    }
    return this.#routing.bootstrapJwt;
  }

  routingRefreshAt(): number {
    const context = this.current();
    if (!this.#routing || !this.#bundle) return this.correctedNowSeconds();
    return Math.min(
      context.refreshAt,
      this.#routing.bootstrapJwtExpiresAt -
        this.#bundle.trust.policy.refreshLeadSeconds,
    );
  }

  setServerClockOffsetMs(offsetMs: number): void {
    this.#clockOffsetMs = offsetMs;
  }

  correctedNowSeconds(): number {
    return Math.floor((this.now() + this.#clockOffsetMs) / 1_000);
  }

  nowMilliseconds(): number {
    return this.now();
  }

  async #fetchJson(
    locator: string,
    kind: "manifest" | "certificate",
  ): Promise<unknown> {
    const url = resolveAuthorizationLocator(this.trellisUrl, locator, kind);
    for (let attempt = 0; attempt < 3; attempt += 1) {
      const response = await this.fetch(url, { redirect: "error" });
      if (response.ok) return await response.json();
      if (response.status !== 404 || attempt === 2) {
        throw new Error(
          `authorization registry returned HTTP ${response.status}`,
        );
      }
      await new Promise((resolve) => setTimeout(resolve, 50 * (attempt + 1)));
    }
    throw new Error("authorization registry object is unavailable");
  }

  async #fetchCertificate(locator: string): Promise<unknown> {
    const cached = this.#certificateByLocator.get(locator);
    if (cached !== undefined) return structuredClone(cached);
    const certificate = await this.#fetchJson(locator, "certificate");
    this.#certificateByLocator.set(locator, structuredClone(certificate));
    return certificate;
  }
}

function resolveAuthorizationLocator(
  trellisUrl: string,
  locator: string,
  kind: "manifest" | "certificate",
): URL {
  const base = new URL(trellisUrl);
  if (base.protocol !== "http:" && base.protocol !== "https:") {
    throw new Error("Trellis authorization origin must use HTTP(S)");
  }
  if (!locator.startsWith("/") && !/^https?:\/\//i.test(locator)) {
    throw new Error("authorization locator must be absolute-path or HTTP(S)");
  }
  if (locator.startsWith("//")) {
    throw new Error("authorization locator must not be protocol-relative");
  }
  if (locator.includes("\\")) {
    throw new Error("authorization locator contains path traversal");
  }
  const scheme = locator.indexOf("://");
  const authorityEnd = scheme < 0 ? -1 : locator.indexOf("/", scheme + 3);
  const rawPath =
    (authorityEnd < 0 && scheme >= 0
      ? "/"
      : locator.slice(authorityEnd < 0 ? 0 : authorityEnd)).split(/[?#]/, 1)[0];
  for (const segment of rawPath.split("/")) {
    let decoded: string;
    try {
      decoded = decodeURIComponent(segment);
    } catch {
      throw new Error("authorization locator path encoding is invalid");
    }
    if (
      decoded === "." || decoded === ".." || decoded.includes("/") ||
      decoded.includes("\\")
    ) {
      throw new Error("authorization locator contains path traversal");
    }
  }
  const url = new URL(locator, base);
  if (
    url.origin !== base.origin ||
    (url.protocol !== "http:" && url.protocol !== "https:") ||
    url.username ||
    url.password ||
    url.search ||
    url.hash
  ) {
    throw new Error(
      "authorization locator must use the Trellis HTTP(S) origin",
    );
  }
  const expected = kind === "manifest"
    ? /^\/\.well-known\/trellis\/authorization\/trust\/manifest\.[1-9][0-9]*$/
    : /^\/\.well-known\/trellis\/authorization\/trust\/certificate\.[A-Za-z0-9_-]+\.[A-Za-z0-9_-]+$/;
  if (!expected.test(url.pathname)) {
    throw new Error(`authorization ${kind} locator path is invalid`);
  }
  return url;
}

/** Refresh a context after proving possession of its bound session key. */
export async function refreshAuthorizationContext(args: {
  trellisUrl: string;
  sessionId: string;
  auth: TrellisAuth;
  cache: AuthorizationContextCache;
  fetch?: typeof globalThis.fetch;
  shouldInstall?: () => boolean;
}): Promise<VerifiedAuthorizationContext> {
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
    proof: { format: "trellis.session-proof.v1", signature: "" } as unknown,
  };
  const requestDigest = await sessionProofRequestDigestV1(request);
  request.proof = await args.auth.signSessionProof({
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
      body: JSON.stringify(request),
    },
  );
  if (!response.ok) throw new AuthorizationContextRefreshError(response.status);
  const next = Value.Parse(
    AuthorizationContextRefreshResponseSchema,
    await response.json(),
  );
  const serverClockOffsetMs = next.serverNow - Math.trunc(
    (requestStartedAt + args.cache.nowMilliseconds()) / 2,
  );
  args.cache.setServerClockOffsetMs(serverClockOffsetMs);
  args.auth.setServerClockOffsetMs(serverClockOffsetMs);
  if (args.shouldInstall?.() === false) {
    throw new Error("authorization context refresh stopped");
  }
  return await args.cache.install(
    next.authorizationContext,
    {
      bootstrapJwt: next.bootstrapJwt,
      bootstrapJwtExpiresAt: next.bootstrapJwtExpiresAt,
    },
    Math.floor(next.serverNow / 1_000),
    args.shouldInstall,
  );
}

/** HTTP refresh failure with terminal-state classification. */
export class AuthorizationContextRefreshError extends Error {
  constructor(readonly status: number) {
    super(`authorization context refresh returned HTTP ${status}`);
    this.name = "AuthorizationContextRefreshError";
  }

  get terminal(): boolean {
    return this.status === 401 || this.status === 403 || this.status === 409;
  }
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
  let timer: number | undefined;
  let failures = 0;
  const schedule = (delayMs: number) => {
    if (!stopped) timer = setTimeout(run, delayMs) as unknown as number;
  };
  const run = async () => {
    const clearGuard = args.cache.clearGuard();
    try {
      let before: string | undefined;
      try {
        before = args.cache.current().contextDigest;
      } catch {
        before = undefined;
      }
      const context = await refreshAuthorizationContext({
        ...args,
        shouldInstall: () => !stopped,
      });
      if (stopped) return;
      failures = 0;
      await args.onRefresh?.(context);
      schedule(
        refreshDelay(
          args.cache,
          before === context.contextDigest ? 5_000 : 1_000,
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
    }
  };
  schedule(refreshDelay(args.cache));
  return () => {
    stopped = true;
    if (timer !== undefined) clearTimeout(timer);
  };
}

function refreshDelay(
  cache: AuthorizationContextCache,
  minimumDelayMs = 1_000,
): number {
  try {
    return Math.max(
      minimumDelayMs,
      (cache.routingRefreshAt() - cache.correctedNowSeconds()) * 1_000,
    );
  } catch {
    return minimumDelayMs;
  }
}

/** Verify a complete authorization context chain through Rust/WASM. */
async function verifyAuthorizationContext(args: {
  bundle: AuthorizationContextBundle;
  manifest: unknown;
  certificate: unknown;
  nowUnixSeconds: number;
  minimumManifestGeneration: number;
}): Promise<VerifiedAuthorizationContext> {
  const result = await verifyAuthorizationContextTokenWasm({
    root: args.bundle.trust.root,
    manifest: args.manifest,
    certificate: args.certificate,
    contextToken: args.bundle.context,
    policy: {
      nowUnixSeconds: args.nowUnixSeconds,
      ...args.bundle.trust.policy,
      minimumManifestGeneration: args.minimumManifestGeneration,
    },
  });
  const verified = Value.Parse(
    Type.Object({
      authority: Type.String({ minLength: 1 }),
      rootKeyId: Type.String({ minLength: 1 }),
      rootDigest: Type.String({ minLength: 1 }),
      manifestDigest: Type.String({ minLength: 1 }),
      contextDigest: Type.String({ minLength: 1 }),
      manifestGeneration: Type.Integer({ minimum: 1 }),
      refreshAt: Type.Integer({ minimum: 0 }),
      context: Type.Intersect([
        Type.Object({
          sessionId: Type.String({ minLength: 1 }),
          participant: Type.Object({
            artifactDigest: Type.String({ minLength: 1 }),
            needsDigest: Type.String({ minLength: 1 }),
          }),
          notBefore: Type.Integer({ minimum: 0 }),
          expiresAt: Type.Integer({ minimum: 1 }),
        }),
        Type.Record(Type.String(), Type.Unknown()),
      ]),
    }),
    result,
  ) as VerifiedAuthorizationContext;
  if (
    verified.contextDigest !== args.bundle.contextDigest ||
    verified.manifestGeneration !==
      args.bundle.trust.issuerManifestGeneration ||
    verified.manifestDigest !== args.bundle.trust.issuerManifestDigest ||
    verified.refreshAt !== args.bundle.refreshAt
  ) {
    throw new Error("authorization context bundle identity mismatch");
  }
  return verified;
}

/** Reject trust rollback, root replacement, and same-generation equivocation. */
export function validateAuthorizationClientStateTransition(
  current: AuthorizationClientState | undefined,
  next: AuthorizationClientState,
): void {
  if (
    next.format !== CLIENT_STATE_FORMAT ||
    next.trust.format !== CLIENT_TRUST_FORMAT ||
    !next.binding.trim()
  ) {
    throw new Error("authorization client state is invalid");
  }
  if (
    (next.context === null) !== (next.contextExpiresAt === null) ||
    (next.context === null) !== (next.routing === null)
  ) {
    throw new Error("authorization context and routing state are not atomic");
  }
  if (!current) return;
  if (
    current.binding !== next.binding ||
    current.trust.authority !== next.trust.authority ||
    current.trust.rootKeyId !== next.trust.rootKeyId ||
    current.trust.rootDigest !== next.trust.rootDigest
  ) {
    throw new Error("authorization trust root changed");
  }
  if (
    next.trust.minimumManifestGeneration <
      current.trust.minimumManifestGeneration
  ) {
    throw new Error("authorization issuer manifest rolled back");
  }
  if (
    next.trust.minimumManifestGeneration ===
      current.trust.minimumManifestGeneration &&
    next.trust.manifestDigestAtMinimumGeneration !==
      current.trust.manifestDigestAtMinimumGeneration
  ) {
    throw new Error("authorization issuer manifest equivocated");
  }
}
