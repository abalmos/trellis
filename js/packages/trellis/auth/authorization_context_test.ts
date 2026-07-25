import { assert, assertEquals, assertRejects, assertThrows } from "@std/assert";
import vectors from "../../../../conformance/authorization-context/vectors.json" with {
  type: "json",
};

import {
  type AuthorizationContextBundle,
  AuthorizationContextCache,
  MemoryAuthorizationContextStore,
  refreshAuthorizationContext,
} from "./authorization_context.ts";
import { FileAuthorizationContextStore } from "./file_authorization_context_store.ts";
import { createAuth } from "./session_auth.ts";

function contextBundle(): AuthorizationContextBundle {
  const chain = vectors.completeChain;
  const policy = vectors.defaults.policy;
  return {
    context: chain.contextToken,
    contextDigest: chain.contextDigest,
    refreshAt: 1_240,
    trust: {
      root: JSON.parse(chain.rootCanonicalJson),
      issuerManifestGeneration: policy.minimumManifestGeneration,
      issuerManifestDigest: chain.manifestDigest,
      issuerManifestLocator:
        `/.well-known/trellis/authorization/trust/manifest.${policy.minimumManifestGeneration}`,
      issuerCertificateLocator:
        `/.well-known/trellis/authorization/trust/certificate.${chain.issuerKeyId}.${chain.certificateDigest}`,
      policy: {
        allowedClockSkewSeconds: policy.allowedClockSkewSeconds,
        maximumContextLifetimeSeconds: policy.maximumContextLifetimeSeconds,
        maximumContextBytes: policy.maximumContextBytes,
        maximumPermissions: policy.maximumPermissions,
        maximumCapabilities: policy.maximumCapabilities,
        refreshLeadSeconds: 60,
        refreshJitterSeconds: 0,
      },
    },
  };
}

function registryFetch(input: URL | Request | string): Promise<Response> {
  const path = new URL(
    input instanceof Request ? input.url : input.toString(),
  ).pathname;
  return Promise.resolve(Response.json(JSON.parse(
    path.includes("/manifest.")
      ? vectors.completeChain.manifestCanonicalJson
      : vectors.completeChain.certificateCanonicalJson,
  )));
}

Deno.test("authorization cache verifies and installs its own Rust-issued context", async () => {
  const chain = vectors.completeChain;
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:test",
    new MemoryAuthorizationContextStore(),
    registryFetch,
  );
  const verified = await cache.install(
    bundle,
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  assertEquals(verified.contextDigest, chain.contextDigest);
  assertEquals(verified.context.sessionId, "ses_test");

  await assertRejects(() =>
    cache.install(
      { ...bundle, contextDigest: `x${bundle.contextDigest.slice(1)}` },
      { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
      policy.nowUnixSeconds,
    )
  );
});

Deno.test("authorization cache rejects non-canonical registry locators", async () => {
  const policy = vectors.defaults.policy;
  const absolute = contextBundle();
  absolute.trust.issuerManifestLocator = new URL(
    absolute.trust.issuerManifestLocator,
    "https://trellis.test",
  ).href;
  absolute.trust.issuerCertificateLocator = new URL(
    absolute.trust.issuerCertificateLocator,
    "https://trellis.test",
  ).href;
  await new AuthorizationContextCache(
    "https://trellis.test",
    "installation:absolute",
    new MemoryAuthorizationContextStore(),
    registryFetch,
  ).install(
    absolute,
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  for (
    const locator of [
      "https://attacker.test/.well-known/trellis/authorization/trust/manifest.7",
      "data:application/json,{}",
      "//trellis.test/.well-known/trellis/authorization/trust/manifest.7",
      "/.well-known/trellis/authorization/trust/../trust/manifest.7",
      "/.well-known/trellis/authorization/contexts/manifest.7",
      "/.well-known/trellis/authorization/trust/manifest.7?x=1",
    ]
  ) {
    const cache = new AuthorizationContextCache(
      "https://trellis.test",
      "installation:test",
      new MemoryAuthorizationContextStore(),
      () => {
        throw new Error("invalid locator reached fetch");
      },
    );
    const bundle = contextBundle();
    bundle.trust.issuerManifestLocator = locator;
    await assertRejects(() =>
      cache.install(
        bundle,
        { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
        policy.nowUnixSeconds,
      )
    );
  }
});

Deno.test("context refresh renews routing material and supports null recovery", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:refresh",
    new MemoryAuthorizationContextStore(),
    registryFetch,
    () => policy.nowUnixSeconds * 1_000,
  );
  await cache.install(
    bundle,
    { bootstrapJwt: "route-old", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  const auth = await createAuth({
    sessionKeySeed: vectors.completeChain.sessionSeed,
  });
  const currentDigests: Array<string | null> = [];
  let route = 0;
  const fetch: typeof globalThis.fetch = async (_input, init) => {
    const body: unknown = JSON.parse(String(init?.body));
    assert(typeof body === "object" && body !== null);
    currentDigests.push(
      Reflect.get(body, "currentContextDigest") as string | null,
    );
    route += 1;
    return Response.json({
      serverNow: policy.nowUnixSeconds * 1_000,
      authorizationContext: bundle,
      bootstrapJwt: `route-${route}`,
      bootstrapJwtExpiresAt: 2_000,
    });
  };

  await refreshAuthorizationContext({
    trellisUrl: "https://trellis.test",
    sessionId: "ses_test",
    auth,
    cache,
    fetch,
  });
  assertEquals(cache.routingJwt(), "route-1");
  await cache.clear();
  await refreshAuthorizationContext({
    trellisUrl: "https://trellis.test",
    sessionId: "ses_test",
    auth,
    cache,
    fetch,
  });
  assertEquals(cache.routingJwt(), "route-2");
  assertEquals(currentDigests, [bundle.contextDigest, null]);
});

Deno.test("stale terminal refresh cannot clear newer local routing material", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const store = new MemoryAuthorizationContextStore();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:refresh-race",
    store,
    registryFetch,
    () => policy.nowUnixSeconds * 1_000,
  );
  await cache.install(
    bundle,
    { bootstrapJwt: "route-old", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  const stale = cache.clearGuard();
  await cache.install(
    bundle,
    { bootstrapJwt: "route-new", bootstrapJwtExpiresAt: 2_100 },
    policy.nowUnixSeconds,
  );

  assertEquals(await cache.clearIfCurrent(stale), false);
  assertEquals(cache.routingJwt(), "route-new");
});

Deno.test("terminal refresh drains stale local state without clearing newer storage", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const store = new MemoryAuthorizationContextStore();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:refresh-store-race",
    store,
    registryFetch,
  );
  await cache.install(
    bundle,
    { bootstrapJwt: "route-old", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  const stale = cache.clearGuard();
  const current = await store.load();
  assert(current);
  await store.commit({
    ...current,
    routing: { bootstrapJwt: "route-new", bootstrapJwtExpiresAt: 2_100 },
  });

  assertEquals(await cache.clearIfCurrent(stale), true);
  assertEquals((await store.load())?.routing?.bootstrapJwt, "route-new");
  assertThrows(() => cache.current(policy.nowUnixSeconds));
});

Deno.test("expired context restores as recovery evidence without clearing trust", async () => {
  const chain = vectors.completeChain;
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const store = new MemoryAuthorizationContextStore();
  await store.commit({
    format: "trellis.authorization-client-state.v1",
    binding: "installation:test",
    trust: {
      format: "trellis.authorization-client-trust.v1",
      authority: "trellis-test",
      rootKeyId: JSON.parse(chain.rootCanonicalJson).keyId,
      rootDigest: chain.rootDigest,
      minimumManifestGeneration: policy.minimumManifestGeneration,
      manifestDigestAtMinimumGeneration: chain.manifestDigest,
    },
    session: {
      sessionId: "untrusted-placeholder",
      participantDigest: "untrusted-placeholder",
      needsDigest: "untrusted-placeholder",
    },
    context: bundle,
    contextExpiresAt: 1_300,
    routing: { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
  });
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:test",
    store,
    registryFetch,
  );

  assertEquals(await cache.restore(1_301), false);
  assertEquals(cache.sessionBinding().sessionId, "ses_test");
  const persisted = await store.load();
  assert(persisted);
  assertEquals(persisted.context, null);
  assertEquals(
    persisted.trust.minimumManifestGeneration,
    policy.minimumManifestGeneration,
  );
});

Deno.test("file context store keeps the trust floor across restart", async () => {
  const path = await Deno.makeTempFile();
  await Deno.remove(path);
  const first = new FileAuthorizationContextStore(path);
  await first.commit({
    format: "trellis.authorization-client-state.v1",
    binding: "service:dep:instance",
    trust: {
      format: "trellis.authorization-client-trust.v1",
      authority: "trellis-test",
      rootKeyId: "root-key",
      rootDigest: "root-digest",
      minimumManifestGeneration: 7,
      manifestDigestAtMinimumGeneration: "manifest-7",
    },
    session: {
      sessionId: "ses_test",
      participantDigest: "participant",
      needsDigest: "needs",
    },
    context: null,
    contextExpiresAt: null,
    routing: null,
  });
  const restarted = new FileAuthorizationContextStore(path);
  const current = await restarted.load();
  assert(current);
  assertEquals(current.trust.rootDigest, "root-digest");
  await assertRejects(() =>
    restarted.commit({
      ...current,
      trust: {
        ...current.trust,
        manifestDigestAtMinimumGeneration: "equivocated",
      },
    })
  );
  await restarted.resetTrust();
});

Deno.test("authorization trust pin survives context clearing", async () => {
  const store = new MemoryAuthorizationContextStore();
  await store.commit({
    format: "trellis.authorization-client-state.v1",
    binding: "installation:test",
    trust: {
      format: "trellis.authorization-client-trust.v1",
      authority: "trellis-test",
      rootKeyId: "root-key",
      rootDigest: "root-digest",
      minimumManifestGeneration: 7,
      manifestDigestAtMinimumGeneration: "manifest-digest",
    },
    session: {
      sessionId: "ses_test",
      participantDigest: "participant",
      needsDigest: "needs",
    },
    context: null,
    contextExpiresAt: null,
    routing: null,
  });
  await store.clearContext();
  assertEquals((await store.load())?.trust.minimumManifestGeneration, 7);
  const current = await store.load();
  assert(current);
  await assertRejects(async () =>
    await store.commit({
      ...current,
      trust: {
        ...current.trust,
        manifestDigestAtMinimumGeneration: "equivocated",
      },
    })
  );
});
