import { ulid } from "ulid";

import type {
  AuthorizationClientState,
  AuthorizationContextStore,
} from "./authorization_context.ts";
import { validateAuthorizationClientStateTransition } from "./authorization_context.ts";

type RuntimeFs = {
  readTextFile(path: string): Promise<string>;
  writeTextFile(path: string, value: string): Promise<void>;
  mkdir(path: string): Promise<void>;
  rename(from: string, to: string): Promise<void>;
  remove(path: string): Promise<void>;
  chmod(path: string): Promise<void>;
};

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
      const fs = await runtimeFs();
      try {
        await fs.remove(this.path);
      } catch (error) {
        if (!isNotFound(error)) throw error;
      }
    });
  }

  async #load(): Promise<AuthorizationClientState | undefined> {
    const fs = await runtimeFs();
    try {
      return JSON.parse(
        await fs.readTextFile(this.path),
      ) as AuthorizationClientState;
    } catch (error) {
      if (isNotFound(error)) return undefined;
      throw error;
    }
  }

  async #write(state: AuthorizationClientState): Promise<void> {
    const fs = await runtimeFs();
    const separator = Math.max(
      this.path.lastIndexOf("/"),
      this.path.lastIndexOf("\\"),
    );
    if (separator > 0) await fs.mkdir(this.path.slice(0, separator));
    const temporary = `${this.path}.${ulid()}.tmp`;
    await fs.writeTextFile(temporary, JSON.stringify(state, null, 2));
    await fs.chmod(temporary);
    await fs.rename(temporary, this.path);
  }

  #run<T>(operation: () => Promise<T>): Promise<T> {
    const result = this.#operation.then(operation);
    this.#operation = result.then(() => undefined, () => undefined);
    return result;
  }
}

async function runtimeFs(): Promise<RuntimeFs> {
  const deno = (globalThis as {
    Deno?: {
      readTextFile(path: string): Promise<string>;
      writeTextFile(
        path: string,
        value: string,
        options: { mode: number },
      ): Promise<void>;
      mkdir(path: string, options: { recursive: boolean }): Promise<void>;
      rename(from: string, to: string): Promise<void>;
      remove(path: string): Promise<void>;
      chmod(path: string, mode: number): Promise<void>;
    };
  }).Deno;
  if (deno) {
    return {
      readTextFile: (path) => deno.readTextFile(path),
      writeTextFile: (path, value) =>
        deno.writeTextFile(path, value, { mode: 0o600 }),
      mkdir: (path) => deno.mkdir(path, { recursive: true }),
      rename: (from, to) => deno.rename(from, to),
      remove: (path) => deno.remove(path),
      chmod: (path) => deno.chmod(path, 0o600),
    };
  }
  if (
    (globalThis as { process?: { versions?: { node?: string } } }).process
      ?.versions?.node
  ) {
    const fs = await import("node:fs/promises");
    return {
      readTextFile: (path) => fs.readFile(path, "utf8"),
      writeTextFile: (path, value) =>
        fs.writeFile(path, value, { mode: 0o600 }),
      mkdir: async (path) => {
        await fs.mkdir(path, { recursive: true });
      },
      rename: (from, to) => fs.rename(from, to),
      remove: (path) => fs.unlink(path),
      chmod: (path) => fs.chmod(path, 0o600),
    };
  }
  throw new Error(
    "file authorization context storage is unavailable in this runtime",
  );
}

function isNotFound(error: unknown): boolean {
  return error instanceof Error &&
    (error.name === "NotFound" ||
      ("code" in error && (error as { code?: string }).code === "ENOENT"));
}
