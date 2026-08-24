import type { StaticDecode } from "typebox";
import { Type } from "typebox";
import { Value } from "typebox/value";
import type { BaseError } from "@qlever-llc/result";
import type { AsyncResult } from "@qlever-llc/result";
import type { OperationRef } from "../operations.ts";

import {
  importEd25519PrivateKeyFromSeedBase64url,
  publicKeyBase64urlFromPrivateKey,
} from "./keys.ts";
import {
  type AuthDeviceUserAuthoritiesListResponseSchema,
  type AuthDeviceUserAuthoritiesListSchema,
  type AuthDeviceUserAuthoritiesRevokeResponseSchema,
  type AuthDeviceUserAuthoritiesRevokeSchema,
  type AuthResolveDeviceUserAuthoritiesProgressSchema,
  type AuthResolveDeviceUserAuthoritiesResponseSchema,
  type AuthResolveDeviceUserAuthoritiesSchema,
} from "./protocol.ts";
import {
  base64urlDecode,
  base64urlEncode,
  sha256,
  toArrayBuffer,
  utf8,
} from "./utils.ts";
import { createAuth } from "./session_auth.ts";
import {
  SESSION_PROOF_FORMAT_V1,
  sessionProofRequestDigest,
} from "./session_proof.ts";
import { ulid } from "ulid";

const DEVICE_IDENTITY_HKDF_INFO = "trellis/device-identity/v1";
const DEVICE_ACTIVATION_HKDF_INFO = "trellis/device-activate/v1";
const DEVICE_QR_MAC_DOMAIN = "trellis-device-qr/v1";
const DEVICE_CONFIRMATION_DOMAIN = "trellis-device-confirm/v1";
const CROCKFORD_ALPHABET = "0123456789ABCDEFGHJKMNPQRSTVWXYZ";
const DEFAULT_WAIT_POLL_INTERVAL_MS = 3_000;

export const DeviceActivationPayloadSchema = Type.Object({
  v: Type.Literal(1),
  publicIdentityKey: Type.String({ minLength: 1 }),
  nonce: Type.String({ minLength: 1 }),
  qrMac: Type.String({ minLength: 1 }),
});

export type DeviceActivationPayload = StaticDecode<
  typeof DeviceActivationPayloadSchema
>;
export type AuthResolveDeviceUserAuthoritiesInput = StaticDecode<
  typeof AuthResolveDeviceUserAuthoritiesSchema
>;
export type AuthResolveDeviceUserAuthoritiesProgress = StaticDecode<
  typeof AuthResolveDeviceUserAuthoritiesProgressSchema
>;
export type AuthResolveDeviceUserAuthoritiesOutput = StaticDecode<
  typeof AuthResolveDeviceUserAuthoritiesResponseSchema
>;
export type AuthDeviceUserAuthoritiesListInput = StaticDecode<
  typeof AuthDeviceUserAuthoritiesListSchema
>;
export type AuthDeviceUserAuthoritiesListOutput = StaticDecode<
  typeof AuthDeviceUserAuthoritiesListResponseSchema
>;
export type AuthDeviceUserAuthoritiesRevokeInput = StaticDecode<
  typeof AuthDeviceUserAuthoritiesRevokeSchema
>;
export type AuthDeviceUserAuthoritiesRevokeResponse = StaticDecode<
  typeof AuthDeviceUserAuthoritiesRevokeResponseSchema
>;
export type DeviceIdentity = {
  identitySeed: Uint8Array;
  identitySeedBase64url: string;
  publicIdentityKey: string;
  activationKey: Uint8Array;
  activationKeyBase64url: string;
};

type AuthResolveDeviceUserAuthoritiesOperationShape = {
  subject: string;
  input: typeof AuthResolveDeviceUserAuthoritiesSchema;
  progress: typeof AuthResolveDeviceUserAuthoritiesProgressSchema;
  output: typeof AuthResolveDeviceUserAuthoritiesResponseSchema;
};

export type AuthResolveDeviceUserAuthoritiesOperation = OperationRef<
  AuthResolveDeviceUserAuthoritiesOperationShape,
  AuthResolveDeviceUserAuthoritiesProgress,
  AuthResolveDeviceUserAuthoritiesOutput
>;

export type DeviceActivationTransport = {
  authDeviceUserAuthoritiesResolve(
    input: AuthResolveDeviceUserAuthoritiesInput,
  ): {
    start(): AsyncResult<AuthResolveDeviceUserAuthoritiesOperation, BaseError>;
  };
  authDeviceUserAuthoritiesList(
    input: AuthDeviceUserAuthoritiesListInput,
  ): AsyncResult<AuthDeviceUserAuthoritiesListOutput, BaseError>;
  authDeviceUserAuthoritiesRevoke(
    input: AuthDeviceUserAuthoritiesRevokeInput,
  ): AsyncResult<AuthDeviceUserAuthoritiesRevokeResponse, BaseError>;
};

function concatBytes(parts: Uint8Array[]): Uint8Array {
  const size = parts.reduce((total, part) => total + part.length, 0);
  const bytes = new Uint8Array(size);
  let offset = 0;
  for (const part of parts) {
    bytes.set(part, offset);
    offset += part.length;
  }
  return bytes;
}

function normalizeSecretBytes(
  value: Uint8Array | string,
  name: string,
): Uint8Array {
  if (typeof value === "string") {
    const decoded = base64urlDecode(value);
    if (decoded.length === 0) throw new Error(`${name} must not be empty`);
    return decoded;
  }
  if (value.length === 0) throw new Error(`${name} must not be empty`);
  return value;
}

async function hkdfSha256(
  inputKeyingMaterial: Uint8Array,
  info: string,
  length: number,
): Promise<Uint8Array> {
  const key = await crypto.subtle.importKey(
    "raw",
    toArrayBuffer(inputKeyingMaterial),
    "HKDF",
    false,
    ["deriveBits"],
  );
  const derivedBits = await crypto.subtle.deriveBits(
    {
      name: "HKDF",
      hash: "SHA-256",
      salt: toArrayBuffer(new Uint8Array(0)),
      info: toArrayBuffer(utf8(info)),
    },
    key,
    length * 8,
  );
  return new Uint8Array(derivedBits);
}

async function hmacSha256(
  keyBytes: Uint8Array,
  data: Uint8Array,
): Promise<Uint8Array> {
  const key = await crypto.subtle.importKey(
    "raw",
    toArrayBuffer(keyBytes),
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return new Uint8Array(
    await crypto.subtle.sign("HMAC", key, toArrayBuffer(data)),
  );
}

function crockfordEncode(bytes: Uint8Array): string {
  let value = 0;
  let bits = 0;
  let output = "";
  for (const byte of bytes) {
    value = (value << 8) | byte;
    bits += 8;
    while (bits >= 5) {
      bits -= 5;
      output += CROCKFORD_ALPHABET[(value >>> bits) & 31] ?? "0";
    }
  }
  if (bits > 0) {
    output += CROCKFORD_ALPHABET[(value << (5 - bits)) & 31] ?? "0";
  }
  return output;
}

function normalizeCrockford(value: string): string {
  return value.trim().toUpperCase().replace(/O/g, "0").replace(/[IL]/g, "1");
}

async function sleep(ms: number, signal?: AbortSignal): Promise<void> {
  if (signal?.aborted) {
    throw signal.reason ?? new DOMException("Aborted", "AbortError");
  }
  await new Promise<void>((resolve, reject) => {
    const timer = setTimeout(() => {
      signal?.removeEventListener("abort", onAbort);
      resolve();
    }, ms);
    function onAbort() {
      clearTimeout(timer);
      reject(signal?.reason ?? new DOMException("Aborted", "AbortError"));
    }
    signal?.addEventListener("abort", onAbort, { once: true });
  });
}

async function responseErrorDetail(response: Response): Promise<string | null> {
  const text = await response.text();
  if (!text) return null;

  try {
    const parsed = JSON.parse(text) as Record<string, unknown>;
    if (typeof parsed.reason === "string" && parsed.reason.length > 0) {
      return parsed.reason;
    }
    if (typeof parsed.message === "string" && parsed.message.length > 0) {
      return parsed.message;
    }
  } catch {
    // Fall through to raw text below.
  }

  return text;
}

export async function deriveDeviceIdentity(
  deviceRootSecret: Uint8Array,
): Promise<DeviceIdentity> {
  if (deviceRootSecret.length !== 32) {
    throw new Error(
      `Invalid device root secret length: ${deviceRootSecret.length} (expected 32)`,
    );
  }
  const identitySeed = await hkdfSha256(
    deviceRootSecret,
    DEVICE_IDENTITY_HKDF_INFO,
    32,
  );
  const activationKey = await hkdfSha256(
    deviceRootSecret,
    DEVICE_ACTIVATION_HKDF_INFO,
    32,
  );
  const identitySeedBase64url = base64urlEncode(identitySeed);
  const identityPrivateKey = await importEd25519PrivateKeyFromSeedBase64url(
    identitySeedBase64url,
  );
  const publicIdentityKey = await publicKeyBase64urlFromPrivateKey(
    identityPrivateKey,
  );
  return {
    identitySeed,
    identitySeedBase64url,
    publicIdentityKey,
    activationKey,
    activationKeyBase64url: base64urlEncode(activationKey),
  };
}

export async function deriveDeviceQrMac(input: {
  activationKey: Uint8Array | string;
  publicIdentityKey: string;
  nonce: string;
}): Promise<string> {
  const activationKey = normalizeSecretBytes(
    input.activationKey,
    "activationKey",
  );
  const mac = await hmacSha256(
    activationKey,
    concatBytes([
      utf8(DEVICE_QR_MAC_DOMAIN),
      utf8(input.publicIdentityKey),
      utf8(input.nonce),
    ]),
  );
  return base64urlEncode(mac.slice(0, 8));
}

export async function buildDeviceActivationPayload(input: {
  activationKey: Uint8Array | string;
  publicIdentityKey: string;
  nonce: string;
}): Promise<DeviceActivationPayload> {
  const qrMac = await deriveDeviceQrMac(input);
  return {
    v: 1,
    publicIdentityKey: input.publicIdentityKey,
    nonce: input.nonce,
    qrMac,
  };
}

export function encodeDeviceActivationPayload(
  payload: DeviceActivationPayload,
): string {
  return base64urlEncode(utf8(JSON.stringify(payload)));
}

export function parseDeviceActivationPayload(
  value: string,
): DeviceActivationPayload {
  const decoded = new TextDecoder().decode(base64urlDecode(value));
  const parsed = JSON.parse(decoded);
  if (!Value.Check(DeviceActivationPayloadSchema, parsed)) {
    throw new Error("Invalid device activation payload");
  }
  return parsed;
}

export async function deriveDeviceConfirmationCode(input: {
  activationKey: Uint8Array | string;
  publicIdentityKey: string;
  nonce: string;
}): Promise<string> {
  const activationKey = normalizeSecretBytes(
    input.activationKey,
    "activationKey",
  );
  const mac = await hmacSha256(
    activationKey,
    concatBytes([
      utf8(DEVICE_CONFIRMATION_DOMAIN),
      utf8(input.publicIdentityKey),
      utf8(input.nonce),
    ]),
  );
  return crockfordEncode(mac.slice(0, 5)).slice(0, 8);
}

export async function verifyDeviceConfirmationCode(input: {
  activationKey: Uint8Array | string;
  publicIdentityKey: string;
  nonce: string;
  confirmationCode: string;
}): Promise<boolean> {
  const expected = await deriveDeviceConfirmationCode(input);
  return normalizeCrockford(expected) ===
    normalizeCrockford(input.confirmationCode);
}

/**
 * Retry the proof-bound device bootstrap operation until activation completes.
 *
 * Each attempt re-signs the same bootstrap request; the server returns
 * `activation_pending` with a suggested `retryAfterMs` until the device is
 * approved, then `ready`.
 */
export async function waitForDeviceActivation(args: {
  trellisUrl: string;
  publicIdentityKey: string;
  identitySeed: Uint8Array | string;
  activationKey: Uint8Array | string;
  deploymentId: string;
  instanceId: string;
  principalId: string;
  participantId: string;
  participantArtifactDigest: string;
  participantNeedsDigest: string;
  nonce?: string;
  signal?: AbortSignal;
  pollIntervalMs?: number;
}): Promise<void> {
  const pollIntervalMs = args.pollIntervalMs ?? DEFAULT_WAIT_POLL_INTERVAL_MS;
  const nonce = args.nonce ?? ulid();
  const identitySeed = normalizeSecretBytes(args.identitySeed, "identitySeed");
  const challengeDigest = base64urlEncode(await sha256(utf8(nonce)));
  const confirmationCode = await deriveDeviceConfirmationCode({
    activationKey: args.activationKey,
    publicIdentityKey: args.publicIdentityKey,
    nonce,
  });
  let reviewId: string | undefined;
  let reviewDeadline: number | undefined;
  while (true) {
    if (reviewDeadline !== undefined && performance.now() >= reviewDeadline) {
      throw new Error("device activation review expired");
    }
    const requestId = ulid();
    const issuedAt = Date.now();
    const identityAuth = await createAuth({
      sessionKeySeed: base64urlEncode(identitySeed),
    });
    const sessionAuth = await createAuth({
      sessionKeySeed: base64urlEncode(
        crypto.getRandomValues(new Uint8Array(32)),
      ),
    });
    const deviceIdentityKeyId = base64urlEncode(
      await sha256(base64urlDecode(identityAuth.sessionKey)),
    );
    const unsigned = {
      requestId,
      issuedAt,
      deploymentId: args.deploymentId,
      instanceId: args.instanceId,
      deviceIdentityKeyId,
      principalId: args.principalId,
      identityPublicKey: identityAuth.sessionKey,
      provisioningSecret: null,
      expectedSecretVersion: null,
      newSessionPublicKey: sessionAuth.sessionKey,
      newSessionNkey: sessionAuth.sessionNkey,
      participantId: args.participantId,
      participantArtifactDigest: args.participantArtifactDigest,
      participantNeedsDigest: args.participantNeedsDigest,
      challengeDigest,
      confirmationCode,
      proof: { format: SESSION_PROOF_FORMAT_V1, signature: "" },
    };
    const requestDigest = await sessionProofRequestDigest(unsigned);
    let response: Response;
    try {
      response = await fetch(
        new URL("/bootstrap/device", args.trellisUrl),
        {
          method: "POST",
          signal: args.signal,
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            ...unsigned,
            proof: await identityAuth.signSessionProof({
              purpose: "deviceBootstrap",
              requestId,
              issuedAt,
              deploymentId: args.deploymentId,
              instanceId: args.instanceId,
              deviceIdentityKeyId,
              newSessionPublicKey: sessionAuth.sessionKey,
              newSessionNkey: sessionAuth.sessionNkey,
              participantId: args.participantId,
              participantDigest: args.participantArtifactDigest,
              challengeDigest,
              requestDigest,
            }),
          }),
        },
      );
    } catch (error) {
      if (args.signal?.aborted) {
        throw error;
      }
      await sleep(pollIntervalMs, args.signal);
      continue;
    }
    if (!response.ok) {
      const detail = await responseErrorDetail(response);
      throw new Error(
        detail
          ? `device activation bootstrap failed: ${response.status} ${detail}`
          : `device activation bootstrap failed: ${response.status}`,
      );
    }
    const body: unknown = await response.json();
    if (typeof body !== "object" || body === null) {
      throw new Error("Invalid device activation bootstrap response");
    }
    const state = Reflect.get(body, "state") as unknown;
    if (state === "ready") {
      return;
    }
    const activation = Reflect.get(body, "activation") as
      | Record<
        string,
        unknown
      >
      | null;
    const currentReviewId = activation?.reviewId;
    const serverNow = Reflect.get(body, "serverNow");
    const expiresAt = activation?.expiresAt;
    if (
      typeof currentReviewId !== "string" ||
      typeof serverNow !== "number" ||
      typeof expiresAt !== "number"
    ) {
      throw new Error("Invalid pending device activation response");
    }
    if (reviewId !== undefined && currentReviewId !== reviewId) {
      throw new Error("device activation review expired");
    }
    if (reviewId === undefined) {
      reviewId = currentReviewId;
      reviewDeadline = performance.now() + Math.max(0, expiresAt - serverNow);
    }
    const retryAfterMs = typeof activation?.retryAfterMs === "number"
      ? activation.retryAfterMs
      : pollIntervalMs;
    await sleep(
      Math.min(
        Math.max(pollIntervalMs, retryAfterMs),
        Math.max(0, (reviewDeadline ?? performance.now()) - performance.now()),
      ),
      args.signal,
    );
  }
}

export function createDeviceActivationClient(
  client: DeviceActivationTransport,
) {
  return {
    resolveDeviceUserAuthorities(input: AuthResolveDeviceUserAuthoritiesInput) {
      return client.authDeviceUserAuthoritiesResolve(input).start().orThrow();
    },
    listDeviceActivations(input: AuthDeviceUserAuthoritiesListInput) {
      return client.authDeviceUserAuthoritiesList(input).orThrow();
    },
    revokeDeviceActivation(input: AuthDeviceUserAuthoritiesRevokeInput) {
      return client.authDeviceUserAuthoritiesRevoke(input).orThrow();
    },
  };
}
