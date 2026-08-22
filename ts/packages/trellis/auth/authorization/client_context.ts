import { Type } from "typebox";
import { Value } from "typebox/value";

import type {
  AuthorizationContextVerificationPolicyV1,
} from "../protocol_wasm.ts";
import { canonicalizeJsonValue } from "../utils.ts";
import {
  type AuthorizationContextPersistence,
  type AuthorizationContextStore,
  validateAuthorizationClientStateTransition,
} from "./store.ts";
import type {
  AuthorizationClientState,
  AuthorizationContextBundle,
  AuthorizationContextVerificationMaterial,
  AuthorizationRoutingMaterial,
  AuthorizationRuntimeBinding,
  AuthorizationSessionBinding,
  AuthorizationTrustBundle,
  VerifiedAuthorizationContext,
} from "./types.ts";

const CLIENT_STATE_FORMAT = "trellis.authorization-client-state.v1";
const CLIENT_TRUST_FORMAT = "trellis.authorization-client-trust.v1";

/** Verifies, persists, and exposes the current authorization context. */
export class AuthorizationContextCache {
  #bundle?: AuthorizationContextBundle;
  #verified?: VerifiedAuthorizationContext;
  #session?: AuthorizationSessionBinding;
  #runtime?: AuthorizationRuntimeBinding;
  #routing?: AuthorizationRoutingMaterial;
  #manifest?: unknown;
  #verificationPolicy?: AuthorizationContextVerificationPolicyV1;
  #clockOffsetMs = 0;
  #operation = 0;
  #refreshRequest?: () => void;
  #refreshRequestPending = false;

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
      (state.context === null) !== (state.contextDigest === null) ||
      (state.context === null) !== (state.contextExpiresAt === null) ||
      (state.context === null) !== (state.routing === null)
    ) {
      throw new Error("persisted context and routing material are not atomic");
    }
    this.#session = structuredClone(state.session);
    this.#runtime = state.runtime ? structuredClone(state.runtime) : undefined;
    if (!state.context || !state.routing) return false;
    const persistedContext = state.context.context;
    if (
      persistedContext !== null && typeof persistedContext === "object" &&
      !Array.isArray(persistedContext) &&
      typeof (persistedContext as Record<string, unknown>)
          .issuerManifestGeneration === "number" &&
      (persistedContext as Record<string, number>).issuerManifestGeneration <
        state.trust.minimumManifestGeneration
    ) {
      await this.store.clearContext(
        state.contextDigest,
        state.routing.bootstrapJwt,
      );
      return false;
    }
    return await this.installRecoverable(
      state.context,
      state.routing,
      nowUnixSeconds,
      state.contextExpiresAt ?? undefined,
      state.runtime,
    );
  }

  async installRecoverable(
    bundle: AuthorizationContextBundle,
    routing: AuthorizationRoutingMaterial,
    nowUnixSeconds = this.correctedNowSeconds(),
    knownContextExpiresAt?: number,
    runtime?: AuthorizationRuntimeBinding,
  ): Promise<boolean> {
    const verificationNow = knownContextExpiresAt !== undefined &&
        knownContextExpiresAt <= nowUnixSeconds
      ? knownContextExpiresAt - 1
      : nowUnixSeconds;
    const verified = await this.install(
      bundle,
      routing,
      verificationNow,
      () => true,
      runtime,
    );
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
    runtime?: AuthorizationRuntimeBinding,
  ): Promise<VerifiedAuthorizationContext> {
    const operation = ++this.#operation;
    const durable = await this.store.load();
    if (durable && durable.binding !== this.binding) {
      throw new Error(
        "authorization context storage belongs to another identity",
      );
    }
    const manifestPolicy = authorizationContextVerificationPolicy(
      bundle.trust.policy,
      nowUnixSeconds,
      durable?.trust.minimumManifestGeneration ?? 1,
    );
    const manifest = bundle.trust.manifest;
    const { verifyAuthorizationManifestWasm } = await import(
      "../protocol_wasm.ts"
    );
    const verifiedManifest = await verifyAuthorizationManifestWasm({
      root: bundle.trust.root,
      manifest,
      policy: manifestPolicy,
    });
    const minimumManifestGeneration = Math.max(
      verifiedManifest.generation,
      durable?.trust.minimumManifestGeneration ?? 0,
    );
    const verificationPolicy = authorizationContextVerificationPolicy(
      bundle.trust.policy,
      nowUnixSeconds,
      minimumManifestGeneration,
    );
    const verified = await verifyAuthorizationContext({
      bundle,
      manifest,
      policy: verificationPolicy,
    });
    const nextRuntime = runtime ?? durable?.runtime ?? this.#runtime;
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
      contextDigest: verified.contextDigest,
      contextExpiresAt: verified.context.expiresAt,
      routing: structuredClone(routing),
      ...(nextRuntime === undefined
        ? {}
        : { runtime: structuredClone(nextRuntime) }),
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
    this.#runtime = next.runtime ? structuredClone(next.runtime) : undefined;
    this.#routing = structuredClone(routing);
    this.#manifest = structuredClone(manifest);
    this.#verificationPolicy = structuredClone(verificationPolicy);
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
      this.#verified?.contextDigest ?? null,
      this.#routing?.bootstrapJwt ?? null,
    ];
  }

  /** Clear terminal state only if no newer context or route JWT replaced it. */
  async clearIfCurrent(
    guard: readonly [string | null, string | null],
  ): Promise<boolean> {
    if (
      (this.#verified?.contextDigest ?? null) !== guard[0] ||
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
    this.#manifest = undefined;
    this.#verificationPolicy = undefined;
    this.#session = undefined;
    this.#runtime = undefined;
    this.#routing = undefined;
    await this.store.resetTrust();
  }

  sessionBinding(): AuthorizationSessionBinding {
    if (!this.#session) {
      throw new Error("no authorization session is installed");
    }
    return structuredClone(this.#session);
  }

  /** Return reconnect metadata retained with the signed context. */
  runtimeBinding(): AuthorizationRuntimeBinding {
    if (!this.#runtime) {
      throw new Error("no authorization runtime metadata is installed");
    }
    return structuredClone(this.#runtime);
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

  /**
   * Return the installed chain material for a provider-side local verifier.
   *
   * This is an in-process handoff; it performs no registry I/O.
   */
  installedVerificationMaterial(): AuthorizationContextVerificationMaterial {
    if (
      !this.#bundle || !this.#verified || this.#manifest === undefined ||
      !this.#verificationPolicy
    ) {
      throw new Error("no authorization verification material is installed");
    }
    return {
      root: structuredClone(this.#bundle.trust.root),
      manifest: structuredClone(this.#manifest),
      context: structuredClone(this.#bundle.context),
      contextDigest: this.#verified.contextDigest,
      policy: structuredClone(this.#verificationPolicy),
      verified: structuredClone(this.#verified),
    };
  }

  /** Return the durable manifest-generation floor used by this cache. */
  minimumManifestGeneration(): number {
    return this.#verificationPolicy?.minimumManifestGeneration ??
      this.#verified?.manifestGeneration ?? 0;
  }

  /** Persist a newly accepted manifest floor without replacing the current context. */
  async advanceManifestFloor(
    generation: number,
    digest: string,
  ): Promise<boolean> {
    const operation = ++this.#operation;
    const current = await this.store.load();
    if (!current) throw new Error("authorization trust floor unavailable");
    if (generation < current.trust.minimumManifestGeneration) {
      throw new Error("authorization issuer manifest rolled back");
    }
    if (generation === current.trust.minimumManifestGeneration) {
      if (digest !== current.trust.manifestDigestAtMinimumGeneration) {
        throw new Error("authorization issuer manifest equivocated");
      }
      return false;
    }
    const next = structuredClone(current);
    next.trust.minimumManifestGeneration = generation;
    next.trust.manifestDigestAtMinimumGeneration = digest;
    validateAuthorizationClientStateTransition(current, next);
    if (operation !== this.#operation) {
      throw new Error("authorization trust floor advancement stopped");
    }
    const persisted = await this.store.commit(next);
    validateAuthorizationClientStateTransition(next, persisted);
    if (canonicalizeJsonValue(persisted) !== canonicalizeJsonValue(next)) {
      throw new Error(
        "authorization trust floor persistence did not commit exact state",
      );
    }
    if (this.#verificationPolicy) {
      this.#verificationPolicy.minimumManifestGeneration = generation;
    }
    return true;
  }

  /** Register the single existing refresh task wake callback. */
  registerRefreshRequest(callback: () => void): () => void {
    if (this.#refreshRequest) {
      throw new Error("authorization refresh callback is already registered");
    }
    this.#refreshRequest = callback;
    if (this.#refreshRequestPending) {
      this.#refreshRequestPending = false;
      callback();
    }
    return () => {
      if (this.#refreshRequest === callback) this.#refreshRequest = undefined;
    };
  }

  /** Wake the existing refresh task after trust-floor advance. */
  requestRefresh(): void {
    const callback = this.#refreshRequest;
    if (callback) {
      callback();
    } else {
      this.#refreshRequestPending = true;
    }
  }
}

/** Build the exact policy passed to the Rust/WASM verifier. */
export function authorizationContextVerificationPolicy(
  trust: AuthorizationTrustBundle["policy"],
  nowUnixSeconds: number,
  minimumManifestGeneration: number,
): AuthorizationContextVerificationPolicyV1 {
  return {
    nowUnixSeconds,
    allowedClockSkewSeconds: trust.allowedClockSkewSeconds,
    maximumContextLifetimeSeconds: trust.maximumContextLifetimeSeconds,
    maximumContextBytes: trust.maximumContextBytes,
    maximumPermissions: trust.maximumPermissions,
    maximumCapabilities: trust.maximumCapabilities,
    minimumManifestGeneration,
    refreshLeadSeconds: trust.refreshLeadSeconds,
    refreshJitterSeconds: trust.refreshJitterSeconds,
  };
}

/** Verify a complete authorization context chain through Rust/WASM. */
export async function verifyAuthorizationContext(args: {
  bundle: AuthorizationContextBundle;
  manifest: unknown;
  policy: AuthorizationContextVerificationPolicyV1;
}): Promise<VerifiedAuthorizationContext> {
  const { verifyAuthorizationContextWasm } = await import(
    "../protocol_wasm.ts"
  );
  const result = await verifyAuthorizationContextWasm({
    root: args.bundle.trust.root,
    manifest: args.manifest,
    context: args.bundle.context,
    policy: args.policy,
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
          issuedAt: Type.Integer(),
          notBefore: Type.Integer({ minimum: 0 }),
          expiresAt: Type.Integer({ minimum: 1 }),
        }),
        Type.Record(Type.String(), Type.Unknown()),
      ]),
    }),
    result,
  ) as VerifiedAuthorizationContext;
  return verified;
}

export type { AuthorizationContextPersistence };
