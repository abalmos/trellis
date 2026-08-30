import "fake-indexeddb/auto";
import { assert, assertEquals, assertRejects } from "@std/assert";
import { ulid } from "ulid";

import type { AuthorizationClientState } from "../authorization_context.ts";
import { BrowserAuthorizationContextStore } from "./storage.ts";

function state(
  generation: number,
  manifestDigest: string,
): AuthorizationClientState {
  return {
    format: "trellis.authorization-client-state.v1",
    binding: "installation:https://trellis.example.com",
    trust: {
      format: "trellis.authorization-client-trust.v1",
      authority: "trellis-test",
      rootKeyId: "root-key",
      rootDigest: "root-digest",
      minimumManifestGeneration: generation,
      manifestDigestAtMinimumGeneration: manifestDigest,
    },
    session: {
      sessionId: "ses_test",
      participantDigest: "participant",
      needsDigest: "needs",
    },
    context: null,
    contextDigest: null,
    contextExpiresAt: null,
    routing: null,
  };
}

Deno.test("browser trust updates are atomic across concurrent tabs", async () => {
  const scope = `https://${ulid()}.example.com`;
  const first = new BrowserAuthorizationContextStore(scope);
  const second = new BrowserAuthorizationContextStore(scope);
  await first.commit(state(7, "manifest-7"));

  const results = await Promise.allSettled([
    first.commit(state(8, "manifest-8-a")),
    second.commit(state(8, "manifest-8-b")),
  ]);
  assertEquals(
    results.filter((result) => result.status === "fulfilled").length,
    1,
  );
  assertEquals(
    results.filter((result) => result.status === "rejected").length,
    1,
  );
  const durable = await first.load();
  assert(durable);
  assertEquals(durable.trust.minimumManifestGeneration, 8);
  await assertRejects(() => first.commit(state(7, "manifest-7")));

  const replacedRoot = state(9, "manifest-9");
  replacedRoot.trust.rootDigest = "replacement";
  await assertRejects(() => first.commit(replacedRoot));
  await first.clearContext();
  assertEquals((await first.load())?.trust.minimumManifestGeneration, 8);
  await first.resetTrust();
});
