import { assert, assertEquals } from "@std/assert";
import { AsyncResult, isErr, UnexpectedError } from "@qlever-llc/result";
import {
  createAuthEventsValidateHandler,
  createAuthSessionsListHandler,
} from "./rpc.ts";
import type { Session } from "../schemas.ts";
import {
  base64urlEncode,
  buildEventProofInput,
  createAuth,
  sha256,
  utf8,
} from "@qlever-llc/trellis/auth";

// Retained unit coverage: pure session list filtering and injected logger use.
function matchFilter(filter: string, key: string): boolean {
  const filterParts = filter.split(".");
  const keyParts = key.split(".");

  if (filterParts.length === 1 && filterParts[0] === ">") return true;

  for (let i = 0; i < filterParts.length; i += 1) {
    const part = filterParts[i];
    if (part === ">") return true;
    if (keyParts[i] === undefined) return false;
    if (part !== "*" && part !== keyParts[i]) return false;
  }

  return keyParts.length === filterParts.length;
}

class InMemoryKV<V> {
  #store = new Map<string, V>();

  seed(key: string, value: V): void {
    this.#store.set(key, value);
  }

  keys(filter: string): AsyncResult<AsyncIterable<string>, UnexpectedError> {
    async function* iter(store: Map<string, V>) {
      for (const key of store.keys()) {
        if (matchFilter(filter, key)) yield key;
      }
    }

    return AsyncResult.ok(iter(this.#store));
  }

  get(key: string): AsyncResult<{ value: V }, UnexpectedError> {
    const value = this.#store.get(key);
    if (value === undefined) {
      return AsyncResult.err(new UnexpectedError({ context: { key } }));
    }
    return AsyncResult.ok({ value });
  }

  put(key: string, value: V): AsyncResult<void, UnexpectedError> {
    this.#store.set(key, value);
    return AsyncResult.ok(undefined);
  }

  delete(key: string): AsyncResult<void, UnexpectedError> {
    this.#store.delete(key);
    return AsyncResult.ok(undefined);
  }
}

type CapturedLog = {
  level: "trace" | "warn";
  fields: Record<string, unknown>;
  message: string;
};

function createTestLogger(logs: CapturedLog[] = []) {
  return {
    trace: (fields: Record<string, unknown>, message: string) => {
      logs.push({ level: "trace", fields, message });
    },
    warn: (fields: Record<string, unknown>, message: string) => {
      logs.push({ level: "warn", fields, message });
    },
  };
}

function sessionStorageFromKV(kv: InMemoryKV<Session>) {
  async function entries(filter: string) {
    const iter = await kv.keys(filter).take();
    const result = [] as Array<{
      sessionKey: string;
      principalId: string;
      session: Session;
    }>;
    if (isErr(iter)) return result;
    for await (const key of iter) {
      const entry = await kv.get(key).take();
      if (isErr(entry)) continue;
      const sessionKey = key;
      const session = entry.value;
      const principalId = session.type === "device"
        ? session.instanceId
        : session.type === "user"
        ? session.userId
        : session.trellisId;
      result.push({ sessionKey, principalId, session: entry.value });
    }
    return result;
  }
  return {
    getOneBySessionKey: async (sessionKey: string) => {
      const entry = await kv.get(sessionKey).take();
      return isErr(entry) ? undefined : entry.value;
    },
    listEntries: () => entries(">"),
    listEntriesByUser: async (userId: string) =>
      (await entries(">")).filter((entry) => entry.principalId === userId),
    deleteBySessionKey: async (sessionKey: string) => {
      await kv.delete(sessionKey).take();
    },
  };
}

const TEST_USER_ID = "usr_github_123";

function makeEventSession(overrides: Partial<Session> = {}): Session {
  return {
    type: "user",
    userId: TEST_USER_ID,
    identity: {
      identityId: "idn_github_123",
      provider: "github",
      subject: "123",
    },
    email: "ada@example.com",
    name: "Ada",
    createdAt: new Date("2026-04-26T00:00:00.000Z"),
    lastAuth: new Date("2026-04-26T00:00:01.000Z"),
    participantKind: "app",
    identityGrantId: "grant_1",
    contractDigest: "sha256-auth-test",
    contractId: "test.app@v1",
    contractDisplayName: "Test App",
    contractDescription: "Test app",
    delegatedCapabilities: ["items.write"],
    delegatedPublishSubjects: ["events.v1.Thing.>"],
    delegatedSubscribeSubjects: [],
    ...overrides,
  } as Session;
}

async function eventProof(args: {
  sessionKey: string;
  sign(data: Uint8Array): Uint8Array | Promise<Uint8Array>;
  subject: string;
  payload: string;
  eventId: string;
  eventTime: string;
}) {
  const payloadHash = await sha256(utf8(args.payload));
  const digest = await sha256(buildEventProofInput(
    args.sessionKey,
    args.subject,
    payloadHash,
    args.eventId,
    args.eventTime,
  ));
  return {
    proof: base64urlEncode(await args.sign(digest)),
    payloadHash: base64urlEncode(payloadHash),
  };
}

Deno.test("session RPC handlers log through the injected logger", async () => {
  const logs: CapturedLog[] = [];
  const handler = createAuthSessionsListHandler({
    logger: createTestLogger(logs),
    sessionStorage: sessionStorageFromKV(new InMemoryKV<Session>()),
  });

  const result = await handler({ input: { user: TEST_USER_ID } });
  const value = result.take();
  if (isErr(value)) throw value.error;

  assertEquals(logs, [{
    level: "trace",
    fields: { rpc: "Auth.Sessions.List", user: TEST_USER_ID },
    message: "RPC request",
  }]);
});

Deno.test("Auth.Events.Validate accepts retained event proof in session interval", async () => {
  const auth = await createAuth({
    sessionKeySeed: base64urlEncode(crypto.getRandomValues(new Uint8Array(32))),
  });
  const subject = "events.v1.Thing.Changed.one";
  const eventTime = "2026-04-26T00:00:02.000Z";
  const signed = await eventProof({
    sessionKey: auth.sessionKey,
    sign: auth.sign,
    subject,
    payload: JSON.stringify({ value: "one" }),
    eventId: "evt_1",
    eventTime,
  });
  const handler = createAuthEventsValidateHandler({
    logger: createTestLogger(),
    sessionStorage: {
      getRetainedBySessionKey: async (sessionKey: string) => ({
        sessionKey,
        principalId: TEST_USER_ID,
        session: makeEventSession(),
        status: "active",
        endedAt: null,
      }),
    },
  });

  const result = await handler({
    input: {
      sessionKey: auth.sessionKey,
      subject,
      payloadHash: signed.payloadHash,
      proof: signed.proof,
      eventId: "evt_1",
      eventTime,
    },
  });
  const value = result.take();
  if (isErr(value)) throw value.error;
  assert(value.allowed);
  assertEquals(value.status, "verified");
  assert("caller" in value);
  assert(value.caller);
  assertEquals(value.caller.type, "user");
  assert("publisher" in value);
  assertEquals(value.publisher?.kind, "user");
  assertEquals(value.publisher?.sessionStatus, "active");

  const disallowedSubject = "events.v1.Other.Changed.one";
  const disallowed = await eventProof({
    sessionKey: auth.sessionKey,
    sign: auth.sign,
    subject: disallowedSubject,
    payload: JSON.stringify({ value: "one" }),
    eventId: "evt_1b",
    eventTime,
  });
  const disallowedResult = await handler({
    input: {
      sessionKey: auth.sessionKey,
      subject: disallowedSubject,
      payloadHash: disallowed.payloadHash,
      proof: disallowed.proof,
      eventId: "evt_1b",
      eventTime,
    },
  });
  const disallowedValue = disallowedResult.take();
  if (isErr(disallowedValue)) throw disallowedValue.error;
  assertEquals(disallowedValue.allowed, false);
  assertEquals(disallowedValue.status, "subject-denied");
});

Deno.test("Auth.Events.Validate accepts retained service publisher session", async () => {
  const auth = await createAuth({
    sessionKeySeed: base64urlEncode(crypto.getRandomValues(new Uint8Array(32))),
  });
  const subject = "events.v1.Health.StatusChanged";
  const eventTime = "2026-04-26T00:00:02.000Z";
  const signed = await eventProof({
    sessionKey: auth.sessionKey,
    sign: auth.sign,
    subject,
    payload: JSON.stringify({ status: "healthy" }),
    eventId: "evt_service_1",
    eventTime,
  });
  const handler = createAuthEventsValidateHandler({
    logger: createTestLogger(),
    sessionStorage: {
      getRetainedBySessionKey: async (sessionKey: string) => ({
        sessionKey,
        principalId: "svc_trellis",
        session: {
          type: "service",
          trellisId: "svc_trellis",
          origin: "service",
          id: sessionKey,
          email: "trellis@trellis.internal",
          name: "trellis",
          createdAt: new Date("2026-04-26T00:00:00.000Z"),
          lastAuth: new Date("2026-04-26T00:00:01.000Z"),
          instanceId: "trellis-control-plane",
          deploymentId: "trellis",
          instanceKey: sessionKey,
          contractId: "trellis.core@v1",
          contractDigest: "sha256-core-test",
        },
        status: "active",
        endedAt: null,
      }),
    },
  });

  const result = await handler({
    input: {
      sessionKey: auth.sessionKey,
      subject,
      payloadHash: signed.payloadHash,
      proof: signed.proof,
      eventId: "evt_service_1",
      eventTime,
    },
  });
  const value = result.take();
  if (isErr(value)) throw value.error;
  assertEquals(value.allowed, true);
  assertEquals(value.status, "verified");
  assert("publisher" in value);
  assertEquals(value.publisher?.kind, "service");
  assertEquals(value.publisher?.deploymentId, "trellis");
  assertEquals(value.publisher?.contractId, "trellis.core@v1");
});

Deno.test("Auth.Events.Validate rejects proofs outside retained session interval", async () => {
  const auth = await createAuth({
    sessionKeySeed: base64urlEncode(crypto.getRandomValues(new Uint8Array(32))),
  });
  const subject = "events.v1.Thing.Changed.one";
  const eventTime = "2026-04-26T00:00:03.000Z";
  const signed = await eventProof({
    sessionKey: auth.sessionKey,
    sign: auth.sign,
    subject,
    payload: JSON.stringify({ value: "one" }),
    eventId: "evt_2",
    eventTime,
  });
  const handler = createAuthEventsValidateHandler({
    logger: createTestLogger(),
    sessionStorage: {
      getRetainedBySessionKey: async (sessionKey: string) => ({
        sessionKey,
        principalId: TEST_USER_ID,
        session: makeEventSession(),
        status: "revoked",
        endedAt: "2026-04-26T00:00:02.000Z",
      }),
    },
  });

  const result = await handler({
    input: {
      sessionKey: auth.sessionKey,
      subject,
      payloadHash: signed.payloadHash,
      proof: signed.proof,
      eventId: "evt_2",
      eventTime,
    },
  });
  const value = result.take();
  if (isErr(value)) throw value.error;
  assertEquals(value.allowed, false);
  assertEquals(value.status, "outside-session-window");
  assert("publisher" in value);
  assertEquals(value.publisher?.sessionStatus, "revoked");
});
