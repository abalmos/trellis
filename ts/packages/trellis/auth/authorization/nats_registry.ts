import { type DirectStreamAPI, jetstreamManager } from "@nats-io/jetstream";
import { type KV, Kvm, type KvWatchEntry } from "@nats-io/kv";
import type { NatsConnection } from "@nats-io/nats-core";

import type { AuthorizationRegistryBinding } from "./types.ts";

const MAX_REGISTRY_KEY_BYTES = 256;
const MANIFEST_CURRENT_KEY = "manifest.current";
const MANIFEST_PREFIX = "manifest.";
const REVOCATION_PREFIX = "revocation.";

/** Parsed manifest pointer and its exact NATS KV revision. */
export type ManifestPointer = {
  generation: number;
  digest: string;
  revision: number;
};

/** Registry I/O counters observed since provider-cache start. */
export type AuthorizationRegistryIoCounters = {
  contextGets: number;
  trustGets: number;
  revocationWatchInitializations: number;
  watchStarts: number;
};

type RegistryEntry = {
  value: Uint8Array;
  revision: number;
  operation: string;
};

/** Connected NATS KV reader for authorization evidence. */
export class AuthorizationRegistryReader {
  readonly #trust: KV;
  readonly #contexts: KV;
  readonly #direct: DirectStreamAPI;
  readonly #binding: AuthorizationRegistryBinding;
  #contextGets = 0;
  #trustGets = 0;
  #revocationWatchInitializations = 0;
  #watchStarts = 0;

  private constructor(
    trust: KV,
    contexts: KV,
    direct: DirectStreamAPI,
    binding: AuthorizationRegistryBinding,
  ) {
    this.#trust = trust;
    this.#contexts = contexts;
    this.#direct = direct;
    this.#binding = binding;
  }

  /** Open the exact registry buckets from bootstrap-owned internal metadata. */
  static async open(
    nats: NatsConnection,
    binding: AuthorizationRegistryBinding,
  ): Promise<AuthorizationRegistryReader> {
    validateBinding(binding);
    const kvm = new Kvm(nats);
    const [trust, contexts] = await Promise.all([
      kvm.open(binding.trustBucket),
      kvm.open(binding.contextBucket),
    ]);
    const manager = await jetstreamManager(nats);
    return new AuthorizationRegistryReader(
      trust,
      contexts,
      manager.direct,
      binding,
    );
  }

  /** Return a copy of internal registry counters. */
  ioCounters(): AuthorizationRegistryIoCounters {
    return {
      contextGets: this.#contextGets,
      trustGets: this.#trustGets,
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

  /** Read one issuer manifest by its exact generation key. */
  async getManifest(generation: number): Promise<RegistryEntry | null> {
    if (!Number.isSafeInteger(generation) || generation <= 0) {
      throw new Error("authorization manifest generation is invalid");
    }
    this.#trustGets += 1;
    return await this.#putOrNull(
      this.#binding.trustBucket,
      `${MANIFEST_PREFIX}${generation}`,
    );
  }

  /** Read the mutable current-manifest pointer and its exact revision. */
  async getManifestCurrent(): Promise<ManifestPointer | null> {
    this.#trustGets += 1;
    const entry = await this.#putOrNull(
      this.#binding.trustBucket,
      MANIFEST_CURRENT_KEY,
    );
    if (entry?.operation !== "PUT") return null;
    const pointer = parseManifestPointer(entry.value);
    return { ...pointer, revision: entry.revision };
  }

  /** Subscribe to the mutable current-manifest pointer. */
  async watchManifestCurrent(): Promise<AsyncIterator<KvWatchEntry>> {
    this.#watchStarts += 1;
    const watcher = await this.#trust.watch({
      key: MANIFEST_CURRENT_KEY,
      include: "updates",
    });
    return watcher[Symbol.asyncIterator]();
  }

  /** Subscribe to all revocation updates, including delete tombstones. */
  async watchRevocations(): Promise<{
    iterator: AsyncIterator<KvWatchEntry>;
    initialPending: number;
  }> {
    this.#watchStarts += 1;
    this.#revocationWatchInitializations += 1;
    const watcher = await this.#contexts.watch({
      key: `${REVOCATION_PREFIX}>`,
    });
    return {
      iterator: watcher[Symbol.asyncIterator](),
      initialPending: watcher.getPending(),
    };
  }

  async #putOrNull(bucket: string, key: string): Promise<RegistryEntry | null> {
    const entry = await this.#direct.getMessage(`KV_${bucket}`, {
      last_by_subj: `$KV.${bucket}.${key}`,
    });
    if (!entry) return null;
    const operation = entry.header?.get("KV-Operation") || "PUT";
    return operation === "PUT"
      ? { value: entry.data, revision: entry.seq, operation }
      : null;
  }
}

/** Parse and validate the signed manifest pointer without verifying signatures. */
export function parseManifestPointer(value: Uint8Array): {
  generation: number;
  digest: string;
} {
  const parsed: unknown = JSON.parse(new TextDecoder().decode(value));
  if (!isRecord(parsed)) {
    throw new Error("current issuer manifest pointer is invalid");
  }
  const keys = Object.keys(parsed).sort();
  if (
    keys.length !== 2 ||
    keys[0] !== "digest" ||
    keys[1] !== "generation" ||
    typeof parsed.generation !== "number" ||
    !Number.isSafeInteger(parsed.generation) ||
    parsed.generation <= 0 ||
    typeof parsed.digest !== "string" ||
    !isRegistryKey(parsed.digest)
  ) {
    throw new Error("current issuer manifest pointer is invalid");
  }
  return { generation: parsed.generation, digest: parsed.digest };
}

/** Map a KV watch entry to a PUT or delete operation. */
export function registryWatchEntry(entry: KvWatchEntry):
  | { operation: "put"; key: string; value: Uint8Array; revision: number }
  | { operation: "delete"; key: string; revision: number } {
  if (entry.operation === "PUT") {
    return {
      operation: "put",
      key: entry.key,
      value: new Uint8Array(entry.value),
      revision: entry.revision,
    };
  }
  return { operation: "delete", key: entry.key, revision: entry.revision };
}

function validateBinding(binding: AuthorizationRegistryBinding): void {
  const entries = Object.entries(binding);
  if (
    entries.length !== 2 || !("trustBucket" in binding) ||
    !("contextBucket" in binding)
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
  return value.length > 0 && value.length <= MAX_REGISTRY_KEY_BYTES &&
    /^[A-Za-z0-9_-]+$/.test(value);
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
