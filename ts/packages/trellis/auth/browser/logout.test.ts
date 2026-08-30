import { AsyncResult } from "@qlever-llc/result";
import { assert, assertEquals, assertRejects } from "@std/assert";

import type { AuthSessionsLogoutOutput } from "../../internal_sdk/generated/auth/mod.ts";
import { completeSessionLogout, logoutSession } from "./logout.ts";
import {
  clearSessionKey,
  generateSessionKey,
  hasSessionKey,
} from "./session.ts";

function logoutOutput(): AuthSessionsLogoutOutput {
  return {
    kickedConnections: 1,
    session: {
      createdAt: 1,
      expiresAt: null,
      inboxPrefix: "_INBOX.ses_test",
      lastSeenAt: 2,
      participantArtifactDigest: "participant",
      participantId: "app.test@v1",
      participantKind: "app",
      participantNeedsDigest: "needs",
      principalId: "usr_test",
      principalKind: "user",
      revokedAt: 3,
      sessionId: "ses_test",
      sessionKeyId: "key_test",
      sessionPublicKey: "public_test",
      state: "revoked",
      version: 2,
    },
  };
}

async function assertRedirects(action: () => Promise<never>): Promise<void> {
  await assertRejects(action, Error, "Redirecting after logout");
}

Deno.test("disconnected logout clears locally without an HTTP request", async () => {
  const handle = await generateSessionKey({ persistence: "temporary" });
  const assigned: string[] = [];
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = (() => {
      throw new Error("HTTP logout must not be used");
    }) as typeof fetch;
    await assertRedirects(() =>
      completeSessionLogout({
        handle,
        returnTo: "/signed-out",
        location: {
          href: "",
          assign: (target) => assigned.push(String(target)),
        },
      })
    );
    assertEquals(assigned, ["/signed-out"]);
    assertEquals(await hasSessionKey({ persistence: "temporary" }), false);
  } finally {
    globalThis.fetch = originalFetch;
    await clearSessionKey({ persistence: "temporary" });
  }
});

Deno.test("connected logout uses generated Auth.Sessions.Logout", async () => {
  const handle = await generateSessionKey({ persistence: "temporary" });
  const assigned: string[] = [];
  let input: Record<string, unknown> | undefined;
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = (() => {
      throw new Error("HTTP logout must not be used");
    }) as typeof fetch;
    await assertRedirects(() =>
      completeSessionLogout({
        handle,
        connected: (value) => {
          input = value;
          return AsyncResult.ok(logoutOutput());
        },
        returnTo: "/signed-out",
        location: {
          href: "",
          assign: (target) => assigned.push(String(target)),
        },
      })
    );
    assert(input);
    assertEquals(input, {});
    assertEquals(assigned, ["/signed-out"]);
    assertEquals(await hasSessionKey({ persistence: "temporary" }), false);
  } finally {
    globalThis.fetch = originalFetch;
    await clearSessionKey({ persistence: "temporary" });
  }
});

Deno.test("disconnected logout clears local credentials", async () => {
  const handle = await generateSessionKey({ persistence: "temporary" });
  try {
    assertEquals(await logoutSession({ handle }), { success: true });
    assertEquals(await hasSessionKey({ persistence: "temporary" }), false);
  } finally {
    await clearSessionKey({ persistence: "temporary" });
  }
});
