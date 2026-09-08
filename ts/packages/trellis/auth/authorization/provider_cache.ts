import type { NatsConnection } from "@nats-io/nats-core";

import type {
  AuthorizationContextHandle,
  AuthorizationIssuerKey,
  AuthorizationVerificationErrorCode,
  VerifiedAuthorizationContextTokenProjection,
  VerifiedAuthorizationEventPublisher,
} from "../protocol_wasm.ts";
import { canonicalizeJsonValue } from "../utils.ts";
import {
  type AuthorizationContextCache,
  authorizationContextVerificationPolicy,
} from "./client_context.ts";
import {
  type AuthorizationRegistryIoCounters,
  AuthorizationRegistryReader,
  registryWatchEntry,
} from "./nats_registry.ts";
import type {
  AuthorizationProviderEvent,
  AuthorizationProviderRequest,
  AuthorizationRegistryBinding,
} from "./types.ts";

const MAX_CACHED_CONTEXTS = 256;

type VerificationFailure = {
  ok: false;
  error: { code: AuthorizationVerificationErrorCode; path: string };
};

type CachedRequestVerificationResult =
  | {
    ok: true;
    contextDigest: string;
    context: VerifiedAuthorizationContextTokenProjection["context"];
  }
  | VerificationFailure;

type CachedEventVerificationResult =
  | {
    ok: true;
    contextDigest: string;
    context: VerifiedAuthorizationContextTokenProjection["context"];
    publisher: VerifiedAuthorizationEventPublisher;
  }
  | VerificationFailure;

/** Observable provider registry health. */
export type AuthorizationProviderCacheHealth = {
  revocationRevision: number;
  lastUpdateAt: number;
  healthy: boolean;
};

/** Provider I/O counters used by local hot-path tests and diagnostics. */
export type AuthorizationProviderIoCounters =
  & AuthorizationRegistryIoCounters
  & {
    contextResolves: number;
    contextVerifications: number;
  };

/** Internal marker for retryable provider registry or readiness failure. */
export class AuthorizationProviderUnavailableError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "AuthorizationProviderUnavailableError";
  }
}

/** Provider attach options. */
export type AuthorizationProviderCacheOptions = { now?: () => number };

type ProviderContextEntry = {
  contextDigest: string;
  context: Record<string, unknown>;
  issuer: AuthorizationIssuerKey;
  revokedAt?: number;
  leases: number;
  watch?: AsyncIterator<import("@nats-io/kv").KvWatchEntry>;
  live?: Promise<{
    handle: AuthorizationContextHandle;
    verified: VerifiedAuthorizationContextTokenProjection;
  }>;
  historical?: Promise<{
    handle: AuthorizationContextHandle;
    verified: VerifiedAuthorizationContextTokenProjection;
  }>;
};

const integrationTestContexts = new WeakMap<
  AuthorizationProviderCache,
  Map<string, ProviderContextEntry>
>();

/** @internal Returns resolved contexts for live integration assertions. */
export function integrationTestResolvedContexts(
  cache: AuthorizationProviderCache,
): Array<{ contextDigest: string; context: Record<string, unknown> }> {
  return [...(integrationTestContexts.get(cache)?.values() ?? [])].map(
    ({ contextDigest, context }) => ({
      contextDigest,
      context: structuredClone(context),
    }),
  );
}

/** Connected provider-side authorization verifier. */
export class AuthorizationProviderCache {
  readonly #registry: AuthorizationRegistryReader;
  readonly #cache: AuthorizationContextCache;
  readonly #now: () => number;
  readonly #contexts = new Map<string, ProviderContextEntry>();
  readonly #inFlight = new Map<string, Promise<ProviderContextEntry>>();
  readonly #issuerKeys = new Map<string, AuthorizationIssuerKey>();
  #contextResolves = 0;
  #contextVerifications = 0;
  #revocationRevision = 0;
  #lastUpdateAt = 0;
  #stopped = false;
  #connected = true;
  #started = false;

  private constructor(
    registry: AuthorizationRegistryReader,
    cache: AuthorizationContextCache,
    options: AuthorizationProviderCacheOptions,
  ) {
    this.#registry = registry;
    this.#cache = cache;
    this.#now = options.now ?? cache.correctedNowSeconds.bind(cache);
    const bundle = cache.bundle();
    this.#issuerKeys.set(bundle.issuer.keyId, structuredClone(bundle.issuer));
    integrationTestContexts.set(this, this.#contexts);
  }

  /** Attach to the bootstrap-selected NATS authorization registry. */
  static async attach(
    nats: NatsConnection,
    binding: AuthorizationRegistryBinding,
    cache: AuthorizationContextCache,
    options: AuthorizationProviderCacheOptions = {},
  ): Promise<AuthorizationProviderCache> {
    if (
      canonicalizeJsonValue(binding) !==
        canonicalizeJsonValue(cache.bundle().authorizationRegistry)
    ) {
      throw new Error("authorization registry binding does not match");
    }
    return new AuthorizationProviderCache(
      await AuthorizationRegistryReader.open(nats, binding),
      cache,
      options,
    );
  }

  /** Enable provider verification. */
  start(): void {
    this.#stopped = false;
    this.#started = true;
  }

  /** Stop verification without closing the caller-owned NATS connection. */
  stop(): void {
    this.#stopped = true;
    for (const entry of this.#contexts.values()) {
      void entry.watch?.return?.();
    }
    this.#contexts.clear();
  }

  /** Wait until the connected registry is available. */
  waitReady(
    options: { signal?: AbortSignal; timeoutMs?: number } = {},
  ): Promise<void> {
    if (!this.#started || this.#stopped || !this.#connected) {
      return Promise.reject(
        new AuthorizationProviderUnavailableError(
          "authorization provider is unavailable",
        ),
      );
    }
    if (options.signal?.aborted) return Promise.reject(options.signal.reason);
    return Promise.resolve();
  }

  /** Return current provider health. */
  health(): AuthorizationProviderCacheHealth {
    return {
      revocationRevision: this.#revocationRevision,
      lastUpdateAt: this.#lastUpdateAt,
      healthy: this.#started && !this.#stopped && this.#connected,
    };
  }

  /** Return provider and registry I/O counters. */
  ioCounters(): AuthorizationProviderIoCounters {
    return {
      ...this.#registry.ioCounters(),
      contextResolves: this.#contextResolves,
      contextVerifications: this.#contextVerifications,
    };
  }

  /** Apply framework-level NATS lifecycle state to provider readiness. */
  observeConnectionPhase(
    phase: "connected" | "disconnected" | "reconnecting" | "error" | "closed",
  ): void {
    if (phase === "error") return;
    const wasConnected = this.#connected;
    this.#connected = phase === "connected";
    if (wasConnected && !this.#connected && phase !== "closed") {
      this.#cache.requestRefresh();
    }
    if (wasConnected && !this.#connected) {
      for (const entry of this.#contexts.values()) {
        void entry.watch?.return?.();
      }
      this.#contexts.clear();
    }
  }

  /** Resolve one context digest through the connected registry. */
  async resolveContext(
    contextDigest: string,
  ): Promise<VerifiedAuthorizationContextTokenProjection> {
    const entry = await this.#lease(contextDigest);
    try {
      const state = await this.#verified(entry, false);
      const { assertAuthorizationContextHandleCurrentWasm } = await import(
        "../protocol_wasm.ts"
      );
      assertAuthorizationContextHandleCurrentWasm(
        state.handle,
        this.#policy(this.#now()),
      );
      return structuredClone(state.verified);
    } finally {
      entry.leases -= 1;
    }
  }

  /** Verify a presented request proof with exact route permissions. */
  async verifyRequest(
    request: AuthorizationProviderRequest,
  ): Promise<CachedRequestVerificationResult> {
    try {
      const entry = await this.#lease(request.contextDigest);
      try {
        if (entry.revokedAt !== undefined) {
          return requestFailure("PermissionDenied", "/authorization-context");
        }
        const state = await this.#verified(entry, false);
        if (request.sessionKey !== state.verified.context.sessionKey) {
          return requestFailure("InvalidInput", "/session-key");
        }
        const { verifyAuthorizationRequestWasm } = await import(
          "../protocol_wasm.ts"
        );
        const result = await verifyAuthorizationRequestWasm({
          contextHandle: state.handle,
          subject: request.subject,
          reply: request.reply,
          payload: request.payload,
          iat: request.iat,
          requestId: request.requestId,
          proof: request.proof,
          requiredPermissions: request.requiredPermissions,
          policy: this.#policy(this.#now()),
        });
        if (!result.ok) return result;
        if (
          result.contextDigest !== state.verified.contextDigest ||
          entry.revokedAt !== undefined
        ) {
          return requestFailure("PermissionDenied", "/authorization-context");
        }
        return { ...result, context: state.verified.context };
      } finally {
        entry.leases -= 1;
      }
    } catch (error) {
      if (error instanceof AuthorizationProviderUnavailableError) throw error;
      return requestFailure("InvalidInput", "/authorization-context");
    }
  }

  /** Verify a presented event proof, including historical issuer evidence. */
  async verifyEvent(
    event: AuthorizationProviderEvent,
  ): Promise<CachedEventVerificationResult> {
    try {
      const entry = await this.#lease(event.contextDigest);
      try {
        const state = await this.#verified(entry, true);
        if (event.sessionKey !== state.verified.context.sessionKey) {
          return eventFailure("InvalidInput", "/session-key");
        }
        const { verifyAuthorizationEventWasm } = await import(
          "../protocol_wasm.ts"
        );
        const result = await verifyAuthorizationEventWasm({
          contextHandle: state.handle,
          subject: event.subject,
          payload: event.payload,
          eventId: event.eventId,
          eventTime: event.eventTime,
          proof: event.proof,
          requiredPermissions: event.requiredPermissions,
          policy: this.#policy(this.#now()),
          revokedAt: entry.revokedAt ?? null,
        });
        if (!result.ok) return result;
        if (
          result.contextDigest !== state.verified.contextDigest ||
          entry.revokedAt !== undefined
        ) {
          return eventFailure("EventRevoked", "/authorization-context");
        }
        return { ...result, context: state.verified.context };
      } finally {
        entry.leases -= 1;
      }
    } catch (error) {
      if (error instanceof AuthorizationProviderUnavailableError) throw error;
      return eventFailure("InvalidInput", "/authorization-context");
    }
  }

  async #entry(contextDigest: string): Promise<ProviderContextEntry> {
    this.#requireAvailable();
    const existing = this.#contexts.get(contextDigest);
    if (existing) {
      this.#contexts.delete(contextDigest);
      this.#contexts.set(contextDigest, existing);
      return existing;
    }
    let pending = this.#inFlight.get(contextDigest);
    if (!pending) {
      this.#makeRoom();
      pending = this.#load(contextDigest);
    }
    this.#inFlight.set(contextDigest, pending);
    try {
      return await pending;
    } finally {
      this.#inFlight.delete(contextDigest);
    }
  }

  async #lease(contextDigest: string): Promise<ProviderContextEntry> {
    const entry = await this.#entry(contextDigest);
    entry.leases += 1;
    this.#contexts.delete(contextDigest);
    this.#contexts.set(contextDigest, entry);
    return entry;
  }

  #makeRoom(): void {
    if (this.#contexts.size + this.#inFlight.size < MAX_CACHED_CONTEXTS) return;
    const oldest = [...this.#contexts].find(([, entry]) => entry.leases === 0);
    if (!oldest) {
      throw new AuthorizationProviderUnavailableError(
        "provider context cache capacity reached",
      );
    }
    this.#contexts.delete(oldest[0]);
    void oldest[1].watch?.return?.();
  }

  async #load(contextDigest: string): Promise<ProviderContextEntry> {
    assertDigest(contextDigest);
    this.#contextResolves += 1;
    const watch = await this.#registry.watchRevocation(contextDigest);
    try {
      const contextEntry = await this.#registry.getContext(contextDigest);
      if (!contextEntry) throw new Error("authorization context is missing");
      const context = parseJsonRecord(contextEntry.value);
      const issuerKeyId = String(context.issuerKeyId ?? "");
      if (!issuerKeyId) {
        throw new Error("authorization context issuer is missing");
      }
      const issuer = await this.#issuer(issuerKeyId);
      const entry: ProviderContextEntry = {
        contextDigest,
        context,
        issuer,
        leases: 0,
        watch: watch.iterator,
      };
      let initial = watch.initialPending;
      while (initial > 0) {
        const result = await watch.iterator.next();
        if (result.done) {
          throw new Error("authorization revocation watch ended");
        }
        this.#applyRevocation(entry, result.value);
        initial -= 1;
      }
      this.#contexts.set(contextDigest, entry);
      void this.#watchRevocation(entry, watch.iterator);
      return entry;
    } catch (error) {
      await watch.iterator.return?.();
      throw error;
    }
  }

  async #watchRevocation(
    entry: ProviderContextEntry,
    iterator: AsyncIterator<import("@nats-io/kv").KvWatchEntry>,
  ): Promise<void> {
    try {
      while (!this.#stopped) {
        const result = await iterator.next();
        if (result.done) break;
        this.#applyRevocation(entry, result.value);
      }
    } catch {
      this.#connected = false;
    } finally {
      await iterator.return?.();
    }
  }

  #applyRevocation(
    entry: ProviderContextEntry,
    value: import("@nats-io/kv").KvWatchEntry,
  ): void {
    const event = registryWatchEntry(value);
    this.#revocationRevision = Math.max(
      this.#revocationRevision,
      event.revision,
    );
    this.#lastUpdateAt = this.#now();
    if (event.operation === "put") {
      entry.revokedAt = parseRevocation(event.value);
    }
  }

  async #issuer(keyId: string): Promise<AuthorizationIssuerKey> {
    const cached = this.#issuerKeys.get(keyId);
    if (cached) return cached;
    const response = await this.#cache.fetch(
      new URL(
        `/auth/keys/${encodeURIComponent(keyId)}`,
        this.#cache.trellisUrl,
      ),
      { cache: "no-store" },
    );
    if (!response.ok) throw new Error("authorization issuer is unavailable");
    const issuer = await response.json() as AuthorizationIssuerKey;
    if (issuer.keyId !== keyId) {
      throw new Error("authorization issuer key mismatch");
    }
    this.#issuerKeys.set(keyId, issuer);
    return issuer;
  }

  #verified(entry: ProviderContextEntry, historical: boolean) {
    const key = historical ? "historical" : "live";
    let pending = entry[key];
    if (!pending) {
      this.#contextVerifications += 1;
      pending = import("../protocol_wasm.ts").then(
        ({ createAuthorizationContextHandleWasm }) =>
          createAuthorizationContextHandleWasm({
            issuer: entry.issuer,
            context: entry.context,
            policy: this.#policy(this.#now()),
            historical,
          }),
      );
      entry[key] = pending;
    }
    return pending;
  }

  #policy(nowUnixSeconds: number) {
    return authorizationContextVerificationPolicy(
      this.#cache.bundle().policy,
      nowUnixSeconds,
    );
  }

  #requireAvailable(): void {
    if (!this.#started || this.#stopped || !this.#connected) {
      throw new AuthorizationProviderUnavailableError(
        "authorization provider is unavailable",
      );
    }
  }
}

function parseJsonRecord(value: Uint8Array): Record<string, unknown> {
  return parseRecord(
    JSON.parse(new TextDecoder().decode(value)),
    "authorization context",
  );
}

function parseRevocation(value: Uint8Array): number {
  const record = parseJsonRecord(value);
  const revokedAt = record.revokedAt;
  if (!Number.isSafeInteger(revokedAt) || Number(revokedAt) < 0) {
    throw new Error("authorization revocation is invalid");
  }
  return Number(revokedAt);
}

function parseRecord(value: unknown, kind: string): Record<string, unknown> {
  if (value === null || typeof value !== "object" || Array.isArray(value)) {
    throw new Error(`${kind} is invalid`);
  }
  return value as Record<string, unknown>;
}

function assertDigest(value: string): void {
  if (!/^[A-Za-z0-9_-]{43}$/.test(value)) {
    throw new Error("authorization context digest is invalid");
  }
}

function requestFailure(
  code: AuthorizationVerificationErrorCode,
  path: string,
): VerificationFailure {
  return { ok: false, error: { code, path } };
}

function eventFailure(
  code: AuthorizationVerificationErrorCode,
  path: string,
): VerificationFailure {
  return { ok: false, error: { code, path } };
}
