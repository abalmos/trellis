import {
  type AuthorizationClientState,
  type AuthorizationContextStore,
  validateAuthorizationClientStateTransition,
} from "../authorization_context.ts";

const DB_NAME = "trellis-auth";
const DB_VERSION = 2;
const STORE_NAME = "keys";
const KEY_ID = "trellis-session-key";
const TRUST_ID = "trellis-authorization-trust";

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

export type StoredKeyPair = {
  id: string;
  privateKey: CryptoKey;
  publicKey: CryptoKey;
  publicKeyRaw: Uint8Array;
  seed: Uint8Array;
  sessionId?: string;
  createdAt: number;
  persistence?: "remembered";
  expiresAt?: number;
};

export type StoredKeyPairOptions = {
  expiresAt?: number;
};

export async function storeKeyPair(
  keyPair: CryptoKeyPair,
  publicKeyRaw: Uint8Array,
  seed: Uint8Array,
  options: StoredKeyPairOptions = {},
): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);

    const record: StoredKeyPair = {
      id: KEY_ID,
      privateKey: keyPair.privateKey,
      publicKey: keyPair.publicKey,
      publicKeyRaw,
      seed,
      createdAt: Date.now(),
      persistence: "remembered",
      ...(options.expiresAt === undefined
        ? {}
        : { expiresAt: options.expiresAt }),
    };

    const request = store.put(record);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve();

    tx.oncomplete = () => db.close();
  });
}

export async function loadKeyPair(): Promise<StoredKeyPair | null> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);

    const request = store.get(KEY_ID);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      const result = request.result as StoredKeyPair | undefined;
      if (result !== undefined && !(result.seed instanceof Uint8Array)) {
        store.delete(KEY_ID);
        resolve(null);
        return;
      }
      if (result?.expiresAt !== undefined && result.expiresAt <= Date.now()) {
        store.delete(KEY_ID);
        resolve(null);
        return;
      }
      resolve(result ?? null);
    };

    tx.oncomplete = () => db.close();
  });
}

/** Associates the durable session ID returned after browser binding. */
export async function storeSessionId(sessionId: string): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);
    const request = store.get(KEY_ID);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => {
      const record = request.result as StoredKeyPair | undefined;
      if (!record) {
        reject(new Error("session key is unavailable"));
        return;
      }
      store.put({ ...record, sessionId });
    };
    tx.oncomplete = () => {
      db.close();
      resolve();
    };
    tx.onerror = () => reject(tx.error);
  });
}

/** Origin-scoped atomic IndexedDB authorization context store. */
export class BrowserAuthorizationContextStore
  implements AuthorizationContextStore {
  readonly #id: string;

  constructor(scope: string) {
    this.#id = `${TRUST_ID}:${new URL(scope).origin}`;
  }

  async load(): Promise<AuthorizationClientState | undefined> {
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readonly");
      const request = tx.objectStore(STORE_NAME).get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const record = request.result as
          | (AuthorizationClientState & { id: string })
          | undefined;
        if (!record) {
          resolve(undefined);
          return;
        }
        const { id: _, ...state } = record;
        resolve(structuredClone(state));
      };
      tx.oncomplete = () => db.close();
    });
  }

  async commit(
    state: AuthorizationClientState,
  ): Promise<AuthorizationClientState> {
    validateAuthorizationClientStateTransition(undefined, state);
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as
          | (AuthorizationClientState & { id: string })
          | undefined;
        if (current) {
          const { id: _, ...currentState } = current;
          try {
            validateAuthorizationClientStateTransition(currentState, state);
          } catch (error) {
            tx.abort();
            reject(error);
            return;
          }
        }
        store.put({ id: this.#id, ...structuredClone(state) });
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
    const db = await openDB();
    return await new Promise((resolve, reject) => {
      let cleared = false;
      const tx = db.transaction(STORE_NAME, "readwrite");
      const store = tx.objectStore(STORE_NAME);
      const request = store.get(this.#id);
      request.onerror = () => reject(request.error);
      request.onsuccess = () => {
        const current = request.result as
          | (AuthorizationClientState & { id: string })
          | undefined;
        if (
          (expectedContextDigest !== undefined &&
            (current?.context?.contextDigest ?? null) !==
              expectedContextDigest) ||
          (expectedBootstrapJwt !== undefined &&
            (current?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
        ) return;
        cleared = true;
        if (current) {
          store.put({
            ...current,
            context: null,
            contextExpiresAt: null,
            routing: null,
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
}

export async function deleteKeyPair(): Promise<void> {
  const db = await openDB();
  return new Promise((resolve, reject) => {
    const tx = db.transaction(STORE_NAME, "readwrite");
    const store = tx.objectStore(STORE_NAME);

    const request = store.delete(KEY_ID);
    request.onerror = () => reject(request.error);
    request.onsuccess = () => resolve();

    tx.oncomplete = () => db.close();
  });
}

export async function hasKeyPair(): Promise<boolean> {
  const keyPair = await loadKeyPair();
  return keyPair !== null;
}
