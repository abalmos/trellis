import { type DirectStreamAPI, jetstreamManager } from "@nats-io/jetstream";
import type { NatsConnection } from "@nats-io/nats-core";

import type { AuthorizationRegistryBinding } from "./types.ts";

const REVOCATION_PREFIX = "revocation.";

/** Registry I/O counters observed since provider-cache start. */
export type AuthorizationRegistryIoCounters = {
  contextGets: number;
  revocationGets: number;
  revocationWatchInitializations: number;
  watchStarts: number;
};

export type RegistryEntry = {
  value: Uint8Array;
  revision: number;
  operation: string;
};

/** One exact revocation-key update observed after subscription flush. */
export type RegistryWatchEntry =
  | { operation: "put"; key: string; value: Uint8Array; revision: number }
  | { operation: "delete"; key: string; revision: number };

/** Connected NATS KV reader for authorization evidence. */
export class AuthorizationRegistryReader {
  readonly #nats: NatsConnection;
  readonly #direct: DirectStreamAPI;
  readonly #binding: AuthorizationRegistryBinding;
  #contextGets = 0;
  #revocationGets = 0;
  #revocationWatchInitializations = 0;
  #watchStarts = 0;

  private constructor(
    nats: NatsConnection,
    direct: DirectStreamAPI,
    binding: AuthorizationRegistryBinding,
  ) {
    this.#nats = nats;
    this.#direct = direct;
    this.#binding = binding;
  }

  /** Open the exact registry buckets from bootstrap-owned internal metadata. */
  static async open(
    nats: NatsConnection,
    binding: AuthorizationRegistryBinding,
  ): Promise<AuthorizationRegistryReader> {
    validateBinding(binding);
    const manager = await jetstreamManager(nats);
    return new AuthorizationRegistryReader(
      nats,
      manager.direct,
      binding,
    );
  }

  /** Return a copy of internal registry counters. */
  ioCounters(): AuthorizationRegistryIoCounters {
    return {
      contextGets: this.#contextGets,
      revocationGets: this.#revocationGets,
      revocationWatchInitializations: this.#revocationWatchInitializations,
      watchStarts: this.#watchStarts,
    };
  }

  /** Read one immutable context by its exact digest key. */
  async getContext(digest: string): Promise<RegistryEntry | null> {
    assertRegistryKey(digest, "authorization context digest");
    this.#contextGets += 1;
    return await this.#putOrNull(
      this.#binding.contextBucket,
      digest,
    );
  }

  /** Read one revocation marker by its exact context digest key. */
  async getRevocation(digest: string): Promise<RegistryEntry | null> {
    assertRegistryKey(digest, "authorization context digest");
    this.#revocationGets += 1;
    return await this.#putOrNull(
      this.#binding.contextBucket,
      `${REVOCATION_PREFIX}${digest}`,
    );
  }

  /** Subscribe only to the active context's revocation key. */
  async watchRevocation(contextDigest: string): Promise<{
    iterator: AsyncIterator<RegistryWatchEntry>;
    close: () => void;
  }> {
    assertRegistryKey(contextDigest, "authorization context digest");
    this.#watchStarts += 1;
    this.#revocationWatchInitializations += 1;
    const key = `${REVOCATION_PREFIX}${contextDigest}`;
    const subscription = this.#nats.subscribe(
      `$KV.${this.#binding.contextBucket}.${key}`,
    );
    try {
      await this.#nats.flush();
    } catch (error) {
      subscription.unsubscribe();
      throw error;
    }
    const reader = this;
    const entries = (async function* (): AsyncGenerator<RegistryWatchEntry> {
      try {
        for await (const message of subscription) {
          const operation = message.headers?.get("KV-Operation") || "PUT";
          if (operation !== "PUT") {
            yield { operation: "delete", key, revision: 0 };
            continue;
          }
          reader.#revocationGets += 1;
          const entry = await reader.#putOrNull(
            reader.#binding.contextBucket,
            key,
          );
          if (entry) {
            yield {
              operation: "put",
              key,
              value: entry.value,
              revision: entry.revision,
            };
          }
        }
      } finally {
        subscription.unsubscribe();
      }
    })();
    return {
      iterator: entries,
      close: () => subscription.unsubscribe(),
    };
  }

  async #putOrNull(bucket: string, key: string): Promise<RegistryEntry | null> {
    let entry: Awaited<ReturnType<DirectStreamAPI["getMessage"]>>;
    try {
      entry = await this.#direct.getMessage(`KV_${bucket}`, {
        last_by_subj: `$KV.${bucket}.${key}`,
      });
    } catch (error) {
      if (
        typeof error === "object" && error !== null && "code" in error &&
        error.code === 404
      ) return null;
      throw error;
    }
    if (!entry) return null;
    const operation = entry.header?.get("KV-Operation") || "PUT";
    if (operation !== "PUT") {
      throw new Error("authorization registry evidence disappeared");
    }
    return { value: entry.data, revision: entry.seq, operation };
  }
}

function validateBinding(binding: AuthorizationRegistryBinding): void {
  const entries = Object.entries(binding);
  if (
    entries.length !== 1 || !("contextBucket" in binding)
  ) {
    throw new Error("authorization registry binding is invalid");
  }
  for (const [name, value] of entries) {
    if (typeof value !== "string" || !value.trim()) {
      throw new Error(`authorization registry binding ${name} is empty`);
    }
  }
}

function assertRegistryKey(value: string, name: string): void {
  if (!isRegistryKey(value)) throw new Error(`${name} is invalid`);
}

function isRegistryKey(value: string): boolean {
  return value.length === 43 && /^[A-Za-z0-9_-]+$/.test(value);
}
