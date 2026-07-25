import { type Authenticator, jwtAuthenticator } from "@nats-io/nats-core";
import { fromSeed, Prefix } from "@nats-io/nkeys";
import { Codec } from "@nats-io/nkeys/lib/codec.js";
import { sha256 as sha256Sync } from "@noble/hashes/sha256";
import { ulid } from "ulid";

import {
  importEd25519PrivateKeyFromSeedBase64url,
  publicKeyBase64urlFromSeed,
} from "./keys.ts";
import { createProof } from "./proof.ts";
import {
  type SessionProofInputV1,
  type SessionProofV1,
  signSessionProofV1,
  signSessionProofV1Sync,
} from "./session_proof.ts";
import { correctedIatSeconds } from "./time.ts";
import {
  base64urlDecode,
  base64urlEncode,
  canonicalizeJsonValue,
  sha256,
  toArrayBuffer,
  utf8,
} from "./utils.ts";

export type NatsConnectOptions = {
  authenticator: Authenticator | Authenticator[];
  inboxPrefix: string;
};

export type TrellisAuth = {
  sessionKey: string; // base64url raw public key
  sessionNkey: string;
  sign: (data: Uint8Array) => Promise<Uint8Array>;
  currentIat: () => number;
  setServerClockOffsetMs: (clockOffsetMs: number) => void;

  oauthInitSig: (
    redirectTo: string,
    context?: unknown,
    provider?: string,
    contract?: Record<string, unknown>,
  ) => Promise<string>;
  bindFlowSig: (flowId: string) => Promise<string>;
  natsConnectSigForIat: (
    iat: number,
    contractDigest: string,
  ) => Promise<string>;

  createProof: (
    subject: string,
    payloadHash: Uint8Array,
    requestId?: string,
    iat?: number,
  ) => Promise<string>;
  signSessionProof: (input: SessionProofInputV1) => Promise<SessionProofV1>;
  natsConnectOptions: (
    opts: {
      sessionId: string;
      participantDigest: string;
      contextDigest: string | (() => string);
      jwt: string | (() => string);
    },
  ) => Promise<NatsConnectOptions>;
};

/**
 * Builds the canonical value signed for NATS runtime-auth tokens.
 */
export function buildNatsConnectSignaturePayload(
  iat: number,
  contractDigest: string,
): string {
  return `${iat}:${contractDigest}`;
}

export async function createAuth(
  opts: { sessionKeySeed: string },
): Promise<TrellisAuth> {
  const seed = base64urlDecode(opts.sessionKeySeed);
  const privateKey = await importEd25519PrivateKeyFromSeedBase64url(
    opts.sessionKeySeed,
  );
  const sessionKey = publicKeyBase64urlFromSeed(seed);
  const encodedSeed = Codec.encodeSeed(Prefix.User, seed);
  const sessionNkey = fromSeed(encodedSeed).getPublicKey();
  let serverClockOffsetMs = 0;

  const sign = async (data: Uint8Array): Promise<Uint8Array> => {
    const sig = await crypto.subtle.sign(
      { name: "Ed25519" },
      privateKey,
      toArrayBuffer(data),
    );
    return new Uint8Array(sig);
  };

  const signDomainHash = async (
    prefix: string,
    value: string,
  ): Promise<string> => {
    const digest = await sha256(utf8(`${prefix}:${value}`));
    const sigBytes = await sign(digest);
    return base64urlEncode(sigBytes);
  };

  const signOauthInit = async (
    redirectTo: string,
    context?: unknown,
    provider?: string,
    contract?: Record<string, unknown>,
  ): Promise<string> => {
    const canonicalContext = canonicalizeJsonValue(context ?? null);
    const payload = contract === undefined
      ? `${redirectTo}:${canonicalContext}`
      : `${redirectTo}:${provider ?? ""}:${
        canonicalizeJsonValue(contract)
      }:${canonicalContext}`;
    return await signDomainHash("oauth-init", payload);
  };

  const currentIat = (): number =>
    correctedIatSeconds(Date.now(), serverClockOffsetMs);

  return {
    sessionKey,
    sessionNkey,
    sign,
    currentIat,
    setServerClockOffsetMs: (clockOffsetMs) => {
      serverClockOffsetMs = clockOffsetMs;
    },
    oauthInitSig: signOauthInit,
    bindFlowSig: (flowId) => signDomainHash("bind-flow", flowId),
    natsConnectSigForIat: (iat, contractDigest) =>
      signDomainHash(
        "nats-connect",
        buildNatsConnectSignaturePayload(iat, contractDigest),
      ),
    createProof: (subject, payloadHash, requestId, iat) =>
      createProof(privateKey, {
        sessionKey,
        subject,
        payloadHash,
        iat: iat ?? currentIat(),
        requestId: requestId ?? ulid(),
      }),
    signSessionProof: (input) =>
      signSessionProofV1(input, privateKey, sessionKey),
    natsConnectOptions: (options) => {
      const sessionKeyId = base64urlEncode(
        sha256Sync(base64urlDecode(sessionKey)),
      );
      return Promise.resolve({
        authenticator: [
          jwtAuthenticator(options.jwt, encodedSeed),
          (nonce) => {
            if (!nonce) throw new Error("NATS server nonce is required");
            const issuedAt = Math.trunc(
              Date.now() + serverClockOffsetMs,
            );
            const requestId = ulid();
            const contextDigest = typeof options.contextDigest === "function"
              ? options.contextDigest()
              : options.contextDigest;
            const input = {
              purpose: "natsConnectContext" as const,
              requestId,
              issuedAt,
              sessionId: options.sessionId,
              sessionKeyId,
              sessionPublicKey: sessionKey,
              sessionNkey,
              participantDigest: options.participantDigest,
              contextDigest,
              nonce,
            };
            return {
              auth_token: JSON.stringify({
                format: "trellis.nats-connect-token.v1",
                requestId,
                issuedAt,
                sessionId: options.sessionId,
                participantDigest: options.participantDigest,
                contextDigest,
                proof: signSessionProofV1Sync(input, seed, sessionKey),
              }),
            };
          },
        ],
        inboxPrefix: `_INBOX.${options.sessionId}`,
      });
    },
  };
}
