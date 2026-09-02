import { assert, assertEquals, assertRejects, assertThrows } from "@std/assert";
import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
  type Payload,
  type Subscription,
} from "@nats-io/nats-core";
import { isErr } from "@qlever-llc/result";

import vectors from "../../../../conformance/authorization-context/vectors.json" with {
  type: "json",
};
import {
  type AuthorizationContextBundle,
  AuthorizationContextCache,
  AuthorizationContextRefreshError,
  AuthorizationProviderCache,
  type AuthorizationProviderEvent,
  type AuthorizationProviderRequest,
  type AuthorizationRuntimeBinding,
  MemoryAuthorizationContextStore,
  refreshAuthorizationContext,
  startAuthorizationContextRefresh,
} from "./authorization_context.ts";
import { FileAuthorizationContextStore } from "./file_authorization_context_store.ts";
import { buildEventProofInput } from "./proof.ts";
import type { PermissionAtom } from "./protocol_wasm.ts";
import { createAuth } from "./session_auth.ts";
import { base64urlEncode, sha256, utf8 } from "./utils.ts";
import { type VerifiedCaller, verifyLocalAuthorization } from "../session.ts";
import type { PermissionAtom as DescriptorPermissionAtom } from "../contract_support/runtime.ts";

function contextBundle(): AuthorizationContextBundle {
  const chain = vectors.completeChain;
  const policy = vectors.defaults.policy;
  return {
    context: JSON.parse(chain.contextCanonicalJson),
    trust: {
      root: JSON.parse(chain.rootCanonicalJson),
      manifest: JSON.parse(chain.manifestCanonicalJson),
      authorizationRegistry: {
        trustBucket: "trust",
        contextBucket: "contexts",
      },
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

async function providerContextCache(
  now = 1_100,
): Promise<AuthorizationContextCache> {
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:provider",
    new MemoryAuthorizationContextStore(),
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
    () => now * 1_000,
  );
  await cache.install(
    contextBundle(),
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    now,
  );
  return cache;
}

function providerRevocation(revokedAt = 1_150): Record<string, unknown> {
  return { revokedAt, futureField: true };
}

function providerNats(
  calls: string[],
  revocations: unknown[] = [],
  missingContext = false,
  watchDelayMs = 0,
): NatsConnection {
  const chain = vectors.completeChain;
  const generation = 7;
  const records = new Map<string, { value: Uint8Array; revision: number }>();
  let revision = 0;
  const put = (bucket: string, key: string, value: string) => {
    revision += 1;
    records.set(`${bucket}:${key}`, {
      value: utf8(value),
      revision,
    });
  };
  if (!missingContext) {
    put("contexts", chain.contextDigest, chain.contextCanonicalJson);
  }
  put(
    "trust",
    "manifest.current",
    JSON.stringify({
      generation,
      digest: chain.manifestDigest,
      futureField: true,
    }),
  );
  put("trust", `manifest.${generation}`, chain.manifestCanonicalJson);
  for (const [index, value] of revocations.entries()) {
    put(
      "contexts",
      `revocation.${index === 0 ? chain.contextDigest : `missing-${index}`}`,
      JSON.stringify(value),
    );
  }

  type TestStatus = ReturnType<NatsConnection["status"]> extends
    AsyncIterable<infer T> ? T : never;
  type TestSubscription = Subscription & { deliver(message: Msg): void };
  type TestConsumer = {
    stream: string;
    name: string;
    config: Record<string, unknown>;
    pending: Array<
      { key: string; record: { value: Uint8Array; revision: number } }
    >;
  };
  const consumers = new Map<string, TestConsumer>();
  const subscriptions = new Map<string, TestSubscription>();
  const decoder = new TextDecoder();
  const encoder = new TextEncoder();

  const subjectMatches = (pattern: string, subject: string): boolean => {
    const patternParts = pattern.split(".");
    const subjectParts = subject.split(".");
    return patternParts.every((part, index) =>
      part === ">" ||
      (part === "*"
        ? subjectParts[index] !== undefined
        : subjectParts[index] === part)
    );
  };
  const consumerPending = (config: Record<string, unknown>) => {
    if (config.deliver_policy === "new") return [];
    const filter = typeof config.filter_subject === "string"
      ? config.filter_subject
      : ">";
    const bucket = typeof config.filter_subject === "string"
      ? config.filter_subject.split(".")[1]
      : undefined;
    if (!bucket) return [];
    return [...records.entries()]
      .filter(([key]) => key.startsWith(`${bucket}:`))
      .map(([key, record]) => ({ key, record }))
      .filter(({ key }) =>
        subjectMatches(
          filter,
          `$KV.${key.replace(":", ".")}`,
        )
      );
  };
  const messageFor = (
    consumer: TestConsumer,
    item: { key: string; record: { value: Uint8Array; revision: number } },
    pending: number,
    deliverySequence: number,
  ): Msg => {
    const [bucket, key] = item.key.split(":", 2);
    const subject = `$KV.${bucket}.${key}`;
    const stream = consumer.stream;
    return {
      subject,
      sid: 1,
      data: item.record.value,
      reply:
        `$JS.ACK._.account.${stream}.${consumer.name}.1.${item.record.revision}.${deliverySequence}.1.${pending}`,
      headers: natsHeaders(),
      respond: () => true,
      json: <T>() => JSON.parse(decoder.decode(item.record.value)) as T,
      string: () => decoder.decode(item.record.value),
    };
  };
  const deliverPending = (subject: string): void => {
    const subscription = subscriptions.get(subject);
    if (!subscription) return;
    for (const consumer of consumers.values()) {
      if (consumer.config.deliver_subject !== subject) continue;
      const pending = consumer.pending.splice(0);
      pending.forEach((item, index) => {
        subscription.deliver(
          messageFor(
            consumer,
            item,
            pending.length - index - 1,
            index + 1,
          ),
        );
      });
    }
  };
  const response = (value: unknown): Msg => {
    const data = encoder.encode(JSON.stringify(value));
    return {
      subject: "_INBOX.response",
      sid: 1,
      data,
      headers: natsHeaders(),
      respond: () => true,
      json: <T>() => value as T,
      string: () => decoder.decode(data),
    };
  };
  const noMessage = (): Msg =>
    response({
      error: {
        code: 404,
        err_code: 10037,
        description: "no messages",
      },
    });
  const status = () => {
    let done = false;
    let wake: (() => void) | undefined;
    const iterator = {
      next: async (): Promise<IteratorResult<TestStatus>> => {
        if (done) return { done: true, value: undefined as never };
        await new Promise<void>((resolve) => wake = resolve);
        return done
          ? { done: true, value: undefined as never }
          : await iterator.next();
      },
      return: async (): Promise<IteratorResult<TestStatus>> => {
        done = true;
        wake?.();
        return { done: true, value: undefined as never };
      },
      [Symbol.asyncIterator]() {
        return iterator;
      },
      stop() {
        void iterator.return();
      },
    };
    return iterator as ReturnType<NatsConnection["status"]>;
  };
  const connection = {
    info: undefined,
    options: { inboxPrefix: "_INBOX.test" },
    closed: () => Promise.resolve(undefined),
    close: () => Promise.resolve(),
    publish: () => {},
    publishMessage: () => {},
    respondMessage: () => true,
    subscribe: (
      subject: string,
      opts?: { callback?: (error: Error | null, message: Msg) => void },
    ) => {
      let closed = false;
      let resolveClosed = () => {};
      const closedPromise = new Promise<void>((resolve) =>
        resolveClosed = resolve
      );
      const subscription: TestSubscription = {
        closed: closedPromise,
        unsubscribe: () => {
          closed = true;
          resolveClosed();
        },
        drain: () => {
          closed = true;
          resolveClosed();
          return Promise.resolve();
        },
        [Symbol.asyncDispose]: () => {
          closed = true;
          resolveClosed();
          return Promise.resolve();
        },
        isDraining: () => false,
        isClosed: () => closed,
        callback: opts?.callback ?? (() => {}),
        getSubject: () => subject,
        getReceived: () => 0,
        getProcessed: () => 0,
        getPending: () => 0,
        getID: () => 1,
        getMax: () => undefined,
        [Symbol.asyncIterator]: async function* () {},
        deliver: (message) => opts?.callback?.(null, message),
      };
      subscriptions.set(subject, subscription);
      queueMicrotask(() => deliverPending(subject));
      return subscription;
    },
    request: async (
      subject: string,
      payload?: Payload,
    ): Promise<Msg> => {
      if (subject === "$JS.API.INFO") return response({ type: "account_info" });
      if (subject.startsWith("$JS.API.DIRECT.GET.")) {
        const marker = subject.indexOf(".$KV.");
        const key = marker === -1
          ? undefined
          : subject.slice(marker + ".$KV.".length).replace(".", ":");
        if (!key) return noMessage();
        calls.push(key);
        if (
          key === `contexts:revocation.${chain.contextDigest}` &&
          revocations[0] !== undefined
        ) {
          const value = utf8(JSON.stringify(revocations[0]));
          records.set(key, { value, revision: ++revision });
        }
        const record = records.get(key);
        if (!record) {
          return {
            ...response({}),
            headers: natsHeaders(404, "No Messages"),
          };
        }
        const [bucket, recordKey] = key.split(":", 2);
        const directHeaders = natsHeaders();
        directHeaders.set("Nats-Stream", `KV_${bucket}`);
        directHeaders.set("Nats-Sequence", String(record.revision));
        directHeaders.set("Nats-Time-Stamp", new Date(0).toISOString());
        directHeaders.set("Nats-Subject", `$KV.${bucket}.${recordKey}`);
        return {
          ...response({}),
          data: record.value,
          headers: directHeaders,
        };
      }
      if (subject.startsWith("$JS.API.STREAM.MSG.GET.")) {
        const body = JSON.parse(decoder.decode(payload as Uint8Array)) as {
          last_by_subj?: string;
        };
        const key = body.last_by_subj?.replace(/^\$KV\./, "").replace(".", ":");
        if (!key) return noMessage();
        calls.push(key);
        const record = records.get(key);
        if (!record) return noMessage();
        const [bucket, recordKey] = key.split(":", 2);
        return response({
          message: {
            subject: `$KV.${bucket}.${recordKey}`,
            seq: record.revision,
            time: new Date(0).toISOString(),
            data: btoa(String.fromCharCode(...record.value)),
          },
        });
      }
      if (subject.startsWith("$JS.API.CONSUMER.CREATE.")) {
        if (watchDelayMs > 0) {
          await new Promise((resolve) => setTimeout(resolve, watchDelayMs));
        }
        const body = JSON.parse(decoder.decode(payload as Uint8Array)) as {
          config: Record<string, unknown>;
        };
        const rest = subject.slice("$JS.API.CONSUMER.CREATE.".length);
        const stream = rest.split(".")[0] ?? "";
        const config = body.config;
        const name = String(config.name ?? `consumer-${consumers.size}`);
        const consumer: TestConsumer = {
          stream,
          name,
          config,
          pending: consumerPending(config),
        };
        consumers.set(`${stream}:${name}`, consumer);
        return response({
          stream_name: stream,
          name,
          config: { ...config, deliver_subject: config.deliver_subject },
          num_pending: consumer.pending.length,
        });
      }
      if (subject.startsWith("$JS.API.CONSUMER.INFO.")) {
        const rest = subject.slice("$JS.API.CONSUMER.INFO.".length);
        const [stream, name] = rest.split(".", 2);
        const consumer = consumers.get(`${stream}:${name}`);
        if (!consumer) return noMessage();
        return response({
          stream_name: stream,
          name,
          config: consumer.config,
          num_pending: consumer.pending.length,
        });
      }
      return response({});
    },
    requestMany: () => Promise.resolve((async function* () {})()),
    flush: () => Promise.resolve(),
    drain: () => Promise.resolve(),
    isClosed: () => false,
    isDraining: () => false,
    getServer: () => "nats://127.0.0.1:4222",
    getServerVersion: () => "2.10.0",
    status,
    stats: () => ({ inBytes: 0, outBytes: 0, inMsgs: 0, outMsgs: 0 }),
    rtt: () => Promise.resolve(0),
    reconnect: () => Promise.resolve(),
    setServers: () => {},
    getServers: () => [],
    features: { get: () => ({ min: "2.10.0", ok: true }) },
    _resub: () => {},
    [Symbol.asyncDispose]: () => Promise.resolve(),
  } as NatsConnection;
  return connection;
}

async function readyProvider(
  cache: AuthorizationContextCache,
  calls: string[] = [],
  now: number | (() => number) = 1_100,
  revocations: unknown[] = [],
  missingContext = false,
): Promise<AuthorizationProviderCache> {
  const provider = await AuthorizationProviderCache.attach(
    providerNats(calls, revocations, missingContext),
    cache.bundle().trust.authorizationRegistry,
    cache,
    { now: typeof now === "function" ? now : () => now },
  );
  provider.start();
  await provider.waitReady({ timeoutMs: 1_000 });
  return provider;
}

function providerPermission(): PermissionAtom {
  return {
    action: "call",
    target: {
      api: "documents@v1",
      kind: "apiSurface",
      name: "Documents.Get",
      surface: "rpc",
    },
  };
}

function providerRequest(
  proof = vectors.completeChain.requestProof,
  contextDigest = vectors.completeChain.contextDigest,
): AuthorizationProviderRequest {
  const request = vectors.defaults.request;
  return {
    contextDigest,
    subject: request.subject,
    reply: request.reply,
    payload: utf8(request.payload),
    iat: request.iat,
    requestId: request.requestId,
    proof,
    requiredPermissions: [providerPermission()],
    requiredCapabilities: ["platform.read"],
  };
}

async function eventProof(
  eventId: string,
  eventTime: string,
): Promise<string> {
  const chain = vectors.completeChain;
  const event = vectors.defaults.event;
  const auth = await createAuth({
    sessionKeySeed: chain.sessionSeed,
    contextDigest: chain.contextDigest,
  });
  const input = buildEventProofInput(
    chain.contextDigest,
    event.subject,
    await sha256(utf8(event.payload)),
    eventId,
    eventTime,
  );
  return base64urlEncode(await auth.sign(await sha256(input)));
}

function providerEvent(
  proof: string,
  eventId = vectors.defaults.event.eventId,
  eventTime = vectors.defaults.event.eventTime,
): AuthorizationProviderEvent {
  const event = vectors.defaults.event;
  return {
    contextDigest: vectors.completeChain.contextDigest,
    subject: event.subject,
    payload: utf8(event.payload),
    eventId,
    eventTime,
    proof,
    requiredPermissions: [providerPermission()],
    requiredCapabilities: [],
  };
}

Deno.test("authorization cache verifies and installs its own Rust-issued context", async () => {
  const chain = vectors.completeChain;
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:test",
    new MemoryAuthorizationContextStore(),
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
  );
  const verified = await cache.install(
    bundle,
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  assertEquals(verified.contextDigest, chain.contextDigest);
  assertEquals(verified.context.sessionId, "ses_test");
});

Deno.test("authorization cache rejects tampered embedded trust chain", async () => {
  const policy = vectors.defaults.policy;
  for (
    const mutate of [
      (bundle: AuthorizationContextBundle) => {
        bundle.trust.manifest = { generation: 9 };
      },
      (bundle: AuthorizationContextBundle) => {
        bundle.trust.manifest = {};
      },
    ]
  ) {
    const cache = new AuthorizationContextCache(
      "https://trellis.test",
      "installation:test",
      new MemoryAuthorizationContextStore(),
      () => {
        throw new Error("tampered chain reached fetch");
      },
    );
    const bundle = contextBundle();
    mutate(bundle);
    await assertRejects(() =>
      cache.install(
        bundle,
        { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
        policy.nowUnixSeconds,
      )
    );
  }
});

Deno.test("authorization cache rejects each mismatched runtime binding field", async () => {
  const policy = vectors.defaults.policy;
  const signed = contextBundle().context as {
    sessionId: string;
    participant: { id: string; artifactDigest: string; needsDigest: string };
    inboxPrefix: string;
  };
  const valid: AuthorizationRuntimeBinding = {
    sessionId: signed.sessionId,
    participantId: signed.participant.id,
    participantArtifactDigest: signed.participant.artifactDigest,
    participantNeedsDigest: signed.participant.needsDigest,
    inboxPrefix: signed.inboxPrefix,
    transports: { websocket: { natsServers: ["wss://trellis.test/nats"] } },
  };
  for (
    const mutate of [
      (runtime: AuthorizationRuntimeBinding) => runtime.sessionId = "ses_other",
      (runtime: AuthorizationRuntimeBinding) =>
        runtime.participantId = "participant-other",
      (runtime: AuthorizationRuntimeBinding) =>
        runtime.participantArtifactDigest = "artifact-other",
      (runtime: AuthorizationRuntimeBinding) =>
        runtime.participantNeedsDigest = "needs-other",
      (runtime: AuthorizationRuntimeBinding) =>
        runtime.inboxPrefix = "_INBOX.other",
      (runtime: AuthorizationRuntimeBinding) =>
        runtime.transports = { websocket: { natsServers: [] } },
    ]
  ) {
    const runtime = structuredClone(valid);
    mutate(runtime);
    const cache = new AuthorizationContextCache(
      "https://trellis.test",
      "installation:test",
      new MemoryAuthorizationContextStore(),
      () => {
        throw new Error("runtime mismatch reached fetch");
      },
    );
    await assertRejects(
      () =>
        cache.install(
          contextBundle(),
          { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
          policy.nowUnixSeconds,
          undefined,
          runtime,
        ),
      Error,
      "authorization runtime binding does not match signed context",
    );
  }
});

Deno.test("context refresh renews routing material and supports null recovery", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const signed = bundle.context as {
    participant: { id: string; artifactDigest: string; needsDigest: string };
    inboxPrefix: string;
  };
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:refresh",
    new MemoryAuthorizationContextStore(),
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
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
  const fetch: typeof globalThis.fetch = (_input, init) => {
    const body: unknown = JSON.parse(String(init?.body));
    assert(typeof body === "object" && body !== null);
    currentDigests.push(
      Reflect.get(body, "currentContextDigest") as string | null,
    );
    route += 1;
    return Promise.resolve(Response.json({
      serverNow: policy.nowUnixSeconds * 1_000,
      authorizationContext: bundle,
      session: {
        sessionId: "ses_test",
        principalId: "usr_test",
        principalKind: "user",
        participantId: signed.participant.id,
        participantKind: "app",
        participantArtifactDigest: signed.participant.artifactDigest,
        participantNeedsDigest: signed.participant.needsDigest,
        sessionPublicKey: auth.sessionKey,
        sessionKeyId: "session-key-test",
        inboxPrefix: signed.inboxPrefix,
        state: "active",
        createdAt: 1_000_000,
        lastSeenAt: 1_100_000,
        expiresAt: null,
        revokedAt: null,
        version: 1,
      },
      nats: {
        jwt: `route-${route}`,
        jwtExpiresAt: 2_000,
        transports: {
          native: { natsServers: ["nats://127.0.0.1:4222"] },
        },
      },
    }));
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
  assertEquals(currentDigests, [vectors.completeChain.contextDigest, null]);
});

Deno.test("context refresh terminality uses exact machine codes", () => {
  assert(new AuthorizationContextRefreshError(401, "user_inactive").terminal);
  assert(
    new AuthorizationContextRefreshError(409, "context_refresh_mismatch")
      .terminal,
  );
  assert(
    !new AuthorizationContextRefreshError(503, "authorization_pending")
      .terminal,
  );
  assert(
    !new AuthorizationContextRefreshError(401, "session_revoked later")
      .terminal,
  );
});

Deno.test("refresh wake is retained before registration and coalesced while running", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const cache = await providerContextCache(policy.nowUnixSeconds);
  const auth = await createAuth({
    sessionKeySeed: vectors.completeChain.sessionSeed,
  });
  const releases: Array<() => void> = [];
  let calls = 0;
  let active = 0;
  let maximumActive = 0;
  let reconnects = 0;
  const fetch: typeof globalThis.fetch = async () => {
    calls += 1;
    active += 1;
    maximumActive = Math.max(maximumActive, active);
    await new Promise<void>((resolve) => releases.push(resolve));
    active -= 1;
    return Response.json({
      serverNow: policy.nowUnixSeconds * 1_000,
      authorizationContext: bundle,
      bootstrapJwt: `route-${calls}`,
      bootstrapJwtExpiresAt: 2_000,
      session: {
        sessionId: "ses_test",
        participantId: "documents-web",
        participantArtifactDigest: "A".repeat(43),
        participantNeedsDigest: "B".repeat(43),
        inboxPrefix: "_INBOX.test",
      },
      nats: {
        jwt: `route-${calls}`,
        jwtExpiresAt: 2_000,
        servers: ["nats://127.0.0.1:4222"],
        transports: {
          native: { natsServers: ["nats://127.0.0.1:4222"] },
        },
      },
    });
  };
  const waitForCalls = async (expected: number) => {
    for (let attempt = 0; attempt < 100 && calls < expected; attempt += 1) {
      await new Promise((resolve) => setTimeout(resolve, 1));
    }
    assertEquals(calls, expected);
  };

  cache.requestRefresh();
  const stop = startAuthorizationContextRefresh({
    trellisUrl: "https://trellis.test",
    sessionId: "ses_test",
    auth,
    cache,
    fetch,
    onRefresh: () => {
      reconnects += 1;
    },
  });
  await waitForCalls(1);
  cache.requestRefresh();
  cache.requestRefresh();
  releases.shift()?.();
  await waitForCalls(2);
  assertEquals(maximumActive, 1);
  assertEquals(reconnects, 0);

  stop();
  cache.requestRefresh();
  releases.shift()?.();
  await new Promise((resolve) => setTimeout(resolve, 5));
  assertEquals(calls, 2);

  const stopRestarted = startAuthorizationContextRefresh({
    trellisUrl: "https://trellis.test",
    sessionId: "ses_test",
    auth,
    cache,
    fetch,
    onRefresh: () => {
      reconnects += 1;
    },
  });
  await waitForCalls(3);
  releases.shift()?.();
  stopRestarted();
  assertEquals(maximumActive, 1);
  assertEquals(reconnects, 0);
});

Deno.test("stale terminal refresh cannot clear newer local routing material", async () => {
  const policy = vectors.defaults.policy;
  const bundle = contextBundle();
  const store = new MemoryAuthorizationContextStore();
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:refresh-race",
    store,
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
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
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
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
  const signedContext = JSON.parse(chain.contextCanonicalJson);
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
      sessionId: signedContext.sessionId,
      participantDigest: signedContext.participant.artifactDigest,
      needsDigest: signedContext.participant.needsDigest,
    },
    context: bundle,
    contextDigest: chain.contextDigest,
    contextExpiresAt: 1_300,
    serverClockOffsetMs: 0,
    routing: { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
  });
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:test",
    store,
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
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
    contextDigest: null,
    contextExpiresAt: null,
    serverClockOffsetMs: 0,
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

Deno.test("new manifest floor survives a crash before context refresh", async () => {
  const store = new MemoryAuthorizationContextStore();
  const signedContext = contextBundle().context as {
    sessionId: string;
    participant: { id: string; artifactDigest: string; needsDigest: string };
    inboxPrefix: string;
  };
  const cache = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:provider",
    store,
    () => {
      throw new Error("unexpected refresh");
    },
    () => 1_100_000,
  );
  await cache.install(
    contextBundle(),
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    1_100,
    undefined,
    {
      sessionId: signedContext.sessionId,
      participantId: signedContext.participant.id,
      participantArtifactDigest: signedContext.participant.artifactDigest,
      participantNeedsDigest: signedContext.participant.needsDigest,
      inboxPrefix: signedContext.inboxPrefix,
      transports: { native: { natsServers: ["nats://localhost:4222"] } },
    },
  );

  assert(await cache.advanceManifestFloor(8, "manifest-8"));
  const beforeRestart = await store.load();
  assertEquals(beforeRestart?.trust.minimumManifestGeneration, 8);
  assert(beforeRestart?.context);

  const restored = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:provider",
    store,
    () => {
      throw new Error("unexpected refresh");
    },
    () => 1_100_000,
  );
  assertEquals(await restored.restore(1_100), false);
  const durable = await store.load();
  assertEquals(durable?.trust.minimumManifestGeneration, 8);
  assertEquals(durable?.context, null);
  assertEquals(durable?.routing, null);
  assertEquals(restored.sessionBinding(), beforeRestart?.session);
  assertEquals(restored.runtimeBinding(), beforeRestart?.runtime);
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
    contextDigest: null,
    contextExpiresAt: null,
    serverClockOffsetMs: 0,
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

Deno.test("provider cache rechecks revocation without refetching its installed context", async () => {
  const chain = vectors.completeChain;
  const calls: string[] = [];
  const cache = await readyProvider(await providerContextCache(), calls);
  try {
    const verified = await cache.resolveContext(chain.contextDigest);
    assertEquals(verified.contextDigest, chain.contextDigest);
    const fetched = cache.ioCounters();
    await cache.resolveContext(chain.contextDigest);
    assertEquals(cache.ioCounters(), {
      ...fetched,
      revocationGets: fetched.revocationGets + 1,
    });
    assertEquals(fetched.contextGets, 0);
  } finally {
    cache.stop();
  }
});

Deno.test("provider context resolution enforces the current validity window", async () => {
  let now = 1_100;
  const cache = await readyProvider(
    await providerContextCache(),
    [],
    () => now,
  );
  try {
    await cache.resolveContext(vectors.completeChain.contextDigest);
    now = 1_400;
    await assertRejects(() =>
      cache.resolveContext(vectors.completeChain.contextDigest)
    );
  } finally {
    cache.stop();
  }
});

Deno.test("provider cache wakes refresh for own revocation and disconnect", async () => {
  const revokedCache = await providerContextCache();
  let revocationWakes = 0;
  const unregisterRevocation = revokedCache.registerRefreshRequest(() => {
    revocationWakes += 1;
  });
  const revoked = await readyProvider(
    revokedCache,
    [],
    1_100,
    [providerRevocation()],
  );
  assertEquals(revocationWakes, 1);
  revoked.stop();
  unregisterRevocation();

  const disconnectedCache = await providerContextCache();
  let disconnectWakes = 0;
  const unregisterDisconnect = disconnectedCache.registerRefreshRequest(() => {
    disconnectWakes += 1;
  });
  const disconnected = await readyProvider(disconnectedCache);
  disconnected.observeConnectionPhase("reconnecting");
  disconnected.observeConnectionPhase("disconnected");
  assertEquals(disconnectWakes, 1);
  disconnected.stop();
  unregisterDisconnect();
});

Deno.test("provider cache reuses installed trust while rechecking revocation", async () => {
  const policy = vectors.defaults.policy;
  const installed = new AuthorizationContextCache(
    "https://trellis.test",
    "installation:provider",
    new MemoryAuthorizationContextStore(),
    () => {
      throw new Error("unexpected trust HTTP fetch");
    },
    () => policy.nowUnixSeconds * 1_000,
  );
  await installed.install(
    contextBundle(),
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  const calls: string[] = [];
  const provider = await readyProvider(
    installed,
    calls,
    policy.nowUnixSeconds,
  );
  try {
    const before = provider.ioCounters();
    const [verified] = await Promise.all([
      provider.resolveContext(vectors.completeChain.contextDigest),
      provider.resolveContext(vectors.completeChain.contextDigest),
      provider.resolveContext(vectors.completeChain.contextDigest),
    ]);
    assertEquals(verified.contextDigest, vectors.completeChain.contextDigest);
    assertEquals(provider.ioCounters(), {
      ...before,
      contextVerifications: before.contextVerifications + 1,
      revocationGets: before.revocationGets + 3,
    });
  } finally {
    provider.stop();
  }
});

Deno.test("provider cache coalesces concurrent unknown context resolution", async () => {
  const calls: string[] = [];
  const cache = await readyProvider(await providerContextCache(), calls);
  try {
    const missingDigest = `B${vectors.completeChain.contextDigest.slice(1)}`;
    const results = await Promise.allSettled([
      cache.resolveContext(missingDigest),
      cache.resolveContext(missingDigest),
      cache.resolveContext(missingDigest),
    ]);
    assert(results.every((result) => result.status === "rejected"));
    assertEquals(cache.ioCounters().contextGets, 1);
    assertEquals(cache.ioCounters().revocationGets, 3);
  } finally {
    cache.stop();
  }
});

Deno.test("provider cache fails closed for missing and revoked contexts", async () => {
  const missingCalls: string[] = [];
  const missing = await readyProvider(
    await providerContextCache(),
    missingCalls,
    1_100,
    [],
    true,
  );
  try {
    const missingResult = await missing.verifyRequest(
      providerRequest(
        undefined,
        `B${vectors.completeChain.contextDigest.slice(1)}`,
      ),
    );
    assert(!missingResult.ok);
  } finally {
    missing.stop();
  }

  const malformedCalls: string[] = [];
  const malformedCache = await providerContextCache();
  const malformed = await AuthorizationProviderCache.attach(
    providerNats(malformedCalls, [{
      ...providerRevocation(),
      revokedAt: "not-a-timestamp",
    }]),
    malformedCache.bundle().trust.authorizationRegistry,
    malformedCache,
    { now: () => 1_100 },
  );
  malformed.start();
  try {
    await assertRejects(() => malformed.waitReady({ timeoutMs: 50 }));
    await assertRejects(
      () =>
        malformed.verifyEvent(
          providerEvent(vectors.completeChain.eventProof),
        ),
      Error,
      "authorization provider is not healthy",
    );
  } finally {
    malformed.stop();
  }

  const revokedCalls: string[] = [];
  const revoked = await readyProvider(
    await providerContextCache(),
    revokedCalls,
    1_100,
    [providerRevocation()],
  );
  try {
    const revokedResult = await revoked.verifyRequest(providerRequest());
    assert(!revokedResult.ok);
    assertEquals(revoked.ioCounters().contextGets, 0);
  } finally {
    revoked.stop();
  }
});

Deno.test("provider cache verifies historical events and rejects revoked contexts", async () => {
  const chain = vectors.completeChain;
  const historicalCalls: string[] = [];
  const historical = await readyProvider(
    await providerContextCache(),
    historicalCalls,
    1_400,
  );
  try {
    const historicalResult = await historical.verifyEvent(
      providerEvent(chain.eventProof),
    );
    assert(historicalResult.ok);
  } finally {
    historical.stop();
  }

  const revokedCalls: string[] = [];
  const revoked = await readyProvider(
    await providerContextCache(),
    revokedCalls,
    1_400,
    [providerRevocation()],
  );
  try {
    const beforeRevocation = await revoked.verifyEvent(
      providerEvent(
        await eventProof("evt_before_revocation", "1970-01-01T00:19:00Z"),
        "evt_before_revocation",
        "1970-01-01T00:19:00Z",
      ),
    );
    assert(!beforeRevocation.ok);
    if (!beforeRevocation.ok) {
      assertEquals(beforeRevocation.error.code, "EventRevoked");
    }
    const atRevocation = await revoked.verifyEvent(
      providerEvent(
        await eventProof("evt_at_revocation", vectors.defaults.event.eventTime),
        "evt_at_revocation",
      ),
    );
    assert(!atRevocation.ok);
    if (!atRevocation.ok) assertEquals(atRevocation.error.code, "EventRevoked");
  } finally {
    revoked.stop();
  }
});

Deno.test("same request and event proofs are accepted", async () => {
  const calls: string[] = [];
  const cache = await readyProvider(await providerContextCache(), calls);
  try {
    const invalid = await cache.verifyRequest(
      providerRequest(`A${vectors.completeChain.requestProof.slice(1)}`),
    );
    assert(!invalid.ok);
    const first = await cache.verifyRequest(providerRequest());
    assert(first.ok, JSON.stringify(first));
    const duplicate = await cache.verifyRequest(providerRequest());
    assert(duplicate.ok);

    const event = await cache.verifyEvent(
      providerEvent(vectors.completeChain.eventProof),
    );
    assert(event.ok);
    const eventDuplicate = await cache.verifyEvent(
      providerEvent(vectors.completeChain.eventProof),
    );
    assert(eventDuplicate.ok);
    assertEquals(cache.ioCounters().contextVerifications, 1);
  } finally {
    cache.stop();
  }
});

function descriptorPermission(
  surfaceName = "Documents.Get",
): DescriptorPermissionAtom {
  return {
    apiId: "documents@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName,
    action: "call",
  };
}

function requestMessage(overrides: {
  data?: Uint8Array;
  reply?: string;
  subject?: string;
  proof?: string;
} = {}): Pick<Msg, "data" | "headers" | "reply" | "subject"> {
  const headers = natsHeaders();
  headers.set("session-key", vectors.completeChain.sessionPublicKey);
  headers.set("authorization-context", vectors.completeChain.contextDigest);
  headers.set("proof", overrides.proof ?? vectors.completeChain.requestProof);
  headers.set("iat", String(vectors.defaults.request.iat));
  headers.set("request-id", vectors.defaults.request.requestId);
  return {
    data: overrides.data ?? utf8(vectors.defaults.request.payload),
    headers,
    reply: overrides.reply ?? vectors.defaults.request.reply,
    subject: overrides.subject ?? vectors.defaults.request.subject,
  };
}

function eventMessage(overrides: {
  data?: Uint8Array;
  subject?: string;
  proof?: string;
} = {}): Pick<Msg, "data" | "headers" | "subject"> {
  const headers = natsHeaders();
  headers.set("session-key", vectors.completeChain.sessionPublicKey);
  headers.set("authorization-context", vectors.completeChain.contextDigest);
  headers.set("proof", overrides.proof ?? vectors.completeChain.eventProof);
  headers.set("Nats-Msg-Id", vectors.defaults.event.eventId);
  headers.set("Trellis-Event-Time", vectors.defaults.event.eventTime);
  return {
    data: overrides.data ?? utf8(vectors.defaults.event.payload),
    headers,
    subject: overrides.subject ?? vectors.defaults.event.subject,
  };
}

async function localProviderCache(
  calls: string[],
): Promise<AuthorizationProviderCache> {
  return await readyProvider(await providerContextCache(), calls);
}

function eventDescriptorPermission(): DescriptorPermissionAtom {
  return descriptorPermission();
}

Deno.test("local request auth projects only verified cross-context caller data", async () => {
  const calls: string[] = [];
  const result = await verifyLocalAuthorization({
    kind: "request",
    cache: await localProviderCache(calls),
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const value = result.take();
  if (isErr(value)) throw value.error;
  const caller: VerifiedCaller = value;
  assertEquals(caller, {
    type: "verified",
    sessionKey: vectors.completeChain.sessionPublicKey,
    principal: { kind: "user", id: "usr_test" },
    participant: {
      kind: "app",
      id: "documents-web",
      artifactDigest: "BAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQEBAQ",
      needsDigest: "BQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQUFBQU",
    },
    deploymentId: null,
    instanceId: null,
    sessionId: "ses_test",
    capabilities: ["platform.read"],
    inboxPrefix: "_INBOX.test",
  });
  assert(calls.length > 0);
});

Deno.test("local request auth denies a missing exact atom before handler dispatch", async () => {
  const result = await verifyLocalAuthorization({
    kind: "request",
    cache: await localProviderCache([]),
    message: requestMessage(),
    permission: descriptorPermission("Documents.Delete"),
    requiredCapabilities: ["platform.read"],
  });
  const value = result.take();
  if (!isErr(value)) throw new Error("missing permission was accepted");
  assertEquals(value.error.reason, "insufficient_permissions");
});

Deno.test("local request auth rejects altered subject, reply, and payload", async () => {
  const cache = await localProviderCache([]);
  for (
    const [message, reason] of [
      [
        requestMessage({ subject: "rpc.v1.Documents.Delete" }),
        "invalid_signature",
      ],
      [
        requestMessage({ reply: "_INBOX.other.reply" }),
        "reply_subject_mismatch",
      ],
      [requestMessage({ data: utf8('{"id":"other"}') }), "invalid_signature"],
    ] as const
  ) {
    const result = await verifyLocalAuthorization({
      kind: "request",
      cache,
      message,
      permission: descriptorPermission(),
      requiredCapabilities: ["platform.read"],
    });
    const value = result.take();
    if (!isErr(value)) throw new Error("altered request was accepted");
    assertEquals(value.error.reason, reason);
  }
});

Deno.test("local request auth accepts a duplicate before handler dispatch", async () => {
  const cache = await localProviderCache([]);
  let handlerCalls = 0;
  const first = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const firstValue = first.take();
  if (!isErr(firstValue)) handlerCalls += 1;
  const duplicate = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const duplicateValue = duplicate.take();
  if (isErr(duplicateValue)) throw duplicateValue.error;
  handlerCalls += 1;
  assertEquals(handlerCalls, 2);
});

Deno.test("local request auth does not let a forged proof poison the caller projection", async () => {
  const cache = await localProviderCache([]);
  const forged = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage({
      proof: `A${vectors.completeChain.requestProof.slice(1)}`,
    }),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const forgedValue = forged.take();
  if (!isErr(forgedValue)) throw new Error("forged proof was accepted");
  const valid = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const value = valid.take();
  if (isErr(value)) throw value.error;
  assertEquals(value.principal.id, "usr_test");
});

Deno.test("local auth resolves no unknown context for an unknown descriptor", async () => {
  const calls: string[] = [];
  const cache = await localProviderCache(calls);
  const before = calls.length;
  const result = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage({
      subject: "rpc.v1.Unknown.Missing",
      proof: vectors.completeChain.requestProof,
    }),
    permission: undefined,
    requiredCapabilities: [],
  });
  const value = result.take();
  if (!isErr(value)) throw new Error("unknown descriptor was accepted");
  assertEquals(value.error.reason, "insufficient_permissions");
  assertEquals(calls.length, before);
  cache.stop();
});

Deno.test("local auth cache hits do not fetch the provider registry", async () => {
  const calls: string[] = [];
  const cache = await localProviderCache(calls);
  const first = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const firstValue = first.take();
  if (isErr(firstValue)) throw firstValue.error;
  const fetched = calls.length;
  const second = await verifyLocalAuthorization({
    kind: "request",
    cache,
    message: requestMessage(),
    permission: descriptorPermission(),
    requiredCapabilities: ["platform.read"],
  });
  const secondValue = second.take();
  if (isErr(secondValue)) throw secondValue.error;
  assertEquals(calls.length, fetched + 1);
});

Deno.test("local event auth uses exact permission and raw event bytes", async () => {
  const cache = await localProviderCache([]);
  const valid = await verifyLocalAuthorization({
    kind: "event",
    cache,
    message: eventMessage(),
    permission: eventDescriptorPermission(),
    requiredCapabilities: [],
  });
  const validValue = valid.take();
  if (isErr(validValue)) throw validValue.error;
  assertEquals(validValue.participant.id, "documents-web");

  const altered = await verifyLocalAuthorization({
    kind: "event",
    cache,
    message: eventMessage({ data: utf8('{"id":"other"}') }),
    permission: eventDescriptorPermission(),
    requiredCapabilities: [],
  });
  const alteredValue = altered.take();
  if (!isErr(alteredValue)) throw new Error("altered event was accepted");
  assertEquals(alteredValue.error.reason, "invalid_signature");

  const duplicate = await verifyLocalAuthorization({
    kind: "event",
    cache,
    message: eventMessage(),
    permission: eventDescriptorPermission(),
    requiredCapabilities: [],
  });
  const duplicateValue = duplicate.take();
  if (isErr(duplicateValue)) throw duplicateValue.error;
  assertEquals(duplicateValue.participant.id, "documents-web");
});
