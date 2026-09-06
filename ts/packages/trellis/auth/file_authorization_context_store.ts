import {
  chmod,
  mkdir,
  readFile,
  rename,
  unlink,
  writeFile,
} from "node:fs/promises";
import { dirname } from "node:path";
import { ulid } from "ulid";

import type {
  AuthorizationClientState,
  AuthorizationContextStore,
} from "./authorization_context.ts";
import { validateAuthorizationClientStateTransition } from "./authorization_context.ts";

/**
 * Crash-safe private-file store for one active client process.
 *
 * Each path must have exactly one active writer; use a distinct private path per
 * service/client process.
 */
export class FileAuthorizationContextStore
  implements AuthorizationContextStore {
  #operation: Promise<void> = Promise.resolve();

  constructor(readonly path: string) {
    if (!path.trim()) {
      throw new Error("authorization context store path is empty");
    }
  }

  /** Load the current atomic client state. */
  load(): Promise<AuthorizationClientState | undefined> {
    return this.#run(() => this.#load());
  }

  /** Validate and atomically replace the current client state. */
  commit(
    state: AuthorizationClientState,
  ): Promise<AuthorizationClientState> {
    return this.#run(async () => {
      validateAuthorizationClientStateTransition(await this.#load(), state);
      await this.#write(state);
      return structuredClone(state);
    });
  }

  /** Clear current context state while retaining trust. */
  clearContext(
    expectedContextDigest?: string | null,
    expectedBootstrapJwt?: string | null,
  ): Promise<boolean> {
    return this.#run(async () => {
      const state = await this.#load();
      if (
        (expectedContextDigest !== undefined &&
          (state?.contextDigest ?? null) !== expectedContextDigest) ||
        (expectedBootstrapJwt !== undefined &&
          (state?.routing?.bootstrapJwt ?? null) !== expectedBootstrapJwt)
      ) return false;
      if (state) {
        await this.#write({
          ...state,
          context: null,
          contextDigest: null,
          contextExpiresAt: null,
          routing: null,
          serverClockOffsetMs: 0,
        });
      }
      return true;
    });
  }

  /** Explicitly remove the complete trust floor and context. */
  resetTrust(): Promise<void> {
    return this.#run(async () => {
      try {
        await unlink(this.path);
      } catch (error) {
        if (!isNotFound(error)) throw error;
      }
    });
  }

  async #load(): Promise<AuthorizationClientState | undefined> {
    try {
      return JSON.parse(
        await readFile(this.path, "utf8"),
      ) as AuthorizationClientState;
    } catch (error) {
      if (isNotFound(error)) return undefined;
      throw error;
    }
  }

  async #write(state: AuthorizationClientState): Promise<void> {
    await mkdir(dirname(this.path), { recursive: true });
    const temporary = `${this.path}.${ulid()}.tmp`;
    await writeFile(temporary, JSON.stringify(state, null, 2), { mode: 0o600 });
    await chmod(temporary, 0o600);
    await rename(temporary, this.path);
  }

  #run<T>(operation: () => Promise<T>): Promise<T> {
    const result = this.#operation.then(operation);
    this.#operation = result.then(() => undefined, () => undefined);
    return result;
  }
}

function isNotFound(error: unknown): boolean {
  return error instanceof Error &&
    "code" in error && error.code === "ENOENT";
}
