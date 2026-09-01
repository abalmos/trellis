import "fake-indexeddb/auto";
import { assert, assertEquals, assertRejects } from "@std/assert";
import { ulid } from "ulid";

import type { AuthorizationClientState } from "../authorization_context.ts";
import { BrowserAuthorizationContextStore } from "./storage.ts";

function state(
  generation: number,
  manifestDigest: string,
  sessionId = "ses_test",
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
      sessionId,
      participantDigest: "participant",
      needsDigest: "needs",
    },
    context: null,
    contextDigest: null,
    contextExpiresAt: null,
    serverClockOffsetMs: 0,
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

Deno.test("browser installation is opaque, participant-scoped, and replaces sessions only after end", async () => {
  const origin = `https://${ulid()}.example.com`;
  const firstScope = JSON.stringify([
    "trellis.browser-installation.v1",
    origin,
    "participant-a",
    "digest-a",
  ]);
  const secondScope = JSON.stringify([
    "trellis.browser-installation.v1",
    origin,
    "participant-b",
    "digest-b",
  ]);
  const firstTab = new BrowserAuthorizationContextStore(firstScope);
  const secondTab = new BrowserAuthorizationContextStore(firstScope);
  const otherParticipant = new BrowserAuthorizationContextStore(secondScope);

  const seedA = await firstTab.getOrCreateSessionSeed();
  assertEquals(await secondTab.getOrCreateSessionSeed(), seedA);
  assert(
    (await otherParticipant.getOrCreateSessionSeed()).some((value, index) =>
      value !== seedA[index]
    ),
  );

  await firstTab.commit(state(7, "manifest-7", "ses_a"));
  await assertRejects(() => secondTab.commit(state(8, "manifest-8", "ses_b")));
  await firstTab.endSession();
  assertEquals((await firstTab.load())?.session, null);
  assertEquals((await firstTab.load())?.trust.minimumManifestGeneration, 7);
  const nextSeed = await secondTab.getOrCreateSessionSeed();
  assert(nextSeed.some((value, index) => value !== seedA[index]));
  await secondTab.commit(state(8, "manifest-8", "ses_b"));
  assertEquals(await firstTab.endSession("ses_a"), false);
  assertEquals((await secondTab.load())?.session?.sessionId, "ses_b");

  await firstTab.resetTrust();
  await otherParticipant.resetTrust();
});

Deno.test("temporary browser installations use the same participant scope", async () => {
  const origin = `https://${ulid()}.example.com`;
  const first = new BrowserAuthorizationContextStore(
    JSON.stringify(["trellis.browser-installation.v1", origin, "a", "digest"]),
    "temporary",
  );
  const second = new BrowserAuthorizationContextStore(
    JSON.stringify(["trellis.browser-installation.v1", origin, "b", "digest"]),
    "temporary",
  );
  const firstSeed = await first.getOrCreateSessionSeed();
  assert(
    (await second.getOrCreateSessionSeed()).some((value, index) =>
      value !== firstSeed[index]
    ),
  );
  await first.resetTrust();
  await second.resetTrust();
});
