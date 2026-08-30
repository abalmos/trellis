import {
  base64urlDecode,
  base64urlEncode,
  sha256,
  toArrayBuffer,
} from "../utils.ts";
import {
  importEd25519PrivateKeyFromSeedBase64url,
  importEd25519PublicKeyFromBase64url,
  publicKeyBase64urlFromSeed,
} from "../keys.ts";
import { createProof } from "../proof.ts";
import {
  deleteKeyPair,
  hasKeyPair,
  loadKeyPair,
  storeKeyPair,
  storeSessionId,
} from "./storage.ts";

export type SessionKeyHandle = {
  privateKey: CryptoKey;
  publicKey: CryptoKey;
  publicKeyRaw: Uint8Array;
  seed: Uint8Array;
  sessionId?: string;
  sessionKey: string;
  persistence?: SessionKeyPersistenceMode;
  expiresAt?: number;
  storageId?: string;
};

export type SessionKeyPersistenceMode = "temporary" | "remembered";

export type SessionKeyOptions = {
  /** Defaults to remembered IndexedDB storage. */
  persistence?: SessionKeyPersistenceMode;
  /** Expiry for remembered keys, as epoch milliseconds or a Date. */
  expiresAt?: number | Date;
  /** Relative expiry for remembered keys. Ignored when expiresAt is set. */
  ttlMs?: number;
  /** @internal Participant-and-origin storage scope. */
  storageScope?: string;
};

let temporarySessionKey: SessionKeyHandle | null = null;

function resolveExpiresAt(options: SessionKeyOptions): number | undefined {
  if (options.expiresAt instanceof Date) return options.expiresAt.getTime();
  if (typeof options.expiresAt === "number") return options.expiresAt;
  if (typeof options.ttlMs === "number") return Date.now() + options.ttlMs;
  return undefined;
}

export async function generateSessionKey(
  options: SessionKeyOptions = {},
): Promise<SessionKeyHandle> {
  const persistence = options.persistence ?? "remembered";
  const storageId = options.storageScope
    ? `trellis-session-key:${options.storageScope}`
    : undefined;
  const expiresAt = resolveExpiresAt(options);
  const seed = crypto.getRandomValues(new Uint8Array(32));
  const sessionKey = publicKeyBase64urlFromSeed(seed);
  const publicKeyRaw = base64urlDecode(sessionKey);
  const keyPair = {
    privateKey: await importEd25519PrivateKeyFromSeedBase64url(
      base64urlEncode(seed),
    ),
    publicKey: await importEd25519PublicKeyFromBase64url(sessionKey),
  };

  const handle: SessionKeyHandle = {
    privateKey: keyPair.privateKey,
    publicKey: keyPair.publicKey,
    publicKeyRaw,
    seed,
    sessionKey,
    persistence,
    ...(expiresAt === undefined ? {} : { expiresAt }),
    ...(storageId === undefined ? {} : { storageId }),
  };

  if (persistence === "temporary") {
    temporarySessionKey = handle;
  } else {
    await storeKeyPair(keyPair, publicKeyRaw, seed, { expiresAt }, storageId);
  }

  return handle;
}

export async function loadSessionKey(
  options: Pick<SessionKeyOptions, "persistence" | "storageScope"> = {},
): Promise<SessionKeyHandle | null> {
  const persistence = options.persistence ?? "remembered";
  if (persistence === "temporary") return temporarySessionKey;
  const storageId = options.storageScope
    ? `trellis-session-key:${options.storageScope}`
    : undefined;
  const stored = await loadKeyPair(storageId);
  if (!stored) return null;
  return {
    privateKey: stored.privateKey,
    publicKey: stored.publicKey,
    publicKeyRaw: stored.publicKeyRaw,
    seed: stored.seed,
    ...(stored.sessionId === undefined ? {} : { sessionId: stored.sessionId }),
    sessionKey: base64urlEncode(stored.publicKeyRaw),
    persistence: "remembered",
    ...(stored.expiresAt === undefined ? {} : { expiresAt: stored.expiresAt }),
    ...(storageId === undefined ? {} : { storageId }),
  };
}

/** Persists the session ID bound to the current browser key. */
export async function setSessionId(
  handle: SessionKeyHandle,
  sessionId: string,
): Promise<void> {
  handle.sessionId = sessionId;
  if (handle.persistence !== "temporary") {
    await storeSessionId(sessionId, handle.storageId);
  }
}

export async function getOrCreateSessionKey(
  options: SessionKeyOptions = {},
): Promise<SessionKeyHandle> {
  const existing = await loadSessionKey(options);
  if (existing) return existing;
  return await generateSessionKey(options);
}

export async function signBytes(
  handle: SessionKeyHandle,
  data: Uint8Array,
): Promise<Uint8Array> {
  const sig = await crypto.subtle.sign(
    { name: "Ed25519" },
    handle.privateKey,
    toArrayBuffer(data),
  );
  return new Uint8Array(sig);
}

export function getPublicSessionKey(handle: SessionKeyHandle): string {
  return handle.sessionKey;
}

export async function createRpcProof(
  handle: SessionKeyHandle,
  contextDigest: string,
  subject: string,
  reply: string,
  payload: Uint8Array,
  requestId: string,
  iat: number,
): Promise<string> {
  const payloadHash = await sha256(payload);
  return await createProof(handle.privateKey, {
    contextDigest,
    subject,
    reply,
    payloadHash,
    iat,
    requestId,
  });
}

export async function clearSessionKey(
  options: Pick<SessionKeyOptions, "persistence" | "storageScope"> = {},
): Promise<void> {
  const persistence = options.persistence;
  if (persistence === undefined || persistence === "temporary") {
    temporarySessionKey = null;
  }
  if (persistence === undefined || persistence === "remembered") {
    const storageId = options.storageScope
      ? `trellis-session-key:${options.storageScope}`
      : undefined;
    await deleteKeyPair(storageId);
  }
}

export async function hasSessionKey(
  options: Pick<SessionKeyOptions, "persistence" | "storageScope"> = {},
): Promise<boolean> {
  const persistence = options.persistence ?? "remembered";
  if (persistence === "temporary") return temporarySessionKey !== null;
  const storageId = options.storageScope
    ? `trellis-session-key:${options.storageScope}`
    : undefined;
  return await hasKeyPair(storageId);
}
