import {
  headers as natsHeaders,
  type Msg,
  type NatsConnection,
  type Payload,
  type Subscription,
} from "@nats-io/nats-core";
import { isErr } from "@qlever-llc/result";
import { assert, assertEquals, assertRejects, assertThrows } from "@std/assert";

import vectors from "../../../../conformance/authorization-context/vectors.json" with {
  type: "json",
};
import type { PermissionAtom as DescriptorPermissionAtom } from "../participant_runtime/api.ts";
import { type VerifiedCaller, verifyLocalAuthorization } from "../session.ts";
import { integrationTestResolvedContexts } from "./authorization/provider_cache.ts";
import {
  type AuthorizationContextBundle,
  AuthorizationContextCache,
  AuthorizationContextRefreshError,
  AuthorizationProviderCache,
  type AuthorizationProviderEvent,
  type AuthorizationProviderRequest,
  type AuthorizationRuntimeBinding,
  startAuthorizationContextRefresh,
} from "./authorization_context.ts";
import type { PermissionAtom } from "./protocol_wasm.ts";
import { createAuth } from "./session_auth.ts";
import {
  base64urlEncode,
  canonicalizeJsonValue,
  sha256,
  utf8,
} from "./utils.ts";

const chain = vectors.completeChain;
const policy = vectors.defaults.policy;

function bundle(): AuthorizationContextBundle {
  return {
    context: JSON.parse(chain.contextCanonicalJson),
    issuer: {
      keyId: chain.issuerKeyId,
      publicKey: chain.issuerPublicKey,
      state: "active",
    },
    authorizationRegistry: { contextBucket: "contexts" },
    policy: {
      allowedClockSkewSeconds: policy.allowedClockSkewSeconds,
      maximumContextLifetimeSeconds: policy.maximumContextLifetimeSeconds,
      maximumContextBytes: policy.maximumContextBytes,
      maximumPermissions: policy.maximumPermissions,
      refreshLeadSeconds: 60,
      refreshJitterSeconds: 0,
    },
  };
}

function runtimeBinding(): AuthorizationRuntimeBinding {
  const context = JSON.parse(chain.contextCanonicalJson);
  return {
    connectionId: context.connectionId,
    loginSessionId: context.loginSessionId,
    participantId: context.participantId,
    inboxPrefix: context.inboxPrefix,
    transports: { native: { natsServers: ["nats://127.0.0.1:4222"] } },
  };
}

function cache(fetch: typeof globalThis.fetch = globalThis.fetch) {
  return new AuthorizationContextCache(
    "https://trellis.test",
    fetch,
    () => policy.nowUnixSeconds * 1_000,
  );
}

async function installedCache(fetch?: typeof globalThis.fetch) {
  const value = cache(fetch);
  await value.install(
    bundle(),
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
    undefined,
    runtimeBinding(),
  );
  return value;
}

Deno.test("authorization refresh can use native bootstrap", async () => {
  const value = await installedCache();
  const auth = await createAuth({ sessionKeySeed: chain.sessionSeed });
  let refreshed!: () => void;
  const didRefresh = new Promise<void>((resolve) => refreshed = resolve);
  const stop = startAuthorizationContextRefresh({
    trellisUrl: "https://trellis.test",
    sessionId: value.current().context.connectionId,
    auth,
    cache: value,
    refresh: async (shouldInstall) => {
      assert(shouldInstall());
      refreshed();
      return value.current();
    },
  });

  value.requestRefresh();
  await didRefresh;
  stop();
  assert(
    new AuthorizationContextRefreshError(401, "identity_not_found").terminal,
  );
  assert(
    new AuthorizationContextRefreshError(401, "identity_inactive").terminal,
  );
});

function permission(): PermissionAtom {
  return vectors.defaults.permission as PermissionAtom;
}

function request(): AuthorizationProviderRequest {
  return {
    contextDigest: chain.contextDigest,
    sessionKey: JSON.parse(chain.contextCanonicalJson).sessionKey,
    subject: vectors.defaults.request.subject,
    reply: vectors.defaults.request.reply,
    payload: utf8(vectors.defaults.request.payload),
    iat: vectors.defaults.request.iat,
    requestId: vectors.defaults.request.requestId,
    proof: chain.requestProof,
    requiredPermissions: [permission()],
    requiredCapabilities: [],
  };
}

function event(): AuthorizationProviderEvent {
  return {
    contextDigest: chain.contextDigest,
    sessionKey: JSON.parse(chain.contextCanonicalJson).sessionKey,
    subject: vectors.defaults.event.subject,
    payload: utf8(vectors.defaults.event.payload),
    eventId: vectors.defaults.event.eventId,
    eventTime: vectors.defaults.event.eventTime,
    proof: chain.eventProof,
    requiredPermissions: [permission()],
    requiredCapabilities: [],
  };
}

async function signedContext(connectionId: string): Promise<[string, string]> {
  const context = JSON.parse(chain.contextCanonicalJson) as Record<
    string,
    unknown
  >;
  delete context.signature;
  context.connectionId = connectionId;
  const domain = utf8("trellis.authorization-context.v1");
  const canonical = utf8(canonicalizeJsonValue(context));
  const input = new Uint8Array(8 + domain.length + canonical.length);
  const view = new DataView(input.buffer);
  view.setUint32(0, domain.length);
  input.set(domain, 4);
  view.setUint32(4 + domain.length, canonical.length);
  input.set(canonical, 8 + domain.length);
  const issuer = await createAuth({ sessionKeySeed: chain.issuerSeed });
  const signed = {
    ...context,
    signature: base64urlEncode(await issuer.sign(await sha256(input))),
  };
  const json = canonicalizeJsonValue(signed);
  return [base64urlEncode(await sha256(utf8(json))), json];
}

type Registry = {
  contexts: Map<string, string>;
  revocations?: Map<string, number>;
  reads: string[];
};

function providerNats(registry: Registry): NatsConnection {
  type TestStatus = ReturnType<NatsConnection["status"]> extends
    AsyncIterable<infer T> ? T : never;
  type TestSubscription = Subscription & { deliver(message: Msg): void };
  type TestConsumer = {
    stream: string;
    name: string;
    config: Record<string, unknown>;
    pending: Array<{ key: string; value: Uint8Array; revision: number }>;
  };
  const consumers = new Map<string, TestConsumer>();
  const subscriptions = new Map<string, TestSubscription>();
  const encoder = new TextEncoder();
  const decoder = new TextDecoder();
  let revision = 0;

  const record = (key: string) => {
    const digest = key.startsWith("revocation.")
      ? key.slice("revocation.".length)
      : key;
    const value = key.startsWith("revocation.")
      ? registry.revocations?.get(digest) === undefined
        ? undefined
        : JSON.stringify({ revokedAt: registry.revocations?.get(digest) })
      : registry.contexts.get(digest);
    return value === undefined
      ? undefined
      : { key, value: encoder.encode(value), revision: ++revision };
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
  const noMessage = () =>
    response({
      error: { code: 404, err_code: 10037, description: "no messages" },
    });
  const message = (
    consumer: TestConsumer,
    item: { key: string; value: Uint8Array; revision: number },
  ): Msg => ({
    subject: `$KV.contexts.${item.key}`,
    sid: 1,
    data: item.value,
    reply:
      `$JS.ACK._.account.${consumer.stream}.${consumer.name}.1.${item.revision}.1.1.0`,
    headers: natsHeaders(),
    respond: () => true,
    json: <T>() => JSON.parse(decoder.decode(item.value)) as T,
    string: () => decoder.decode(item.value),
  });
  const status = () => {
    let done = false;
    let wake: (() => void) | undefined;
    const iterator = {
      next: async (): Promise<IteratorResult<TestStatus>> => {
        if (done) return { done: true, value: undefined as never };
        await new Promise<void>((resolve) => wake = resolve);
        return { done: true, value: undefined as never };
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

  return {
    info: undefined,
    options: { inboxPrefix: "_INBOX.test" },
    closed: () => Promise.resolve(undefined),
    close: () => Promise.resolve(),
    publish: () => {},
    publishMessage: () => {},
    respondMessage: () => true,
    subscribe: (
      subject: string,
      options?: { callback?: (error: Error | null, message: Msg) => void },
    ) => {
      let closed = false;
      let resolveClosed = () => {};
      const closedPromise = new Promise<void>((resolve) =>
        resolveClosed = resolve
      );
      const close = () => {
        if (closed) return;
        closed = true;
        resolveClosed();
      };
      const subscription: TestSubscription = {
        closed: closedPromise,
        unsubscribe: close,
        drain: () => {
          close();
          return Promise.resolve();
        },
        [Symbol.asyncDispose]: () => {
          close();
          return Promise.resolve();
        },
        isDraining: () => false,
        isClosed: () => closed,
        callback: options?.callback ?? (() => {}),
        getSubject: () => subject,
        getReceived: () => 0,
        getProcessed: () => 0,
        getPending: () => 0,
        getID: () => 1,
        getMax: () => undefined,
        [Symbol.asyncIterator]: async function* () {},
        deliver: (value) => options?.callback?.(null, value),
      };
      subscriptions.set(subject, subscription);
      queueMicrotask(() => {
        for (const consumer of consumers.values()) {
          if (consumer.config.deliver_subject !== subject) continue;
          for (const item of consumer.pending.splice(0)) {
            subscription.deliver(message(consumer, item));
          }
        }
      });
      return subscription;
    },
    request: async (subject: string, payload?: Payload): Promise<Msg> => {
      if (subject === "$JS.API.INFO") return response({ type: "account_info" });
      if (subject.startsWith("$JS.API.DIRECT.GET.")) {
        const marker = subject.indexOf(".$KV.contexts.");
        const key = marker < 0
          ? ""
          : subject.slice(marker + ".$KV.contexts.".length);
        registry.reads.push(key);
        const item = record(key);
        if (!item) {
          return { ...response({}), headers: natsHeaders(404, "No Messages") };
        }
        const headers = natsHeaders();
        headers.set("Nats-Stream", "KV_contexts");
        headers.set("Nats-Sequence", String(item.revision));
        headers.set("Nats-Time-Stamp", new Date(0).toISOString());
        headers.set("Nats-Subject", `$KV.contexts.${key}`);
        return { ...response({}), data: item.value, headers };
      }
      if (subject.startsWith("$JS.API.STREAM.MSG.GET.")) {
        const body = JSON.parse(decoder.decode(payload as Uint8Array)) as {
          last_by_subj?: string;
        };
        const key = body.last_by_subj?.replace("$KV.contexts.", "") ?? "";
        registry.reads.push(key);
        const item = record(key);
        return item
          ? response({
            message: {
              subject: `$KV.contexts.${key}`,
              seq: item.revision,
              time: new Date(0).toISOString(),
              data: btoa(String.fromCharCode(...item.value)),
            },
          })
          : noMessage();
      }
      if (subject.startsWith("$JS.API.CONSUMER.CREATE.")) {
        const body = JSON.parse(decoder.decode(payload as Uint8Array)) as {
          config: Record<string, unknown>;
        };
        const stream = subject.slice("$JS.API.CONSUMER.CREATE.".length).split(
          ".",
        )[0] ?? "";
        const name = String(body.config.name ?? `consumer-${consumers.size}`);
        const key = String(body.config.filter_subject ?? "").replace(
          "$KV.contexts.",
          "",
        );
        const pending = record(key);
        consumers.set(`${stream}:${name}`, {
          stream,
          name,
          config: body.config,
          pending: pending ? [pending] : [],
        });
        return response({
          stream_name: stream,
          name,
          config: body.config,
          num_pending: pending ? 1 : 0,
        });
      }
      if (subject.startsWith("$JS.API.CONSUMER.INFO.")) {
        const [stream, name] = subject.slice(
          "$JS.API.CONSUMER.INFO.".length,
        ).split(".", 2);
        const consumer = consumers.get(`${stream}:${name}`);
        return consumer
          ? response({
            stream_name: stream,
            name,
            config: consumer.config,
            num_pending: consumer.pending.length,
          })
          : noMessage();
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
}

async function provider(registry: Registry) {
  const installed = await installedCache(() => {
    throw new Error("unexpected issuer fetch");
  });
  const value = await AuthorizationProviderCache.attach(
    providerNats(registry),
    installed.bundle().authorizationRegistry,
    installed,
    { now: () => policy.nowUnixSeconds },
  );
  value.start();
  await value.waitReady();
  return value;
}

Deno.test("online issuer context verification rejects signed-content tampering", async () => {
  const value = cache();
  const verified = await value.install(
    bundle(),
    { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
    policy.nowUnixSeconds,
  );
  assertEquals(verified.contextDigest, chain.contextDigest);
  assertEquals(verified.context.connectionId, "01JY0000000000000000000001");

  const tampered = bundle();
  (tampered.context as { principalId: string }).principalId = "tampered";
  await assertRejects(() =>
    cache().install(
      tampered,
      { bootstrapJwt: "route", bootstrapJwtExpiresAt: 2_000 },
      policy.nowUnixSeconds,
    )
  );
});

Deno.test("authorization context cache installs, binds runtime, and clears in memory", async () => {
  const value = await installedCache();
  assertEquals(
    value.current(policy.nowUnixSeconds).contextDigest,
    chain.contextDigest,
  );
  assertEquals(value.runtimeBinding(), runtimeBinding());
  assertEquals(value.routingJwt(), "route");

  await value.clear();
  assertThrows(() => value.current(policy.nowUnixSeconds));
  assertThrows(() => value.bundle());
  assertEquals(value.runtimeBinding(), runtimeBinding());
});

Deno.test("provider resolves a cold context once and reuses the verified hot entry", async () => {
  const registry: Registry = {
    contexts: new Map([[chain.contextDigest, chain.contextCanonicalJson]]),
    reads: [],
  };
  const value = await provider(registry);
  try {
    await value.resolveContext(chain.contextDigest);
    const cold = value.ioCounters();
    await value.resolveContext(chain.contextDigest);
    assertEquals(cold.contextGets, 1);
    assertEquals(cold.contextVerifications, 1);
    assertEquals(value.ioCounters(), cold);
  } finally {
    value.stop();
  }
});

Deno.test("provider reconnect resolves a fresh context and revocation watch", async () => {
  const registry: Registry = {
    contexts: new Map([[chain.contextDigest, chain.contextCanonicalJson]]),
    reads: [],
  };
  const value = await provider(registry);
  try {
    await value.resolveContext(chain.contextDigest);
    value.observeConnectionPhase("disconnected");
    value.observeConnectionPhase("connected");
    await value.resolveContext(chain.contextDigest);
    assertEquals(value.ioCounters().contextGets, 2);
  } finally {
    value.stop();
  }
});

Deno.test("lean request and event verifier outputs are enriched from cached context", async () => {
  const value = await provider({
    contexts: new Map([[chain.contextDigest, chain.contextCanonicalJson]]),
    reads: [],
  });
  try {
    const requestResult = await value.verifyRequest(request());
    assert(requestResult.ok, JSON.stringify(requestResult));
    assertEquals(
      requestResult.context.principalId,
      "01JY0000000000000000000002",
    );

    const eventResult = await value.verifyEvent(event());
    assert(eventResult.ok, JSON.stringify(eventResult));
    assertEquals(
      eventResult.context.connectionId,
      "01JY0000000000000000000001",
    );
    assertEquals(
      eventResult.publisher.connectionId,
      eventResult.context.connectionId,
    );

    const mismatchedRequest = request();
    mismatchedRequest.sessionKey =
      "UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert(!(await value.verifyRequest(mismatchedRequest)).ok);
    const mismatchedEvent = event();
    mismatchedEvent.sessionKey =
      "UAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
    assert(!(await value.verifyEvent(mismatchedEvent)).ok);

    const headers = natsHeaders();
    headers.set("authorization-context", chain.contextDigest);
    headers.set("session-key", request().sessionKey);
    headers.set("proof", chain.requestProof);
    headers.set("iat", String(vectors.defaults.request.iat));
    headers.set("request-id", vectors.defaults.request.requestId);
    const local = await verifyLocalAuthorization({
      kind: "request",
      cache: value,
      message: {
        data: utf8(vectors.defaults.request.payload),
        headers,
        reply: vectors.defaults.request.reply,
        subject: vectors.defaults.request.subject,
      },
      permission: {
        apiId: "documents@v1",
        apiVersion: "v1",
        surfaceKind: "rpc",
        surfaceName: "Documents.Get",
        action: "call",
      } satisfies DescriptorPermissionAtom,
      requiredCapabilities: [],
    });
    const caller = local.take();
    if (isErr(caller)) throw caller.error;
    assertEquals(
      (caller as VerifiedCaller).connectionId,
      "01JY0000000000000000000001",
    );
  } finally {
    value.stop();
  }
});

Deno.test("provider explicitly denies a revoked context", async () => {
  const value = await provider({
    contexts: new Map([[chain.contextDigest, chain.contextCanonicalJson]]),
    revocations: new Map([[chain.contextDigest, 1_150]]),
    reads: [],
  });
  try {
    const requestResult = await value.verifyRequest(request());
    assert(!requestResult.ok);
    assertEquals(requestResult.error.code, "PermissionDenied");
    const eventResult = await value.verifyEvent(event());
    assert(!eventResult.ok);
    assertEquals(eventResult.error.code, "EventRevoked");
  } finally {
    value.stop();
  }
});

Deno.test("provider LRU stays at 256 entries and evicts the oldest context", async () => {
  const contexts = new Map<string, string>();
  for (let index = 0; index < 257; index += 1) {
    const [digest, context] = await signedContext(`connection-${index}`);
    contexts.set(digest, context);
  }
  const value = await provider({ contexts, reads: [] });
  try {
    for (const digest of contexts.keys()) {
      await value.resolveContext(digest);
    }
    const digests = [...contexts.keys()];
    const first = digests[0];
    const last = digests.at(-1);
    assert(first && last);
    const resolved = integrationTestResolvedContexts(value).map((entry) =>
      entry.contextDigest
    );
    assertEquals(resolved.length, 256);
    assert(!resolved.includes(first));
    assert(resolved.includes(last));
  } finally {
    value.stop();
  }
});
