import type { NatsConnection } from "@nats-io/nats-core";

import type {
  AuthorizationContextHandle,
  AuthorizationIssuerKey,
  AuthorizationVerificationErrorCode,
  VerifiedAuthorizationContextTokenProjection,
  VerifiedAuthorizationEventPublisher,
} from "../protocol_wasm.ts";
import { canonicalizeJsonValue } from "../utils.ts";
import type { AuthorizationContextCache } from "./client_context.ts";
import {
  type AuthorizationRegistryIoCounters,
  AuthorizationRegistryReader,
  type RegistryWatchEntry,
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
  constructor(message: string, cause?: unknown) {
    super(message, { cause });
    this.name = "AuthorizationProviderUnavailableError";
  }
}

class InvalidIssuerResponseError extends Error {}

/** Provider attach options. */
export type AuthorizationProviderCacheOptions = { now?: () => number };

type ProviderContextEntry = {
  contextDigest: string;
  context: Record<string, unknown>;
  issuer: AuthorizationIssuerKey;
  generation: number;
  covered: boolean;
  disposed: boolean;
  resourcesDisposed: boolean;
  revokedAt?: number;
  leases: number;
  watch?: AsyncIterator<RegistryWatchEntry>;
  closeWatch?: () => void;
  live?: Promise<{
    handle: AuthorizationContextHandle;
    verified: VerifiedAuthorizationContextTokenProjection;
  }>;
  historical?: Promise<{
    handle: AuthorizationContextHandle;
    verified: VerifiedAuthorizationContextTokenProjection;
  }>;
};

type PendingContextEntry = {
  generation: number;
  owner: symbol;
  promise: Promise<ProviderContextEntry>;
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
  readonly #inFlight = new Map<string, PendingContextEntry>();
  readonly #ownIssuer: AuthorizationIssuerKey;
  #contextResolves = 0;
  #contextVerifications = 0;
  #revocationRevision = 0;
  #lastUpdateAt = 0;
  #stopped = false;
  #connected = true;
  #started = false;
  #generation = 0;

  private constructor(
    registry: AuthorizationRegistryReader,
    cache: AuthorizationContextCache,
    options: AuthorizationProviderCacheOptions,
  ) {
    this.#registry = registry;
    this.#cache = cache;
    this.#now = options.now ?? cache.correctedNowSeconds.bind(cache);
    this.#ownIssuer = structuredClone(cache.bundle().issuer);
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
    if (this.#started && !this.#stopped) return;
    this.#generation += 1;
    this.#stopped = false;
    this.#started = true;
  }

  /** Stop verification without closing the caller-owned NATS connection. */
  stop(): void {
    this.#generation += 1;
    this.#stopped = true;
    for (const entry of this.#contexts.values()) this.#invalidate(entry);
    this.#contexts.clear();
    this.#inFlight.clear();
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
      this.#generation += 1;
      for (const entry of this.#contexts.values()) this.#invalidate(entry);
      this.#contexts.clear();
      this.#inFlight.clear();
    }
  }

  /** Resolve one context digest through the connected registry. */
  async resolveContext(
    contextDigest: string,
  ): Promise<VerifiedAuthorizationContextTokenProjection> {
    const entry = await this.#lease(contextDigest, false);
    try {
      const state = await this.#verified(entry, false);
      this.#requireEntry(entry);
      const { assertAuthorizationContextHandleCurrentWasm } = await import(
        "../protocol_wasm.ts"
      );
      assertAuthorizationContextHandleCurrentWasm(
        state.handle,
        this.#policy(this.#now()),
      );
      this.#requireEntry(entry);
      if (entry.revokedAt !== undefined) {
        throw new Error("authorization context is revoked");
      }
      return structuredClone(state.verified);
    } finally {
      this.#release(entry);
    }
  }

  /** Verify a presented request proof with exact route permissions. */
  async verifyRequest(
    request: AuthorizationProviderRequest,
  ): Promise<CachedRequestVerificationResult> {
    try {
      const entry = await this.#lease(request.contextDigest, false);
      try {
        this.#requireEntry(entry);
        if (entry.revokedAt !== undefined) {
          return requestFailure("PermissionDenied", "/authorization-context");
        }
        const state = await this.#verified(entry, false);
        this.#requireEntry(entry);
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
        this.#requireEntry(entry);
        if (!result.ok) return result;
        if (
          result.contextDigest !== request.contextDigest ||
          state.verified.contextDigest !== request.contextDigest ||
          entry.revokedAt !== undefined
        ) {
          return requestFailure("PermissionDenied", "/authorization-context");
        }
        return { ...result, context: state.verified.context };
      } finally {
        this.#release(entry);
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
      const entry = await this.#lease(event.contextDigest, true);
      try {
        this.#requireEntry(entry);
        const state = await this.#verified(entry, true);
        this.#requireEntry(entry);
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
        this.#requireEntry(entry);
        if (!result.ok) return result;
        if (
          result.contextDigest !== event.contextDigest ||
          state.verified.contextDigest !== event.contextDigest ||
          entry.revokedAt !== undefined
        ) {
          return eventFailure("EventRevoked", "/authorization-context");
        }
        return { ...result, context: state.verified.context };
      } finally {
        this.#release(entry);
      }
    } catch (error) {
      if (error instanceof AuthorizationProviderUnavailableError) throw error;
      return eventFailure("InvalidInput", "/authorization-context");
    }
  }

  async #entry(
    contextDigest: string,
    historical: boolean,
  ): Promise<ProviderContextEntry> {
    assertDigest(contextDigest);
    this.#requireAvailable();
    const existing = this.#contexts.get(contextDigest);
    if (existing) {
      this.#requireEntry(existing);
      this.#contexts.delete(contextDigest);
      this.#contexts.set(contextDigest, existing);
      return existing;
    }
    let pending = this.#inFlight.get(contextDigest);
    if (!pending) {
      this.#makeRoom();
      const generation = this.#generation;
      const owner = Symbol(contextDigest);
      pending = {
        generation,
        owner,
        promise: this.#load(contextDigest, historical, generation, owner),
      };
      this.#inFlight.set(contextDigest, pending);
    }
    try {
      return await pending.promise;
    } finally {
      if (this.#inFlight.get(contextDigest) === pending) {
        this.#inFlight.delete(contextDigest);
      }
    }
  }

  async #lease(
    contextDigest: string,
    historical: boolean,
  ): Promise<ProviderContextEntry> {
    const entry = await this.#entry(contextDigest, historical);
    this.#requireEntry(entry);
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
    this.#invalidate(oldest[1]);
  }

  async #load(
    contextDigest: string,
    historical: boolean,
    generation: number,
    owner: symbol,
  ): Promise<ProviderContextEntry> {
    assertDigest(contextDigest);
    this.#contextResolves += 1;
    const watch = await this.#registryIo(
      "authorization revocation watch is unavailable",
      () => this.#registry.watchRevocation(contextDigest),
    );
    let entry: ProviderContextEntry | undefined;
    let watchOwned = false;
    try {
      const contextEntry = await this.#registryIo(
        "authorization context registry is unavailable",
        () => this.#registry.getContext(contextDigest),
      );
      if (!contextEntry) {
        throw new AuthorizationProviderUnavailableError(
          "authorization context is missing",
        );
      }
      let context: Record<string, unknown>;
      try {
        context = parseJsonRecord(contextEntry.value);
      } catch (error) {
        throw new AuthorizationProviderUnavailableError(
          "authorization context registry entry is malformed",
          error,
        );
      }
      const issuerKeyId = String(context.issuerKeyId ?? "");
      if (!issuerKeyId) {
        throw new AuthorizationProviderUnavailableError(
          "authorization context issuer is missing",
        );
      }
      const issuer = await this.#issuer(issuerKeyId);
      entry = {
        contextDigest,
        context,
        issuer,
        generation,
        covered: false,
        disposed: false,
        resourcesDisposed: false,
        leases: 0,
        watch: watch.iterator,
        closeWatch: watch.close,
      };
      entry.covered = true;
      watchOwned = true;
      void this.#watchRevocation(entry, watch.iterator);
      const revocation = await this.#registryIo(
        "authorization revocation registry is unavailable",
        () => this.#registry.getRevocation(contextDigest),
      );
      if (revocation) {
        this.#revocationRevision = Math.max(
          this.#revocationRevision,
          revocation.revision,
        );
        this.#lastUpdateAt = this.#now();
        entry.revokedAt = Math.max(
          entry.revokedAt ?? 0,
          parseRevocation(revocation.value),
        );
      }
      const verified = await this.#verified(entry, historical);
      if (verified.verified.contextDigest !== contextDigest) {
        throw new Error(
          "authorization context digest does not match its registry key",
        );
      }
      this.#requirePendingOwner(contextDigest, generation, owner);
      if (entry.disposed || !entry.covered) {
        throw new AuthorizationProviderUnavailableError(
          "authorization context lost revocation coverage",
        );
      }
      this.#contexts.set(contextDigest, entry);
      return entry;
    } catch (error) {
      if (entry) {
        this.#invalidate(entry);
      } else if (!watchOwned) {
        watch.close();
      }
      throw error;
    }
  }

  async #watchRevocation(
    entry: ProviderContextEntry,
    iterator: AsyncIterator<RegistryWatchEntry>,
  ): Promise<void> {
    try {
      while (entry.covered && !entry.disposed) {
        const result = await iterator.next();
        if (result.done) return;
        this.#applyRevocation(entry, result.value);
      }
    } catch {
      // Coverage is invalidated below; the next request resynchronizes it.
    } finally {
      this.#invalidate(entry);
      try {
        await iterator.return?.();
      } catch {
        // The entry is already unusable and will reload through a new watch.
      }
    }
  }

  #applyRevocation(
    entry: ProviderContextEntry,
    event: RegistryWatchEntry,
  ): void {
    this.#revocationRevision = Math.max(
      this.#revocationRevision,
      event.revision,
    );
    this.#lastUpdateAt = this.#now();
    if (event.operation !== "put") {
      throw new AuthorizationProviderUnavailableError(
        "authorization revocation registry entry disappeared",
      );
    }
    entry.revokedAt = Math.max(
      entry.revokedAt ?? 0,
      parseRevocation(event.value),
    );
  }

  async #issuer(keyId: string): Promise<AuthorizationIssuerKey> {
    if (this.#ownIssuer.keyId === keyId) return this.#ownIssuer;
    for (const entry of this.#contexts.values()) {
      if (!entry.disposed && entry.issuer.keyId === keyId) return entry.issuer;
    }
    const url = new URL(
      `/auth/keys/${encodeURIComponent(keyId)}`,
      this.#cache.trellisUrl,
    );
    if (url.origin !== new URL(this.#cache.trellisUrl).origin) {
      throw new Error("authorization issuer origin is invalid");
    }
    let response: Response;
    try {
      response = await this.#cache.fetch(url, {
        cache: "no-store",
        redirect: "error",
      });
    } catch (error) {
      throw new AuthorizationProviderUnavailableError(
        "authorization issuer is unavailable",
        error,
      );
    }
    if (!response.ok) {
      throw new AuthorizationProviderUnavailableError(
        `authorization issuer is unavailable (${response.status})`,
      );
    }
    let bytes: Uint8Array;
    try {
      bytes = await readBoundedResponse(response, 16_384);
    } catch (error) {
      throw new AuthorizationProviderUnavailableError(
        "authorization issuer response is unavailable",
        error,
      );
    }
    let record: Record<string, unknown>;
    try {
      record = parseRecord(
        JSON.parse(new TextDecoder().decode(bytes)),
        "authorization issuer",
      );
    } catch (error) {
      throw new AuthorizationProviderUnavailableError(
        "authorization issuer response is unavailable",
        error,
      );
    }
    try {
      if (record.keyId !== keyId) {
        throw new Error("authorization issuer key mismatch");
      }
      if (typeof record.publicKey !== "string") {
        throw new Error("authorization issuer public key is invalid");
      }
      assertDigest(keyId);
      assertDigest(record.publicKey);
      if (
        record.state !== "active" && record.state !== "retired" &&
        record.state !== "revoked"
      ) {
        throw new Error("authorization issuer state is invalid");
      }
      return {
        keyId,
        publicKey: record.publicKey,
        state: record.state,
      };
    } catch (error) {
      throw new AuthorizationProviderUnavailableError(
        "authorization issuer response is unavailable",
        error,
      );
    }
  }

  #verified(entry: ProviderContextEntry, historical: boolean) {
    const key = historical ? "historical" : "live";
    let pending = entry[key];
    if (!pending) {
      this.#contextVerifications += 1;
      const created = import("../protocol_wasm.ts").then(
        ({ createAuthorizationContextHandleWasm }) =>
          createAuthorizationContextHandleWasm({
            issuer: entry.issuer,
            context: entry.context,
            policy: this.#policy(this.#now()),
            historical,
          }),
      ).then((verified) => {
        if (entry.resourcesDisposed) {
          verified.handle.free();
          throw new AuthorizationProviderUnavailableError(
            "authorization context lost revocation coverage",
          );
        }
        return verified;
      });
      pending = created.catch((error) => {
        if (entry[key] === pending) entry[key] = undefined;
        throw error;
      });
      entry[key] = pending;
    }
    return pending;
  }

  #policy(nowUnixSeconds: number) {
    try {
      return this.#cache.verificationPolicy(nowUnixSeconds);
    } catch (error) {
      throw new AuthorizationProviderUnavailableError(
        "authorization verification policy is unavailable",
        error,
      );
    }
  }

  #requirePendingOwner(
    contextDigest: string,
    generation: number,
    owner: symbol,
  ): void {
    this.#requireAvailable();
    const pending = this.#inFlight.get(contextDigest);
    if (
      generation !== this.#generation || pending?.generation !== generation ||
      pending.owner !== owner
    ) {
      throw new AuthorizationProviderUnavailableError(
        "authorization context load became stale",
      );
    }
  }

  #requireEntry(entry: ProviderContextEntry): void {
    this.#requireAvailable();
    if (
      entry.generation !== this.#generation || !entry.covered ||
      entry.disposed || this.#contexts.get(entry.contextDigest) !== entry
    ) {
      throw new AuthorizationProviderUnavailableError(
        "authorization context lost revocation coverage",
      );
    }
  }

  #invalidate(entry: ProviderContextEntry): void {
    entry.covered = false;
    entry.disposed = true;
    entry.closeWatch?.();
    entry.closeWatch = undefined;
    if (this.#contexts.get(entry.contextDigest) === entry) {
      this.#contexts.delete(entry.contextDigest);
    }
    if (entry.leases === 0) this.#disposeResources(entry);
  }

  #release(entry: ProviderContextEntry): void {
    entry.leases -= 1;
    if (entry.disposed && entry.leases === 0) this.#disposeResources(entry);
  }

  #disposeResources(entry: ProviderContextEntry): void {
    if (entry.resourcesDisposed) return;
    entry.resourcesDisposed = true;
    for (const pending of [entry.live, entry.historical]) {
      void pending?.then(({ handle }) => handle.free()).catch(() => {});
    }
  }

  async #registryIo<T>(
    message: string,
    operation: () => Promise<T>,
  ): Promise<T> {
    try {
      return await operation();
    } catch (error) {
      if (error instanceof AuthorizationProviderUnavailableError) throw error;
      throw new AuthorizationProviderUnavailableError(message, error);
    }
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

async function readBoundedResponse(
  response: Response,
  limit: number,
): Promise<Uint8Array> {
  const contentLength = response.headers.get("content-length");
  if (contentLength !== null && Number(contentLength) > limit) {
    throw new InvalidIssuerResponseError(
      "authorization issuer response is too large",
    );
  }
  if (!response.body) return new Uint8Array();
  const reader = response.body.getReader();
  const chunks: Uint8Array[] = [];
  let length = 0;
  try {
    for (;;) {
      const { done, value } = await reader.read();
      if (done) break;
      length += value.byteLength;
      if (length > limit) {
        throw new InvalidIssuerResponseError(
          "authorization issuer response is too large",
        );
      }
      chunks.push(value);
    }
  } finally {
    reader.releaseLock();
  }
  const bytes = new Uint8Array(length);
  let offset = 0;
  for (const chunk of chunks) {
    bytes.set(chunk, offset);
    offset += chunk.byteLength;
  }
  return bytes;
}

function parseRevocation(value: Uint8Array): number {
  try {
    const record = parseJsonRecord(value);
    const revokedAt = record.revokedAt;
    if (!Number.isSafeInteger(revokedAt) || Number(revokedAt) <= 0) {
      throw new Error("authorization revocation is invalid");
    }
    return Number(revokedAt);
  } catch (error) {
    throw new AuthorizationProviderUnavailableError(
      "authorization revocation registry is unavailable",
      error,
    );
  }
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
