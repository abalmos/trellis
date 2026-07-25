import { assert, assertEquals, assertRejects } from "@std/assert";
import { headers, type Msg, type NatsConnection } from "@nats-io/nats-core";
import { createUser } from "@nats-io/nkeys";
import { Type } from "typebox";

import { base64urlEncode } from "./auth/mod.ts";
import { generateSessionKey, type SessionKeyHandle } from "./auth/browser.ts";
import { testAuthorizationContext } from "./auth/test_context.ts";
import {
  ClientAuthHandledError,
  connectClientWithDeps,
  TrellisClient,
} from "./client_connect.ts";
import { MemoryAuthorizationContextStore } from "./auth/authorization_context.ts";
import { defineAppContract, defineServiceContract } from "./contract.ts";
import { AuthError, TransportError } from "./errors/index.ts";

const authRequiredRpcOwner = defineServiceContract(
  { schemas: { Empty: Type.Object({}) }, errors: { AuthError } },
  (ref) => ({
    id: "admin.test-auth-required@v1",
    displayName: "Auth Required Test",
    description: "Expose one authenticated RPC for connection tests.",
    rpc: {
      "Admin.TestAuthRequired": {
        version: "v1",
        subject: "rpc.v1.Admin.TestAuthRequired",
        input: ref.schema("Empty"),
        output: ref.schema("Empty"),
        errors: [ref.error("AuthError")],
      },
    },
  }),
);

const testContract = defineAppContract(() => ({
  id: "client.example@v1",
  displayName: "Example Client",
  description: "Example client contract",
  uses: [authRequiredRpcOwner.AdminTestAuthRequired],
}));

const authRequiredRpcContract = testContract;

const TEST_SEED = "AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA";
const TEST_PARTICIPANT = {
  id: testContract.CONTRACT_ID,
  artifactDigest: testContract.CONTRACT_DIGEST,
  needsDigest: testContract.CONTRACT_DIGEST,
};
const textDecoder = new TextDecoder();

async function createBrowserHandle(): Promise<SessionKeyHandle> {
  const handle = await generateSessionKey({ persistence: "temporary" });
  handle.sessionId = "session-key";
  return handle;
}

async function authTokenFromAuthenticator(
  authenticator: unknown,
): Promise<string> {
  const candidates = Array.isArray(authenticator)
    ? authenticator.flat(Infinity)
    : [authenticator];
  for (const candidate of candidates) {
    if (typeof candidate !== "function") continue;
    try {
      const value = await candidate("nonce");
      if (
        value && typeof value === "object" && "auth_token" in value &&
        typeof value.auth_token === "string"
      ) {
        return value.auth_token;
      }
    } catch {
      continue;
    }
  }

  throw new Error("Expected runtime authenticator to expose auth_token");
}

async function jwtFromAuthenticator(authenticator: unknown): Promise<string> {
  const candidates = Array.isArray(authenticator)
    ? authenticator.flat(Infinity)
    : [authenticator];
  for (const candidate of candidates) {
    if (typeof candidate !== "function") continue;
    try {
      const value = await candidate("nonce");
      if (
        value && typeof value === "object" && "jwt" in value &&
        typeof value.jwt === "string"
      ) {
        return value.jwt;
      }
    } catch {
      continue;
    }
  }

  throw new Error("Expected runtime authenticator to expose jwt");
}

async function waitFor(condition: () => boolean, attempts = 50): Promise<void> {
  for (let attempt = 0; attempt < attempts; attempt += 1) {
    if (condition()) return;
    await new Promise((resolve) => setTimeout(resolve, 10));
  }
  throw new Error("Timed out waiting for condition");
}

function createControllableNatsConnection(authErrorReason?: string): {
  connection: NatsConnection;
  close(): Promise<void>;
} {
  let resolveClosed = () => {};
  const closedPromise = new Promise<void>((resolve) => {
    resolveClosed = resolve;
  });
  const connection: NatsConnection & { options: { inboxPrefix: string } } = {
    options: { inboxPrefix: "_INBOX" },
    closed: async () => await closedPromise,
    close: async () => {
      resolveClosed();
    },
    publish: () => {},
    publishMessage: () => {},
    respondMessage: () => false,
    subscribe: () => {
      throw new Error("subscribe should not be called in this test");
    },
    request: async () => {
      if (authErrorReason) {
        const responseHeaders = headers();
        responseHeaders.set("status", "error");
        return {
          subject: "rpc.v1.Admin.TestAuthRequired",
          sid: 1,
          data: new Uint8Array(),
          headers: responseHeaders,
          json: () => ({
            id: "err_01",
            type: "AuthError",
            message: `Auth failed: ${authErrorReason}`,
            reason: authErrorReason,
          }),
          string: () => "",
          respond: () => false,
        } as Msg;
      }
      throw new Error("request should not be called in this test");
    },
    requestMany: async () => {
      throw new Error("requestMany should not be called in this test");
    },
    flush: async () => {},
    drain: async () => {},
    isClosed: () => false,
    isDraining: () => false,
    getServer: () => "nats://127.0.0.1:4222",
    status: () => ({
      async *[Symbol.asyncIterator]() {},
    }),
    stats: () => ({ inBytes: 0, outBytes: 0, inMsgs: 0, outMsgs: 0 }),
    rtt: async () => 0,
    reconnect: async () => {},
    setServers: () => {},
    getServers: () => [],
    [Symbol.asyncDispose]: async () => {},
  };
  return {
    connection,
    close: async () => {
      resolveClosed();
      await closedPromise;
    },
  };
}

Deno.test("connectClientWithDeps cleans browser callback URLs without CSP-unsafe evaluation", async () => {
  const mutableGlobal = globalThis as typeof globalThis & {
    window?: { history: { replaceState: typeof history.replaceState } };
    document?: unknown;
  };
  const hadWindow = "window" in mutableGlobal;
  const hadDocument = "document" in mutableGlobal;
  const originalWindow = mutableGlobal.window;
  const originalDocument = mutableGlobal.document;
  const originalFunction = globalThis.Function;
  let replacedUrl = "";

  try {
    Object.defineProperty(globalThis, "Function", {
      configurable: true,
      writable: true,
      value: () => {
        throw new EvalError("unsafe-eval blocked by test CSP");
      },
    });
    Object.defineProperty(globalThis, "window", {
      configurable: true,
      writable: true,
      value: {
        history: {
          replaceState: (
            _state: unknown,
            _title: string,
            url?: string | URL,
          ) => {
            replacedUrl = String(url ?? "");
          },
        },
      },
    });
    Object.defineProperty(globalThis, "document", {
      configurable: true,
      writable: true,
      value: {},
    });

    const handle = await createBrowserHandle();
    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(
              "https://app.example/callback?flowId=flow-a&authError=denied#done",
            ),
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: () => {
            throw new Error("transport should not load for auth errors");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.denied");
    assertEquals(replacedUrl, "/callback#done");
  } finally {
    Object.defineProperty(globalThis, "Function", {
      configurable: true,
      writable: true,
      value: originalFunction,
    });
    if (hadWindow) {
      Object.defineProperty(globalThis, "window", {
        configurable: true,
        writable: true,
        value: originalWindow,
      });
    } else {
      Reflect.deleteProperty(globalThis, "window");
    }
    if (hadDocument) {
      Object.defineProperty(globalThis, "document", {
        configurable: true,
        writable: true,
        value: originalDocument,
      });
    } else {
      Reflect.deleteProperty(globalThis, "document");
    }
  }
});

Deno.test("connectClientWithDeps tolerates additive bootstrap fields and uses reconnect-safe auth", async () => {
  const originalFetch = globalThis.fetch;
  let connectInboxPrefix = "";
  let connectAuthenticator: unknown;
  let maxReconnectAttempts: unknown;
  let connectTimeout: unknown;
  let nowMs = 1_700_000_000_000;

  try {
    globalThis.fetch = (() => {
      return Promise.resolve(
        new Response(
          JSON.stringify({
            serverNow: 1_700_000_000_000,
            sessionId: "session-key",
            inboxPrefix: "_INBOX.session-key",
            participantId: testContract.CONTRACT.id,
            participantArtifactDigest: testContract.CONTRACT_DIGEST,
            participantNeedsDigest: testContract.CONTRACT_DIGEST,
            authorizationContext: testAuthorizationContext(),
            futureServerField: { enabled: true },
            nats: {
              jwt: "jwt",
              jwtExpiresAt: 2_000_000_000,
              servers: ["nats://127.0.0.1:4222"],
              futureNatsField: "ignored",
            },
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async (options) => {
              connectAuthenticator = options.authenticator;
              connectInboxPrefix = String(options.inboxPrefix ?? "");
              maxReconnectAttempts = options.maxReconnectAttempts;
              connectTimeout = options.timeout;
              throw new Error("stop-after-connect");
            },
          }),
          now: () => nowMs,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");

    const firstToken = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      format: string;
      issuedAt: number;
      participantDigest: string;
      sessionId: string;
    };
    nowMs += 31_000;
    const secondToken = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as typeof firstToken;

    assertEquals(connectInboxPrefix, "_INBOX.session-key");
    assertEquals(maxReconnectAttempts, -1);
    assertEquals(connectTimeout, 10_000);
    assertEquals(firstToken.format, "trellis.nats-connect-token.v1");
    assertEquals(firstToken.participantDigest, testContract.CONTRACT_DIGEST);
    assertEquals(firstToken.sessionId, "session-key");
    assert(secondToken.issuedAt > firstToken.issuedAt);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps retries bootstrap once after iat_out_of_range using server offset", async () => {
  const originalFetch = globalThis.fetch;
  const bootstrapIats: number[] = [];

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        const body = JSON.parse(String(init?.body)) as { issuedAt: number };
        bootstrapIats.push(body.issuedAt);
        if (bootstrapIats.length === 1) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                reason: "iat_out_of_range",
                serverNow: 1_700_000_030_000,
              }),
              {
                status: 400,
                headers: { "Content-Type": "application/json" },
              },
            ),
          );
        }

        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_030_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(bootstrapIats, [1_700_000_000_000, 1_700_000_030_000]);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps callback bind failures to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      if (url.endsWith("/auth/flow/flow_123/bind")) {
        return Promise.resolve(
          new Response(JSON.stringify({ status: "expired" }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
            flowId: "flow_123",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("transport should not be used");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.bind_expired");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps malformed bind responses to TransportError", async () => {
  const originalFetch = globalThis.fetch;
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      if (url.includes("/auth/flow/flow-invalid/bind")) {
        return Promise.resolve(
          new Response("not-json", {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(
              "https://app.example.com/dashboard?flowId=flow-invalid",
            ),
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.bind_invalid_response");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps insufficient bind capabilities to TransportError", async () => {
  const originalFetch = globalThis.fetch;
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      if (url.includes("/auth/flow/flow-insufficient/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              status: "insufficient_capabilities",
              approval: {
                contractId: testContract.CONTRACT.id,
                contractDigest: testContract.CONTRACT_DIGEST,
                displayName: testContract.CONTRACT.displayName,
                description: testContract.CONTRACT.description,
                participantKind: "app",
                capabilities: {
                  admin: {
                    displayName: "Admin",
                    description: "Admin access",
                  },
                },
              },
              missingCapabilities: ["admin"],
              userCapabilities: [],
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(
              "https://app.example.com/dashboard?flowId=flow-insufficient",
            ),
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.bind_invalid_response");
    assertEquals(error.getContext().missingCapabilities, undefined);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps invalid login flow responses to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(JSON.stringify({ status: "pending" }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("transport should not be used");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.login_invalid_response");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps invalid bootstrap responses to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = (() => {
      return Promise.resolve(
        new Response(JSON.stringify({ status: "ready" }), {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("transport should not be used");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.bootstrap.invalid_response");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps malformed bootstrap responses to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = (() => {
      return Promise.resolve(
        new Response("not-json", {
          status: 200,
          headers: { "Content-Type": "application/json" },
        }),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.bootstrap.invalid_response");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps malformed login flow responses to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response("not-json", {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.login_invalid_response");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps maps runtime connection failures to TransportError", async () => {
  const originalFetch = globalThis.fetch;

  try {
    globalThis.fetch = (() => {
      return Promise.resolve(
        new Response(
          JSON.stringify({
            serverNow: 1_700_000_000_000,
            sessionId: "session-key",
            inboxPrefix: "_INBOX.session-key",
            participantId: testContract.CONTRACT.id,
            participantArtifactDigest: testContract.CONTRACT_DIGEST,
            participantNeedsDigest: testContract.CONTRACT_DIGEST,
            authorizationContext: testAuthorizationContext(),
            nats: {
              jwt: "jwt",
              jwtExpiresAt: 2_000_000_000,
              servers: ["nats://127.0.0.1:4222"],
            },
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("connection refused");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps preserves trellisUrl path when calling bootstrap", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      fetchUrls.push(url);
      return Promise.resolve(
        new Response(
          JSON.stringify({
            serverNow: 1_700_000_000_000,
            sessionId: "session-key",
            inboxPrefix: "_INBOX.session-key",
            participantId: testContract.CONTRACT.id,
            participantArtifactDigest: testContract.CONTRACT_DIGEST,
            participantNeedsDigest: testContract.CONTRACT_DIGEST,
            authorizationContext: testAuthorizationContext(),
            nats: {
              jwt: "jwt",
              jwtExpiresAt: 2_000_000_000,
              servers: ["nats://127.0.0.1:4222"],
            },
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com/base",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(
      fetchUrls[0],
      "https://trellis.example.com/base/bootstrap/client",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps precomputes fresh browser-mode runtime auth tokens for later reconnects", async () => {
  const originalFetch = globalThis.fetch;
  let connectAuthenticator: unknown;
  let nowMs = 1_700_000_000_000;
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = (() => {
      return Promise.resolve(
        new Response(
          JSON.stringify({
            serverNow: 1_700_000_000_000,
            sessionId: "session-key",
            inboxPrefix: "_INBOX.session-key",
            participantId: testContract.CONTRACT.id,
            participantArtifactDigest: testContract.CONTRACT_DIGEST,
            participantNeedsDigest: testContract.CONTRACT_DIGEST,
            authorizationContext: testAuthorizationContext(),
            nats: {
              jwt: "jwt",
              jwtExpiresAt: 2_000_000_000,
              servers: ["nats://127.0.0.1:4222"],
            },
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            redirectTo: "https://app.example.com/callback",
            handle: handle,
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async (options) => {
              connectAuthenticator = options.authenticator;
              throw new Error("stop-after-connect");
            },
          }),
          now: () => nowMs,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    const firstToken = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      issuedAt: number;
      participantDigest: string;
    };
    nowMs += 31_000;
    const secondToken = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      issuedAt: number;
      participantDigest: string;
    };

    assertEquals(firstToken.participantDigest, testContract.CONTRACT_DIGEST);
    assertEquals(secondToken.participantDigest, testContract.CONTRACT_DIGEST);
    assert(secondToken.issuedAt > firstToken.issuedAt);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps does not bind browser callbacks from window.location implicitly", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const fetchUrls: string[] = [];
  const handle = await createBrowserHandle();

  try {
    Object.defineProperty(globalThis, "window", {
      value: {
        location: {
          href:
            "https://app.example.com/callback?flowId=implicit-flow&redirectTo=%2Fdashboard",
        },
      },
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: { handle },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(fetchUrls, ["https://trellis.example.com/bootstrap/client"]);
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps requires explicit browser redirect state when reauth is needed", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const handle = await createBrowserHandle();

  try {
    Object.defineProperty(globalThis, "window", {
      value: {
        location: {
          href: "https://app.example.com/callback?redirectTo=%2Fdashboard",
        },
      },
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: { handle },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      Error,
      "Client authentication requires a redirectTo URL",
    );
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps redirects browser to loginUrl using ClientAuthHandledError when no onAuthRequired is set", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const handle = await createBrowserHandle();
  const testWindow = {
    location: {
      href: "https://app.example.com/callback?redirectTo=%2Fdashboard",
    },
  };
  const fetchUrls: string[] = [];

  try {
    Object.defineProperty(globalThis, "window", {
      value: testWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        const body = JSON.parse(String(init?.body ?? "null")) as {
          redirectTarget?: string;
        };
        assertEquals(body.redirectTarget, "https://app.example.com/dashboard");
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-handled",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-handled",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(testWindow.location.href),
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      ClientAuthHandledError,
      "Client authentication was handled by the caller",
    );

    assertEquals(
      testWindow.location.href,
      "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-handled",
    );
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("TrellisClient.connect preserves ClientAuthHandledError through orThrow", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const handle = await createBrowserHandle();
  const testWindow = {
    location: {
      href: "https://app.example.com/callback?redirectTo=%2Fdashboard",
    },
  };

  try {
    Object.defineProperty(globalThis, "window", {
      value: testWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        const body = JSON.parse(String(init?.body ?? "null")) as {
          redirectTarget?: string;
        };
        assertEquals(body.redirectTarget, "https://app.example.com/dashboard");
        return Promise.resolve(
          Response.json({
            state: "flow",
            flowId: "flow-handled",
            portalUrl:
              "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-handled",
          }),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    await assertRejects(
      () =>
        TrellisClient.connect({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(testWindow.location.href),
          },
        }).orThrow(),
      ClientAuthHandledError,
      "Client authentication was handled by the caller",
    );
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps lets auth continuation handle browser login without redirect", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const handle = await createBrowserHandle();
  const testWindow = {
    location: {
      href: "https://app.example.com/login?redirectTo=%2Fdashboard",
    },
  };
  const fetchUrls: string[] = [];

  try {
    Object.defineProperty(globalThis, "window", {
      value: testWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        const body = JSON.parse(String(init?.body ?? "null")) as {
          redirectTarget?: string;
        };
        assertEquals(body.redirectTarget, "https://app.example.com/dashboard");
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-handled",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-handled",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl: new URL(testWindow.location.href),
          },
          onAuthRequired: () => ({ status: "handled" }),
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      ClientAuthHandledError,
      "Client authentication was handled by the caller",
    );

    assertEquals(
      testWindow.location.href,
      "https://app.example.com/login?redirectTo=%2Fdashboard",
    );
    assertEquals(
      fetchUrls.some((url) => url.includes("/auth/flow/flow-handled/bind")),
      false,
    );
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps keeps session-bound enrollment during reconnect", async () => {
  const originalFetch = globalThis.fetch;
  let connectAuthenticator: unknown;
  let nowMs = 1_700_000_000_000;
  let bootstrapCalls = 0;
  const testConnection = createControllableNatsConnection();
  const initialSentinelSeed = textDecoder.decode(createUser().getSeed());
  const refreshedSentinelSeed = textDecoder.decode(createUser().getSeed());
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (!url.endsWith("/bootstrap/client")) {
        throw new Error(`Unexpected fetch ${url}`);
      }
      bootstrapCalls += 1;
      const contractDigest = testContract.CONTRACT_DIGEST;
      const sentinel = bootstrapCalls === 1
        ? { jwt: "jwt-a", seed: initialSentinelSeed }
        : { jwt: "jwt-b", seed: refreshedSentinelSeed };
      return Promise.resolve(
        new Response(
          JSON.stringify({
            serverNow: 1_700_000_000_000 + (bootstrapCalls - 1) * 301_000,
            sessionId: "session-key",
            inboxPrefix: "_INBOX.session-key",
            participantId: testContract.CONTRACT.id,
            participantArtifactDigest: contractDigest,
            participantNeedsDigest: contractDigest,
            authorizationContext: testAuthorizationContext(
              1_700_000_002 + (bootstrapCalls - 1) * 301,
            ),
            nats: {
              jwt: sentinel.jwt,
              jwtExpiresAt: 2_000_000_000,
              servers: ["nats://127.0.0.1:4222"],
            },
          }),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const client = await connectClientWithDeps({
      trellisUrl: "https://trellis.example.com",
      contract: testContract,
      participant: TEST_PARTICIPANT,
      auth: {
        handle: handle,
      },
    }, {
      authorizationContextStore: new MemoryAuthorizationContextStore(),
      loadTransport: async () => ({
        connect: async (options) => {
          connectAuthenticator = options.authenticator;
          return testConnection.connection;
        },
      }),
      now: () => nowMs,
    });

    assertEquals(client.connection.status.kind, "client");
    assertEquals(client.connection.status.phase, "connected");
    nowMs += 200_000;
    await authTokenFromAuthenticator(connectAuthenticator);
    const token = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      participantDigest?: string;
      issuedAt?: number;
    };

    assertEquals(bootstrapCalls, 1);
    assertEquals(token.participantDigest, testContract.CONTRACT_DIGEST);
    assertEquals(await jwtFromAuthenticator(connectAuthenticator), "jwt-a");
    assert(typeof token.issuedAt === "number");
    await testConnection.close();
    await new Promise((resolve) => setTimeout(resolve, 20));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps does not restart browser auth during reconnect", async () => {
  const originalFetch = globalThis.fetch;
  let connectAuthenticator: unknown;
  let nowMs = 1_700_000_000_000;
  let bootstrapCalls = 0;
  let authRequiredCalls = 0;
  let currentUrlValue = new URL("https://app.example.com/start");
  const testConnection = createControllableNatsConnection();
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        bootstrapCalls += 1;
        if (bootstrapCalls === 1) {
          return Promise.resolve(
            new Response(
              JSON.stringify({
                serverNow: 1_700_000_000_000,
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantId: testContract.CONTRACT.id,
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
                participantNeedsDigest: testContract.CONTRACT_DIGEST,
                authorizationContext: testAuthorizationContext(),
                nats: {
                  jwt: "jwt",
                  jwtExpiresAt: 2_000_000_000,
                  servers: ["nats://127.0.0.1:4222"],
                },
              }),
              { status: 200, headers: { "Content-Type": "application/json" } },
            ),
          );
        }
        if (bootstrapCalls === 2) {
          return Promise.resolve(
            new Response(null, { status: 401 }),
          );
        }
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_301_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        authRequiredCalls += 1;
        const body = JSON.parse(String(init?.body ?? "null"));
        assertEquals(body.redirectTarget, "https://app.example.com/after");
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-2",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-2",
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-2/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    await connectClientWithDeps({
      trellisUrl: "https://trellis.example.com",
      contract: testContract,
      participant: TEST_PARTICIPANT,
      auth: {
        currentUrl: () => currentUrlValue,
        handle: handle,
      },
      onAuthRequired: async () => ({ status: "bound", flowId: "flow-2" }),
    }, {
      authorizationContextStore: new MemoryAuthorizationContextStore(),
      loadTransport: async () => ({
        connect: async (options) => {
          connectAuthenticator = options.authenticator;
          return testConnection.connection;
        },
      }),
      now: () => nowMs,
    });

    currentUrlValue = new URL("https://app.example.com/after");
    nowMs += 200_000;
    await authTokenFromAuthenticator(connectAuthenticator);
    const token = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      participantDigest?: string;
    };

    assertEquals(authRequiredCalls, 0);
    assertEquals(bootstrapCalls, 1);
    assertEquals(token.participantDigest, testContract.CONTRACT_DIGEST);
    await testConnection.close();
    await new Promise((resolve) => setTimeout(resolve, 20));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps does not rebootstrap during NATS reconnect", async () => {
  const originalFetch = globalThis.fetch;
  let connectAuthenticator: unknown;
  let nowMs = 1_700_000_000_000;
  let bootstrapCalls = 0;
  let authRequiredCalls = 0;
  const testConnection = createControllableNatsConnection();
  const handle = await createBrowserHandle();

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      const url = String(input);
      if (url.endsWith("/bootstrap/client")) {
        bootstrapCalls += 1;
        const contractId = bootstrapCalls === 2
          ? "other.app@v1"
          : testContract.CONTRACT.id;
        const contractDigest = bootstrapCalls === 1
          ? testContract.CONTRACT_DIGEST
          : bootstrapCalls === 2
          ? "digest-other"
          : testContract.CONTRACT_DIGEST;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000 + (bootstrapCalls - 1) * 301_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: contractId,
              participantArtifactDigest: contractDigest,
              participantNeedsDigest: contractDigest,
              authorizationContext: testAuthorizationContext(
                1_700_000_002 + (bootstrapCalls - 1) * 301,
              ),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        authRequiredCalls += 1;
        const body = JSON.parse(String(init?.body ?? "null"));
        assertEquals(body.redirectTarget, "https://app.example.com/after");
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-wrong-contract",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-wrong-contract",
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-wrong-contract/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }
      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    await connectClientWithDeps({
      trellisUrl: "https://trellis.example.com",
      contract: testContract,
      participant: TEST_PARTICIPANT,
      auth: {
        redirectTo: "https://app.example.com/after",
        handle: handle,
      },
      onAuthRequired: () => ({
        status: "bound",
        flowId: "flow-wrong-contract",
      }),
    }, {
      authorizationContextStore: new MemoryAuthorizationContextStore(),
      loadTransport: async () => ({
        connect: async (options) => {
          connectAuthenticator = options.authenticator;
          return testConnection.connection;
        },
      }),
      now: () => nowMs,
    });

    nowMs += 200_000;
    await authTokenFromAuthenticator(connectAuthenticator);
    const token = JSON.parse(
      await authTokenFromAuthenticator(connectAuthenticator),
    ) as {
      participantDigest?: string;
    };

    assertEquals(authRequiredCalls, 0);
    assertEquals(bootstrapCalls, 1);
    assertEquals(token.participantDigest, testContract.CONTRACT_DIGEST);
    await testConnection.close();
    await new Promise((resolve) => setTimeout(resolve, 20));
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps uses auth continuation when bootstrap requires login", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    let bootstrapCalls = 0;
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 0) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.includes("/auth/flow/flow-1/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-1",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-1",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 1) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
          onAuthRequired: async ({ loginUrl }) => {
            assertEquals(
              loginUrl,
              "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-1",
            );
            return { status: "bound", flowId: "flow-1" };
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(
      fetchUrls.some((url) => url.includes("/auth/flow/flow-1/bind")),
      true,
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps restarts browser auth after an expired callback bind", async () => {
  const originalFetch = globalThis.fetch;
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const currentUrl = new URL(
    "https://app.example.com/dashboard?flowId=flow-expired&redirectTo=%2Fdashboard#section",
  );
  const replaceStateCalls: Array<{ url?: string | URL | null }> = [];
  const handle = await createBrowserHandle();
  const fetchUrls: string[] = [];
  const loginUrls: string[] = [];

  try {
    Object.defineProperty(globalThis, "window", {
      value: {
        history: {
          replaceState: (_: unknown, __: string, url?: string | URL | null) => {
            replaceStateCalls.push({ url });
          },
        },
      },
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.includes("/auth/flow/flow-expired/bind")) {
        return Promise.resolve(
          new Response(JSON.stringify({ status: "expired" }), {
            status: 200,
            headers: { "Content-Type": "application/json" },
          }),
        );
      }
      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(null, { status: 401 }),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-new",
              portalUrl: "https://trellis.example.com/login?flowId=flow-new",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl,
          },
          onAuthRequired: ({ loginUrl }) => {
            loginUrls.push(loginUrl);
            return { status: "handled" };
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      ClientAuthHandledError,
    );

    assertEquals(error.name, "ClientAuthHandledError");
    assertEquals(fetchUrls.some((url) => url.endsWith("/auth/requests")), true);
    assertEquals(loginUrls, [
      "https://trellis.example.com/login?flowId=flow-new",
    ]);
    assertEquals(
      currentUrl.toString(),
      "https://app.example.com/dashboard?redirectTo=%2Fdashboard#section",
    );
    assertEquals(replaceStateCalls, [{
      url: "/dashboard?redirectTo=%2Fdashboard#section",
    }]);
  } finally {
    globalThis.fetch = originalFetch;
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps surfaces browser authError callbacks without starting reauth", async () => {
  const originalWindow = globalThis.window;
  const originalDocument = globalThis.document;
  const currentUrl = new URL(
    "https://app.example.com/dashboard?authError=approval_denied#section",
  );
  const replaceStateCalls: Array<{ url?: string | URL | null }> = [];
  const handle = await createBrowserHandle();

  try {
    Object.defineProperty(globalThis, "window", {
      value: {
        history: {
          replaceState: (_: unknown, __: string, url?: string | URL | null) => {
            replaceStateCalls.push({ url });
          },
        },
      },
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: {},
      configurable: true,
      writable: true,
    });

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            handle,
            currentUrl,
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => {
            throw new Error("loadTransport should not be called");
          },
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.auth.approval_denied");
    assertEquals(
      currentUrl.toString(),
      "https://app.example.com/dashboard#section",
    );
    assertEquals(replaceStateCalls, [{ url: "/dashboard#section" }]);
  } finally {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
      writable: true,
    });
    Object.defineProperty(globalThis, "document", {
      value: originalDocument,
      configurable: true,
      writable: true,
    });
  }
});

Deno.test("connectClientWithDeps reauths when bootstrap resolves a different contract", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    let bootstrapCalls = 0;
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 0) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: "other.client@v1",
              participantArtifactDigest: "digest-other",
              participantNeedsDigest: "digest-other",
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-3/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-3",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-3",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 1) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
          onAuthRequired: async () => ({ status: "bound", flowId: "flow-3" }),
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(fetchUrls.some((url) => url.includes("/auth/requests")), true);
    assertEquals(
      fetchUrls.some((url) => url.includes("/auth/flow/flow-3/bind")),
      true,
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps reauths when bootstrap resolves stale contract digest", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    let bootstrapCalls = 0;
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 0) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: "digest-old",
              participantNeedsDigest: "digest-old",
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-stale-digest/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-stale-digest",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-stale-digest",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 1) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
          onAuthRequired: async () => ({
            status: "bound",
            flowId: "flow-stale-digest",
          }),
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(fetchUrls.some((url) => url.includes("/auth/requests")), true);
    assertEquals(
      fetchUrls.some((url) =>
        url.includes("/auth/flow/flow-stale-digest/bind")
      ),
      true,
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps reauths when bootstrap reports insufficient permissions", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    let bootstrapCalls = 0;
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 0) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              status: "not_ready",
              reason: "insufficient_permissions",
              serverNow: 1_700_000_000_000,
            }),
            {
              status: 403,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-2",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-2",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-2/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 1) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
          onAuthRequired: async ({ loginUrl }) => {
            assertEquals(
              loginUrl,
              "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-2",
            );
            return { status: "bound", flowId: "flow-2" };
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(fetchUrls.some((url) => url.endsWith("/auth/requests")), true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("connectClientWithDeps reauths when bootstrap reports contract_not_active", async () => {
  const originalFetch = globalThis.fetch;
  const fetchUrls: string[] = [];

  try {
    let bootstrapCalls = 0;
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);
      fetchUrls.push(url);
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 0) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              status: "not_ready",
              reason: "contract_not_active",
              serverNow: 1_700_000_000_000,
            }),
            {
              status: 403,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-4",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-4",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.includes("/auth/flow/flow-4/bind")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              session: {
                sessionId: "session-key",
                inboxPrefix: "_INBOX.session-key",
                participantArtifactDigest: testContract.CONTRACT_DIGEST,
              },
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }
      if (url.endsWith("/bootstrap/client") && bootstrapCalls === 1) {
        bootstrapCalls += 1;
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: "session-key",
              inboxPrefix: "_INBOX.session-key",
              participantId: testContract.CONTRACT.id,
              participantArtifactDigest: testContract.CONTRACT_DIGEST,
              participantNeedsDigest: testContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const error = await assertRejects(
      () =>
        connectClientWithDeps({
          trellisUrl: "https://trellis.example.com",
          contract: testContract,
          participant: TEST_PARTICIPANT,
          auth: {
            mode: "session_key",
            authorizationContextEphemeral: true,
            sessionKeySeed: TEST_SEED,
            sessionId: "session-key",
            redirectTo: "https://cli.example.com/callback",
          },
          onAuthRequired: async ({ loginUrl }) => {
            assertEquals(
              loginUrl,
              "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-4",
            );
            return { status: "bound", flowId: "flow-4" };
          },
        }, {
          authorizationContextStore: new MemoryAuthorizationContextStore(),
          loadTransport: async () => ({
            connect: async () => {
              throw new Error("stop-after-connect");
            },
          }),
          now: () => 1_700_000_000_000,
        }),
      TransportError,
    );

    assertEquals(error.code, "trellis.runtime.connect_failed");
    assertEquals(fetchUrls.some((url) => url.endsWith("/auth/requests")), true);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("browser clients preserve session_not_found auth-required behavior", async () => {
  const originalFetch = globalThis.fetch;
  const handle = await createBrowserHandle();
  const nats = createControllableNatsConnection("session_not_found");
  let authRequiredCalls = 0;

  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      const url = String(input);

      if (url.endsWith("/bootstrap/client")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              serverNow: 1_700_000_000_000,
              sessionId: handle.sessionKey,
              inboxPrefix: "_INBOX.session-key",
              participantId: authRequiredRpcContract.CONTRACT.id,
              participantArtifactDigest:
                authRequiredRpcContract.CONTRACT_DIGEST,
              participantNeedsDigest: authRequiredRpcContract.CONTRACT_DIGEST,
              authorizationContext: testAuthorizationContext(),
              nats: {
                jwt: "jwt",
                jwtExpiresAt: 2_000_000_000,
                servers: ["nats://127.0.0.1:4222"],
              },
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      if (url.endsWith("/auth/requests")) {
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "flow",
              flowId: "flow-session-missing",
              portalUrl:
                "https://trellis.example.com/_trellis/portal/users/login?flowId=flow-session-missing",
            }),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }

      throw new Error(`Unexpected fetch ${url}`);
    }) as typeof fetch;

    const trellis = await connectClientWithDeps({
      trellisUrl: "https://trellis.example.com",
      contract: authRequiredRpcContract,
      participant: TEST_PARTICIPANT,
      auth: {
        mode: "browser",
        handle,
        redirectTo: "https://console.example.com/admin/users",
        currentUrl: "https://console.example.com/admin/users",
      },
      onAuthRequired: () => {
        authRequiredCalls += 1;
        return { status: "handled" };
      },
    }, {
      authorizationContextStore: new MemoryAuthorizationContextStore(),
      loadTransport: async () => ({
        connect: async () => nats.connection,
      }),
      now: () => 1_700_000_000_000,
    });

    const error = await assertRejects(
      () => trellis.adminTestAuthRequired({}).orThrow(),
      AuthError,
    );

    assertEquals(error.reason, "session_not_found");
    assertEquals(authRequiredCalls, 1);
  } finally {
    await nats.close();
    globalThis.fetch = originalFetch;
  }
});
