import {
  type AuthorizationClientState,
  type AuthorizationContextStore,
  validateAuthorizationClientStateTransition,
} from "../authorization_context.ts";

const DB_NAME = "trellis-auth";
const DB_VERSION = 2;
const STORE_NAME = "keys";
const INSTALLATION_ID = "trellis-browser-installation";

type BrowserInstallationRecord = {
  readonly id: string;
  readonly seed?: Uint8Array;
  readonly authorization?: AuthorizationClientState;
};

const temporaryInstallations = new Map<string, BrowserInstallationRecord>();

function openDB(): Promise<IDBDatabase> {
  return new Promise((resolve, reject) => {
    const request = indexedDB.open(DB_NAME, DB_VERSION);

    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve(request.result);

    request.onupgradeneeded = () => {
      const db = request.result;
      if (!db.objectStoreNames.contains(STORE_NAME)) {
        db.createObjectStore(STORE_NAME, { keyPath: "id" });
      }
    };
  });
}

/** Canonical opaque identity for one participant installation at one Trellis origin. */
export function browserInstallationScope(
  trellisUrl: string,
  participantId: string,
  participantArtifactDigest: string,
): string {
  return JSON.stringify([
    "trellis.browser-installation.v1",
    new URL(trellisUrl).origin,
    participantId,
    participantArtifactDigest,
  ]);
}

/** One participant-scoped browser credential, session, runtime, and trust installation. */
export class BrowserAuthorizationContextStore
  implements AuthorizationContextStore {
  readonly #id: string;
  readonly #temporary: boolean;

  constructor(
    scope: string,
    persistence: "remembered" | "temporary" = "remembered",
  ) {
    if (!scope.trim() || scope.length > 4_096) {
      throw new Error("browser installation scope is invalid");
    }
    this.#id = `${INSTALLATION_ID}:${scope}`;
    this.#temporary = persistence === "temporary";
  }

  async getOrCreateSessionSeed(): Promise<Uint8Array> {
    if (this.#temporary) {
      const current = temporaryInstallations.get(this.#id);
      if (current?.seed) return current.seed.slice();
      const seed = crypto.getRandomValues(new Uint8Array(32));
      temporaryInstallations.set(this.#id, { ...current, id: this.#id, seed });
      return seed.slice();
    }
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      let seed: Uint8Array;
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as BrowserInstallationRecord | undefined;
        seed = current?.seed instanceof Uint8Array
          ? current.seed
          : crypto.getRandomValues(new Uint8Array(32));
        if (!current?.seed) store.put({ ...current, id: this.#id, seed });
      };
      tx.oncomplete = () => {
        db.close();
        resolve(seed.slice());
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  async sessionId(): Promise<string | undefined> {
    return (await this.load())?.session?.sessionId;
  }

  async load(): Promise<AuthorizationClientState | undefined> {
    if (this.#temporary) {
      const state = temporaryInstallations.get(this.#id)?.authorization;
      return state ? structuredClone(state) : undefined;
    }
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readonly");
      const request = tx.objectStore(STORE_NAME).get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const state = (request.result as BrowserInstallationRecord | undefined)
          ?.authorization;
        resolve(state ? structuredClone(state) : undefined);
      };
      tx.oncomplete = () => db.close();
    });
  }

  async commit(
    state: AuthorizationClientState,
  ): Promise<AuthorizationClientState> {
    validateAuthorizationClientStateTransition(undefined, state);
    if (this.#temporary) {
      const current = temporaryInstallations.get(this.#id);
      validateAuthorizationClientStateTransition(current?.authorization, state);
      temporaryInstallations.set(this.#id, {
        ...current,
        id: this.#id,
        authorization: structuredClone(state),
      });
      return structuredClone(state);
    }
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as BrowserInstallationRecord | undefined;
        if (current?.authorization) {
          try {
            validateAuthorizationClientStateTransition(
              current.authorization,
              state,
            );
          } catch (error) {
            tx.abort();
            reject(error);
            return;
          }
        }
        store.put({
          ...current,
          id: this.#id,
          authorization: structuredClone(state),
        });
      };
      tx.oncomplete = () => {
        db.close();
        resolve(structuredClone(state));
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  async clearContext(
    expectedContextDigest?: string | null,
    expectedBootstrapJwt?: string | null,
  ): Promise<boolean> {
    if (this.#temporary) {
      const current = temporaryInstallations.get(this.#id);
      const state = current?.authorization;
      if (
        (expectedContextDigest !== undefined &&
          (state?.contextDigest ?? null) !== expectedContextDigest) ||
        (expectedBootstrapJwt !== undefined &&
          (state?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
      ) return false;
      if (state) {
        temporaryInstallations.set(this.#id, {
          ...current,
          id: this.#id,
          authorization: {
            ...state,
            context: null,
            contextDigest: null,
            contextExpiresAt: null,
            routing: null,
          },
        });
      }
      return true;
    }
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      let cleared = false;
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as BrowserInstallationRecord | undefined;
        const state = current?.authorization;
        if (
          (expectedContextDigest !== undefined &&
            (state?.contextDigest ?? null) !==
              expectedContextDigest) ||
          (expectedBootstrapJwt !== undefined &&
            (state?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
        ) return;
        cleared = true;
        if (state) {
          store.put({
            ...current,
            authorization: {
              ...state,
              context: null,
              contextDigest: null,
              contextExpiresAt: null,
              routing: null,
            },
          });
        }
      };
      tx.oncomplete = () => {
        db.close();
        resolve(cleared);
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  async resetTrust(): Promise<void> {
    if (this.#temporary) {
      temporaryInstallations.delete(this.#id);
      return;
    }
    const db = await openDB();
    await new Promise<void>((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readwrite");
      tx.objectStore(STORE_NAME).delete(this.#id);
      tx.oncomplete = () => {
        db.close();
        resolve();
      };
      tx.onerror = () => reject(tx.error);
    });
  }

  async endSession(expectedSessionId?: string): Promise<boolean> {
    if (this.#temporary) {
      const current = temporaryInstallations.get(this.#id);
      if (
        expectedSessionId &&
        current?.authorization?.session?.sessionId !== expectedSessionId
      ) return false;
      temporaryInstallations.set(this.#id, {
        id: this.#id,
        ...(current?.authorization
          ? {
            authorization: {
              ...current.authorization,
              session: null,
              context: null,
              contextDigest: null,
              contextExpiresAt: null,
              routing: null,
              runtime: undefined,
            },
          }
          : {}),
      });
      return true;
    }
    const db = await openDB();
    return await new Promise<boolean>((resolve, reject) => {
      let cleared = true;
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as BrowserInstallationRecord | undefined;
        if (
          expectedSessionId &&
          current?.authorization?.session?.sessionId !== expectedSessionId
        ) {
          cleared = false;
          return;
        }
        store.put({
          id: this.#id,
          ...(current?.authorization
            ? {
              authorization: {
                ...current.authorization,
                session: null,
                context: null,
                contextDigest: null,
                contextExpiresAt: null,
                routing: null,
                runtime: undefined,
              },
            }
            : {}),
        });
      };
      tx.oncomplete = () => {
        db.close();
        resolve(cleared);
      };
      tx.onerror = () => reject(tx.error);
    });
  }
}
