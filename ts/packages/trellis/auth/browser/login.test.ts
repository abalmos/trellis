import { assertEquals, assertExists, assertRejects } from "@std/assert";

import { bindFlow, buildLoginUrl } from "./login.ts";
import { defineAppContract } from "../../contract_support/mod.ts";
import type { SessionKeyHandle } from "./session.ts";
import {
  sessionProofRequestDigest,
  verifySessionProof,
} from "../session_proof.ts";

const TEST_CONTRACT = defineAppContract(() => ({
  id: "demo.app@v1",
  displayName: "Demo app",
  description: "Demo app contract.",
}));

async function createHandle(): Promise<SessionKeyHandle> {
  const keyPair = await crypto.subtle.generateKey(
    { name: "Ed25519" },
    false,
    ["sign", "verify"],
  ) as CryptoKeyPair;
  const publicKeyRaw = new Uint8Array(
    await crypto.subtle.exportKey("raw", keyPair.publicKey),
  );
  const sessionKey = btoa(String.fromCharCode(...publicKeyRaw)).replace(
    /\+/g,
    "-",
  ).replace(/\//g, "_").replace(/=+$/g, "");
  return {
    seed: new Uint8Array(32),
    privateKey: keyPair.privateKey,
    publicKey: keyPair.publicKey,
    publicKeyRaw,
    sessionKey,
  };
}

Deno.test("buildLoginUrl sends the protocol-owned user auth proof", async () => {
  const originalFetch = globalThis.fetch;
  try {
    const handle = await createHandle();
    globalThis.fetch = (async (input, init) => {
      assertEquals(String(input), "http://localhost:3000/auth/requests");
      assertEquals(init?.method, "POST");
      const body = JSON.parse(String(init?.body));
      assertExists(body.requestId);
      assertEquals(Number.isSafeInteger(body.issuedAt), true);
      assertEquals(body.sessionPublicKey, handle.sessionKey);
      assertExists(body.sessionNkey);
      assertEquals(body.redirectTarget, "http://localhost:5173/profile");
      assertEquals(body.participantId, TEST_CONTRACT.CONTRACT_ID);
      assertEquals(body.participantArtifact, TEST_CONTRACT.PARTICIPANT);
      const requestDigest = await sessionProofRequestDigest(body);
      await verifySessionProof(
        {
          purpose: "userAuthRequest",
          requestId: body.requestId,
          issuedAt: body.issuedAt,
          sessionPublicKey: body.sessionPublicKey,
          sessionNkey: body.sessionNkey,
          participantId: body.participantId,
          participantDigest: body.participantArtifactDigest,
          redirectTarget: body.redirectTarget,
          requestDigest,
        },
        body.proof,
        handle.sessionKey,
        body.issuedAt,
        {
          maximumAgeMs: 1_000,
          maximumFutureSkewMs: 1_000,
        },
      );
      return new Response(JSON.stringify({
        state: "flow",
        flowId: "flow-1",
        portalUrl: "http://localhost:3000/login?flowId=flow-1",
      }));
    }) as typeof fetch;

    const url = await buildLoginUrl({
      authUrl: "http://localhost:3000",
      redirectTo: "http://localhost:5173/profile",
      handle,
      contract: TEST_CONTRACT,
    });

    assertEquals(
      url,
      "http://localhost:3000/login?flowId=flow-1",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("bindFlow posts a flow-scoped bind request", async () => {
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = (async (input, init) => {
      const url = String(input);
      assertEquals(url, "http://localhost:3000/auth/flow/flow-123/bind");
      assertEquals(init?.method, "POST");
      assertEquals(init?.headers, { "Content-Type": "application/json" });
      const body = JSON.parse(String(init?.body));
      assertEquals(body.sessionKey.length, 43);
      assertEquals(typeof body.sig, "string");
      assertEquals("authToken" in body, false);
      return new Response(
        JSON.stringify({
          status: "bound",
          inboxPrefix: "_INBOX.abc123",
          expires: "2026-01-01T00:00:00.000Z",
          sentinel: { jwt: "jwt", seed: "seed" },
          transports: {
            native: { natsServers: ["nats://localhost:4222"] },
            websocket: { natsServers: ["ws://localhost:8080"] },
          },
        }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    }) as typeof fetch;

    const response = await bindFlow(
      { authUrl: "http://localhost:3000" },
      await createHandle(),
      "flow-123",
    );
    assertEquals(response.status, "bound");
    if (response.status !== "bound") {
      throw new Error("expected bound response");
    }
    assertEquals(response.transports.websocket?.natsServers, [
      "ws://localhost:8080",
    ]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("bindFlow surfaces expired flow responses without a parse error", async () => {
  const originalFetch = globalThis.fetch;
  try {
    const handle = await createHandle();
    globalThis.fetch = (async () => {
      return new Response(
        JSON.stringify({ status: "expired" }),
        {
          status: 200,
          headers: { "Content-Type": "application/json" },
        },
      );
    }) as typeof fetch;

    await assertRejects(
      () =>
        bindFlow(
          { authUrl: "http://localhost:3000" },
          handle,
          "flow-expired",
        ),
      Error,
      "Bind failed: expired",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});
