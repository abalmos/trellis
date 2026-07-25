import { assert, assertEquals, assertNotEquals } from "@std/assert";
import {
  base64urlDecode,
  base64urlEncode,
  buildEventProofInput,
  correctedIatSeconds,
  createAuth,
  sha256,
  toArrayBuffer,
  trellisIdFromOriginId,
  utf8,
  verifyEventProof,
  verifyProof,
  verifySessionProofV1,
} from "./mod.ts";

function authTokenFromAuthenticatorResult(value: unknown): string {
  if (!value || typeof value !== "object") {
    throw new Error(
      "Expected NATS authenticator to return an auth token payload",
    );
  }

  const record = value as { auth_token?: unknown };
  if (typeof record.auth_token !== "string") {
    throw new Error("Expected NATS authenticator to return auth_token");
  }

  return record.auth_token;
}

Deno.test("createAuth derives sessionKey from 32-byte seed", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });

  const pk = base64urlDecode(auth.sessionKey);
  assertEquals(pk.length, 32);
});

Deno.test("oauthInitSig signs the auth-start payload including provider, contract, and context", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });

  const redirectTo = "https://example.com/app";
  const sig = await auth.oauthInitSig(
    redirectTo,
    { subtitle: "Welcome back" },
    "github",
    { id: "trellis.console@v1", origin: "https://console.example.com" },
  );

  const digest = await sha256(
    utf8(
      'oauth-init:https://example.com/app:github:{"id":"trellis.console@v1","origin":"https://console.example.com"}:{"subtitle":"Welcome back"}',
    ),
  );
  const pub = await crypto.subtle.importKey(
    "raw",
    toArrayBuffer(base64urlDecode(auth.sessionKey)),
    { name: "Ed25519" },
    true,
    ["verify"],
  );

  const ok = await crypto.subtle.verify(
    { name: "Ed25519" },
    pub,
    toArrayBuffer(base64urlDecode(sig)),
    toArrayBuffer(digest),
  );
  assert(ok);
});

Deno.test("proof creation and verification match ADR format", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });

  const subject = "rpc.v1.User.Find";
  const payloadHash = await sha256(
    utf8(JSON.stringify({ userId: { origin: "github", id: "1" } })),
  );
  const iat = 1_700_000_000;
  const requestId = "req_123";
  const proof = await auth.createProof(subject, payloadHash, requestId, iat);

  const ok = await verifyProof(
    auth.sessionKey,
    { sessionKey: auth.sessionKey, subject, payloadHash, iat, requestId },
    proof,
  );
  assert(ok);

  const bad = await verifyProof(
    auth.sessionKey,
    {
      sessionKey: auth.sessionKey,
      subject,
      payloadHash: await sha256(utf8("different")),
      iat,
      requestId,
    },
    proof,
  );
  assertEquals(bad, false);
});

Deno.test("event proof uses event id and event time domain", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });
  const payloadHash = await sha256(utf8(JSON.stringify({ value: "one" })));
  const eventId = "evt_123";
  const eventTime = "2026-04-26T00:00:00.000Z";
  const digest = await sha256(
    buildEventProofInput(
      auth.sessionKey,
      "events.v1.Thing.Changed.one",
      payloadHash,
      eventId,
      eventTime,
    ),
  );
  const proof = base64urlEncode(await auth.sign(digest));

  assert(
    await verifyEventProof(
      auth.sessionKey,
      {
        sessionKey: auth.sessionKey,
        subject: "events.v1.Thing.Changed.one",
        payloadHash,
        eventId,
        eventTime,
      },
      proof,
    ),
  );
  assertEquals(
    await verifyEventProof(
      auth.sessionKey,
      {
        sessionKey: auth.sessionKey,
        subject: "events.v1.Thing.Changed.one",
        payloadHash,
        eventId: "evt_other",
        eventTime,
      },
      proof,
    ),
    false,
  );
});

Deno.test("trellisIdFromOriginId is stable and 22 chars", async () => {
  const id1 = await trellisIdFromOriginId("github", "123");
  const id2 = await trellisIdFromOriginId("github", "123");
  const id3 = await trellisIdFromOriginId("github", "124");

  assertEquals(id1.length, 22);
  assertEquals(id1, id2);
  assert(id1 !== id3);
});

Deno.test("natsConnectOptions signs nonce-bound reconnect proofs", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });
  const participantDigest = base64urlEncode(await sha256(utf8("participant")));
  const contextDigest = base64urlEncode(await sha256(utf8("context")));
  const originalNow = Date.now;

  try {
    let nowMs = 1_700_000_000_000;
    Date.now = () => nowMs;

    const options = await auth.natsConnectOptions({
      sessionId: "ses_test",
      participantDigest,
      contextDigest,
      jwt: "deny-all-jwt",
    });
    const authenticators = Array.isArray(options.authenticator)
      ? options.authenticator
      : [options.authenticator];
    const jwt = authenticators[0]("nonce-a") as {
      jwt: string;
      nkey: string;
    };
    const firstToken = JSON.parse(
      authTokenFromAuthenticatorResult(authenticators[1]("nonce-a")),
    ) as {
      format: string;
      requestId: string;
      issuedAt: number;
      sessionId: string;
      participantDigest: string;
      contextDigest: string;
      proof: { format: "trellis.session-proof.v1"; signature: string };
    };

    nowMs += 31_000;

    const secondToken = JSON.parse(
      authTokenFromAuthenticatorResult(authenticators[1]("nonce-b")),
    ) as typeof firstToken;

    assertEquals(options.inboxPrefix, "_INBOX.ses_test");
    assertEquals(jwt.jwt, "deny-all-jwt");
    assertEquals(firstToken.format, "trellis.nats-connect-token.v1");
    assertEquals(firstToken.sessionId, "ses_test");
    assertEquals(firstToken.participantDigest, participantDigest);
    assertEquals(firstToken.contextDigest, contextDigest);
    assertEquals(secondToken.issuedAt - firstToken.issuedAt, 31_000);
    assertNotEquals(firstToken.proof.signature, secondToken.proof.signature);
    const sessionKeyId = base64urlEncode(
      await sha256(base64urlDecode(auth.sessionKey)),
    );
    await verifySessionProofV1(
      {
        purpose: "natsConnectContext",
        requestId: firstToken.requestId,
        issuedAt: firstToken.issuedAt,
        sessionId: firstToken.sessionId,
        sessionKeyId,
        sessionPublicKey: auth.sessionKey,
        sessionNkey: jwt.nkey,
        participantDigest: firstToken.participantDigest,
        contextDigest: firstToken.contextDigest,
        nonce: "nonce-a",
      },
      firstToken.proof,
      auth.sessionKey,
      firstToken.issuedAt,
    );
  } finally {
    Date.now = originalNow;
  }
});

Deno.test("createAuth applies server clock offsets to current iat and reconnect auth tokens", async () => {
  const seed = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
  const auth = await createAuth({ sessionKeySeed: seed });
  const participantDigest = base64urlEncode(await sha256(utf8("participant")));
  const contextDigest = base64urlEncode(await sha256(utf8("context")));
  const originalNow = Date.now;

  try {
    Date.now = () => 1_700_000_000_250;
    auth.setServerClockOffsetMs(900);

    assertEquals(auth.currentIat(), correctedIatSeconds(Date.now(), 900));

    const options = await auth.natsConnectOptions({
      sessionId: "ses_test",
      participantDigest,
      contextDigest,
      jwt: "deny-all-jwt",
    });
    const authenticators = Array.isArray(options.authenticator)
      ? options.authenticator
      : [options.authenticator];
    const token = JSON.parse(
      authTokenFromAuthenticatorResult(authenticators[1]("nonce")),
    ) as {
      issuedAt: number;
    };

    assertEquals(token.issuedAt, 1_700_000_001_150);
  } finally {
    Date.now = originalNow;
  }
});
