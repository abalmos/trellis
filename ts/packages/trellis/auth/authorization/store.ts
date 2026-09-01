import type { AuthorizationClientState } from "./types.ts";

const CLIENT_STATE_FORMAT = "trellis.authorization-client-state.v1";
const CLIENT_TRUST_FORMAT = "trellis.authorization-client-trust.v1";

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
        (this.#state?.contextDigest ?? null) !==
          expectedContextDigest) ||
      (expectedBootstrapJwt !== undefined &&
        (this.#state?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
    ) return Promise.resolve(false);
    if (this.#state) {
      this.#state.context = null;
      this.#state.contextDigest = null;
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

/** Reject trust rollback, root replacement, and same-generation equivocation. */
export function validateAuthorizationClientStateTransition(
  current: AuthorizationClientState | undefined,
  next: AuthorizationClientState,
): void {
  if (
    next.format !== CLIENT_STATE_FORMAT ||
    next.trust.format !== CLIENT_TRUST_FORMAT ||
    !next.binding.trim() ||
    !Number.isFinite(next.serverClockOffsetMs)
  ) {
    throw new Error("authorization client state is invalid");
  }
  if (
    (next.context === null) !== (next.contextDigest === null) ||
    (next.context === null) !== (next.contextExpiresAt === null) ||
    (next.context === null) !== (next.routing === null)
  ) {
    throw new Error("authorization context and routing state are not atomic");
  }
  if (
    next.session === null &&
    (next.context !== null || next.routing !== null ||
      next.runtime !== undefined)
  ) {
    throw new Error("authorization sessionless state retains runtime material");
  }
  if (
    next.runtime &&
    (!next.session || next.runtime.sessionId !== next.session.sessionId)
  ) {
    throw new Error("authorization runtime binding session mismatch");
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
  if (
    current.session &&
    (!next.session || current.session.sessionId !== next.session.sessionId)
  ) {
    throw new Error(
      "active authorization session must be ended before replacement",
    );
  }
}

export type {
  AuthorizationClientState,
  AuthorizationContextBundle,
  AuthorizationRoutingMaterial,
  AuthorizationTrustState,
} from "./types.ts";
