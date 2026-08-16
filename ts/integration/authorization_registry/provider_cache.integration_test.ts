import { assert, assertEquals, assertRejects } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { Kvm } from "@nats-io/kv";
import { jetstreamManager } from "@nats-io/jetstream";
import type { NatsConnection } from "@nats-io/nats-core";
import type {
  AuthorizationContextCache,
  AuthorizationProviderCache,
} from "../../packages/trellis/auth/authorization_context.ts";
import { integrationTestResolvedContexts } from "../../packages/trellis/auth/authorization/provider_cache.ts";
import { connectTrellisServiceWithAuthorizationTestHook } from "../../packages/trellis/service/runtime/service.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createRpcFixture } from "../rpc/_fixture.ts";

const CASE_ID = "authorization-registry.provider-cache" as const;
const fixture = createRpcFixture(CASE_ID);

liveTrellisTest({
  name:
    "authorization-registry.provider-cache resolves over NATS and applies live revocation",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const serviceKey = await runtime.registerService({
      name: fixture.serviceName,
      contract: fixture.serviceContract,
    });
    let provider: AuthorizationProviderCache | undefined;
    let connection: NatsConnection | undefined;
    let contextCache: AuthorizationContextCache | undefined;
    const originalFetch = globalThis.fetch;
    let refreshRequests = 0;
    let activeRefreshRequests = 0;
    let maximumActiveRefreshRequests = 0;
    let validationHttpRequests = 0;
    let countValidationHttpRequests = false;
    globalThis.fetch = (async (...args: Parameters<typeof fetch>) => {
      const url = new URL(
        args[0] instanceof Request ? args[0].url : String(args[0]),
      );
      if (url.pathname === "/auth/context/refresh") {
        refreshRequests += 1;
        activeRefreshRequests += 1;
        maximumActiveRefreshRequests = Math.max(
          maximumActiveRefreshRequests,
          activeRefreshRequests,
        );
        try {
          return await originalFetch(...args);
        } finally {
          activeRefreshRequests -= 1;
        }
      }
      if (countValidationHttpRequests) validationHttpRequests += 1;
      return await originalFetch(...args);
    }) as typeof fetch;
    try {
      const service = fixture.aliasRuntime(
        await connectTrellisServiceWithAuthorizationTestHook({
          authorizationContextEphemeral: true,
          trellisUrl: runtime.trellisUrl,
          contract: fixture.serviceContract,
          name: fixture.serviceName,
          identity: serviceKey,
          telemetry: false,
          runtime: {},
        }, (cache, nc, context) => {
          provider = cache;
          connection = nc;
          contextCache = context;
          context.requestRefresh();
        }).orThrow(),
      );
      try {
        await service.handleEntityGet(({ input }) =>
          Result.ok({ id: input.id, found: true })
        );
        const client = await runtime.connectClient({
          name: fixture.clientName,
          contract: fixture.clientContract,
        });
        if (!provider || !connection || !contextCache) {
          throw new Error("service authorization provider was not captured");
        }
        const providerCache = provider;
        const serviceConnection = connection;
        const serviceContextCache = contextCache;
        if (!runtime.startNatsMessageObserver) {
          throw new Error("runtime does not expose NATS observation");
        }
        const authObserver = await runtime.startNatsMessageObserver(
          "rpc.v1.Auth.*.Validate",
        );
        countValidationHttpRequests = true;
        try {
          await runtime.waitFor(() => refreshRequests === 1, {
            timeoutMs: 5_000,
          });
          await runtime.waitFor(() => activeRefreshRequests === 0, {
            timeoutMs: 5_000,
          });
          await new Promise((resolve) => setTimeout(resolve, 100));
          assertEquals(refreshRequests, 1);
          assertEquals(maximumActiveRefreshRequests, 1);
          const durable = await serviceContextCache.store.load();
          assert(durable);
          assertEquals(
            durable.trust.minimumManifestGeneration,
            serviceContextCache.minimumManifestGeneration(),
          );
          const before = providerCache.ioCounters();
          const installedContextDigests = new Set(
            integrationTestResolvedContexts(providerCache).map((entry) =>
              entry.contextDigest
            ),
          );
          assertEquals(
            (await client.entityGet({ id: "registry-first" }).orThrow()).id,
            "registry-first",
          );
          const first = providerCache.ioCounters();
          assertEquals(first.contextGets - before.contextGets, 1);
          assertEquals(first.contextResolves - before.contextResolves, 1);
          assertEquals(first.trustGets - before.trustGets, 0);
          assertEquals(validationHttpRequests, 0);
          assertEquals(authObserver.frames().length, 0);

          assertEquals(
            (await client.entityGet({ id: "registry-hit" }).orThrow()).id,
            "registry-hit",
          );
          assertEquals(providerCache.ioCounters(), first);
          assertEquals(validationHttpRequests, 0);
          assertEquals(authObserver.frames().length, 0);

          const beforeReconnect = providerCache.ioCounters();
          await serviceConnection.reconnect();
          const reconnectDeadline = Date.now() + 15_000;
          while (Date.now() < reconnectDeadline) {
            const current = providerCache.ioCounters();
            if (
              current.revocationWatchInitializations >
                beforeReconnect.revocationWatchInitializations &&
              current.trustGets > beforeReconnect.trustGets &&
              providerCache.health().healthy
            ) break;
            await new Promise((resolve) => setTimeout(resolve, 50));
          }
          const afterReconnect = providerCache.ioCounters();
          assert(
            afterReconnect.revocationWatchInitializations >
                beforeReconnect.revocationWatchInitializations &&
              afterReconnect.trustGets > beforeReconnect.trustGets &&
              providerCache.health().healthy,
            `provider did not reinitialize after reconnect: ${
              JSON.stringify({
                health: providerCache.health(),
                beforeReconnect,
                afterReconnect,
              })
            }`,
          );
          await providerCache.waitReady({ timeoutMs: 15_000 });

          const caller = integrationTestResolvedContexts(providerCache).find(
            (entry) => !installedContextDigests.has(entry.contextDigest),
          );
          if (!caller) throw new Error("resolved caller context is missing");
          const registry = await new Kvm(serviceConnection).open(
            "trellis_authorization_contexts",
          );
          const trust = await new Kvm(serviceConnection).open(
            "trellis_authorization_trust",
          );
          const direct = (await jetstreamManager(serviceConnection)).direct;
          assert(
            await direct.getMessage("KV_trellis_authorization_contexts", {
              last_by_subj:
                `$KV.trellis_authorization_contexts.${caller.contextDigest}`,
            }),
          );
          assert(
            await direct.getMessage("KV_trellis_authorization_trust", {
              last_by_subj: "$KV.trellis_authorization_trust.manifest.current",
            }),
          );
          await assertRejects(() => registry.get(caller.contextDigest));
          await assertRejects(async () => {
            const keys = await registry.keys();
            for await (const _key of keys) break;
          });
          await assertRejects(async () => {
            const watcher = await registry.watch({ key: ">" });
            for await (const _entry of watcher) break;
          });
          await assertRejects(async () => {
            const watcher = await registry.watch({ key: "context.>" });
            for await (const _entry of watcher) break;
          });
          await assertRejects(() =>
            registry.put(
              `forbidden.${caller.contextDigest}`,
              new TextEncoder().encode("forbidden"),
            )
          );
          await assertRejects(() => registry.delete(caller.contextDigest));
          await assertRejects(() => registry.purge(caller.contextDigest));
          await assertRejects(() =>
            trust.put("manifest.current", new Uint8Array())
          );

          if (!runtime.publishAuthorizationRevocation) {
            throw new Error("runtime does not expose revocation publication");
          }
          await runtime.publishAuthorizationRevocation(caller.contextDigest, {
            revokedAt: Math.floor(Date.now() / 1_000),
          });

          await runtime.waitFor(async () => {
            try {
              await client.entityGet({ id: "registry-revoked" }).orThrow();
              return false;
            } catch {
              return true;
            }
          }, { timeoutMs: 5_000 });
          assertEquals(validationHttpRequests, 0);
          assertEquals(authObserver.frames().length, 0);
        } finally {
          await authObserver.stop();
        }
      } finally {
        await service.stop();
      }
    } finally {
      globalThis.fetch = originalFetch;
    }
  },
});
