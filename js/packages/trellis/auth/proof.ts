import {
  base64urlDecode,
  base64urlEncode,
  sha256,
  toArrayBuffer,
  utf8,
} from "./utils.ts";
import { importEd25519PublicKeyFromBase64url } from "./keys.ts";
import { AsyncResult } from "@qlever-llc/result";

export type ProofParams = {
  sessionKey: string;
  subject: string;
  payloadHash: Uint8Array;
  iat: number;
  requestId: string;
};

export type EventProofParams = {
  sessionKey: string;
  subject: string;
  payloadHash: Uint8Array;
  eventId: string;
  eventTime: string;
};

const EVENT_PROOF_DOMAIN = utf8("trellis-event-proof-v1");

function appendLengthPrefixed(
  buf: Uint8Array,
  view: DataView,
  offset: number,
  value: Uint8Array,
): number {
  view.setUint32(offset, value.length);
  const valueOffset = offset + 4;
  buf.set(value, valueOffset);
  return valueOffset + value.length;
}

export function buildProofInput(
  sessionKey: string,
  subject: string,
  payloadHash: Uint8Array,
  iat: number,
  requestId: string,
): Uint8Array {
  const sessionKeyBytes = utf8(sessionKey);
  const subjectBytes = utf8(subject);
  const iatBytes = utf8(String(iat));
  const requestIdBytes = utf8(requestId);

  const buf = new Uint8Array(
    4 +
      sessionKeyBytes.length +
      4 +
      subjectBytes.length +
      4 +
      payloadHash.length +
      4 +
      iatBytes.length +
      4 +
      requestIdBytes.length,
  );
  const view = new DataView(buf.buffer);

  let offset = 0;
  offset = appendLengthPrefixed(buf, view, offset, sessionKeyBytes);
  offset = appendLengthPrefixed(buf, view, offset, subjectBytes);
  offset = appendLengthPrefixed(buf, view, offset, payloadHash);
  offset = appendLengthPrefixed(buf, view, offset, iatBytes);
  appendLengthPrefixed(buf, view, offset, requestIdBytes);

  return buf;
}

export function buildEventProofInput(
  sessionKey: string,
  subject: string,
  payloadHash: Uint8Array,
  eventId: string,
  eventTime: string,
): Uint8Array {
  const sessionKeyBytes = utf8(sessionKey);
  const subjectBytes = utf8(subject);
  const eventIdBytes = utf8(eventId);
  const eventTimeBytes = utf8(eventTime);

  const buf = new Uint8Array(
    4 + EVENT_PROOF_DOMAIN.length +
      4 + sessionKeyBytes.length +
      4 + subjectBytes.length +
      4 + payloadHash.length +
      4 + eventIdBytes.length +
      4 + eventTimeBytes.length,
  );
  const view = new DataView(buf.buffer);

  let offset = 0;
  offset = appendLengthPrefixed(buf, view, offset, EVENT_PROOF_DOMAIN);
  offset = appendLengthPrefixed(buf, view, offset, sessionKeyBytes);
  offset = appendLengthPrefixed(buf, view, offset, subjectBytes);
  offset = appendLengthPrefixed(buf, view, offset, payloadHash);
  offset = appendLengthPrefixed(buf, view, offset, eventIdBytes);
  appendLengthPrefixed(buf, view, offset, eventTimeBytes);

  return buf;
}

export async function createProof(
  privateKey: CryptoKey,
  params: ProofParams,
): Promise<string> {
  const input = buildProofInput(
    params.sessionKey,
    params.subject,
    params.payloadHash,
    params.iat,
    params.requestId,
  );
  const digest = await sha256(input);
  const sig = await crypto.subtle.sign(
    { name: "Ed25519" },
    privateKey,
    toArrayBuffer(digest),
  );
  return base64urlEncode(new Uint8Array(sig));
}

export async function createEventProof(
  privateKey: CryptoKey,
  params: EventProofParams,
): Promise<string> {
  const input = buildEventProofInput(
    params.sessionKey,
    params.subject,
    params.payloadHash,
    params.eventId,
    params.eventTime,
  );
  const digest = await sha256(input);
  const sig = await crypto.subtle.sign(
    { name: "Ed25519" },
    privateKey,
    toArrayBuffer(digest),
  );
  return base64urlEncode(new Uint8Array(sig));
}

export async function verifyProof(
  publicSessionKey: string,
  params: ProofParams,
  proofBase64url: string,
): Promise<boolean> {
  const result = await AsyncResult.try(async () => {
    const input = buildProofInput(
      params.sessionKey,
      params.subject,
      params.payloadHash,
      params.iat,
      params.requestId,
    );
    const digest = await sha256(input);
    const signature = base64urlDecode(proofBase64url);
    const pub = await importEd25519PublicKeyFromBase64url(publicSessionKey);
    return crypto.subtle.verify(
      { name: "Ed25519" },
      pub,
      toArrayBuffer(signature),
      toArrayBuffer(digest),
    );
  });
  return result.unwrapOr(false);
}

export async function verifyEventProof(
  publicSessionKey: string,
  params: EventProofParams,
  proofBase64url: string,
): Promise<boolean> {
  const result = await AsyncResult.try(async () => {
    const input = buildEventProofInput(
      params.sessionKey,
      params.subject,
      params.payloadHash,
      params.eventId,
      params.eventTime,
    );
    const digest = await sha256(input);
    const signature = base64urlDecode(proofBase64url);
    const pub = await importEd25519PublicKeyFromBase64url(publicSessionKey);
    return crypto.subtle.verify(
      { name: "Ed25519" },
      pub,
      toArrayBuffer(signature),
      toArrayBuffer(digest),
    );
  });
  return result.unwrapOr(false);
}
