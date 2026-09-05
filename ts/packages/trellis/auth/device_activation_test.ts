import { assert, assertEquals, assertRejects } from "@std/assert";
import { AsyncResult } from "@qlever-llc/result";
import type {
  OperationEvent,
  OperationSignalAck,
  OperationSnapshot,
  TerminalOperation,
} from "../operations.ts";
import { UnexpectedError } from "../errors/index.ts";

import {
  type AuthDeviceUserAuthoritiesListInput,
  type AuthDeviceUserAuthoritiesListOutput,
  type AuthDeviceUserAuthoritiesRevokeInput,
  type AuthDeviceUserAuthoritiesRevokeResponse,
  type AuthResolveDeviceUserAuthoritiesInput,
  type AuthResolveDeviceUserAuthoritiesOperation,
  type AuthResolveDeviceUserAuthoritiesOutput,
  type AuthResolveDeviceUserAuthoritiesProgress,
  buildDeviceActivationPayload,
  createDeviceActivationClient,
  deriveDeviceConfirmationCode,
  deriveDeviceIdentity,
  type DeviceActivationTransport,
  encodeDeviceActivationPayload,
  parseDeviceActivationPayload,
  verifyDeviceConfirmationCode,
  waitForDeviceActivation,
} from "./device_activation.ts";
const PARTICIPANT_DIGEST = "A".repeat(43);

function unsupportedActivationOperationControl() {
  return AsyncResult.err(
    new UnexpectedError({
      cause: new Error("fake activation operation control is unsupported"),
    }),
  );
}

Deno.test("device activation payload helpers round-trip encoded payloads", async () => {
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(7));
  const payload = await buildDeviceActivationPayload({
    activationKey: identity.activationKey,
    publicIdentityKey: identity.publicIdentityKey,
    nonce: "nonce_123",
  });

  const encoded = encodeDeviceActivationPayload(payload);
  assertEquals(parseDeviceActivationPayload(encoded), payload);
});

Deno.test("device activation helpers verify confirmation codes", async () => {
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(9));

  const confirmationCode = await deriveDeviceConfirmationCode({
    activationKey: identity.activationKey,
    publicIdentityKey: identity.publicIdentityKey,
    nonce: "nonce_123",
  });
  assertEquals(confirmationCode.length, 8);
  assert(
    await verifyDeviceConfirmationCode({
      activationKey: identity.activationKey,
      publicIdentityKey: identity.publicIdentityKey,
      nonce: "nonce_123",
      confirmationCode: confirmationCode.toLowerCase(),
    }),
  );
});

function bootstrapWaitArgs(
  identity: Awaited<ReturnType<typeof deriveDeviceIdentity>>,
) {
  return {
    trellisUrl: "https://trellis.example.com",
    publicIdentityKey: identity.publicIdentityKey,
    identitySeed: identity.identitySeed,
    activationKey: identity.activationKey,
    deploymentId: "reader.default",
    instanceId: "dev_123",
    principalId: "device_123",
    participantId: "acme.reader@v1",
    participantArtifactDigest: PARTICIPANT_DIGEST,
    participantNeedsDigest: PARTICIPANT_DIGEST,
    nonce: "nonce_123",
  };
}

function activationPendingBody(): Record<string, unknown> {
  const serverNow = Date.now();
  return {
    state: "activation_pending",
    serverNow,
    activation: {
      state: "pending",
      reviewId: "dar_123",
      activationUrl: "https://trellis.example.com/login/device?flowId=dar_123",
      expiresAt: serverNow + 1_000,
      retryAfterMs: 5,
    },
  };
}

function bootstrapReadyBody(): Record<string, unknown> {
  return {
    state: "ready",
    serverNow: Date.now(),
  };
}

Deno.test("device activation wait retries the bootstrap route until ready", async () => {
  const originalFetch = globalThis.fetch;
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(5));
  const waitArgs = bootstrapWaitArgs(identity);
  const urls: string[] = [];
  const sessionKeys: string[] = [];
  let calls = 0;

  try {
    globalThis.fetch = ((input: URL | Request | string, init?: RequestInit) => {
      urls.push(String(input));
      sessionKeys.push(
        (JSON.parse(String(init?.body)) as { newSessionPublicKey: string })
          .newSessionPublicKey,
      );
      calls += 1;
      const body = calls === 1 ? activationPendingBody() : bootstrapReadyBody();
      return Promise.resolve(
        new Response(
          JSON.stringify(body),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    const ready = await waitForDeviceActivation({
      ...waitArgs,
      pollIntervalMs: 0,
    });

    assertEquals(calls, 2);
    assert(urls.every((url) => url.endsWith("/bootstrap/device")));
    assertEquals(sessionKeys[0], sessionKeys[1]);
    assertEquals(ready.sessionIdentity.sessionKey, sessionKeys[0]);
    assertEquals(ready.bundle.state, "ready");
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("device activation wait honors retryAfterMs from activation_pending", async () => {
  const originalFetch = globalThis.fetch;
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(6));
  const waitArgs = bootstrapWaitArgs(identity);
  let calls = 0;

  try {
    globalThis.fetch =
      ((_input: URL | Request | string, _init?: RequestInit) => {
        calls += 1;
        const body = calls === 1
          ? activationPendingBody()
          : bootstrapReadyBody();
        return Promise.resolve(
          new Response(
            JSON.stringify(body),
            {
              status: 200,
              headers: { "Content-Type": "application/json" },
            },
          ),
        );
      }) as typeof fetch;

    const startedAt = Date.now();
    await waitForDeviceActivation({
      ...waitArgs,
      pollIntervalMs: 0,
    });
    assert(Date.now() - startedAt >= 5);
    assertEquals(calls, 2);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("device activation wait rejects non-success bootstrap responses", async () => {
  const originalFetch = globalThis.fetch;
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(7));
  const waitArgs = bootstrapWaitArgs(identity);

  try {
    globalThis.fetch = ((_input: URL | Request | string, _init?: RequestInit) =>
      Promise.resolve(
        new Response(
          JSON.stringify({
            error: { code: "contract_digest_not_allowed" },
          }),
          {
            status: 403,
            headers: { "Content-Type": "application/json" },
          },
        ),
      )) as typeof fetch;

    await assertRejects(
      () =>
        waitForDeviceActivation({
          ...waitArgs,
          pollIntervalMs: 0,
        }),
      Error,
      "Trellis HTTP 403: contract_digest_not_allowed",
    );
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("device activation wait retries transient fetch failures", async () => {
  const originalFetch = globalThis.fetch;
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(8));
  const waitArgs = bootstrapWaitArgs(identity);
  let calls = 0;

  try {
    globalThis.fetch = ((_: URL | Request | string, _init?: RequestInit) => {
      calls += 1;
      if (calls === 1) {
        return Promise.reject(new TypeError("connection refused"));
      }
      return Promise.resolve(
        new Response(
          JSON.stringify(bootstrapReadyBody()),
          {
            status: 200,
            headers: { "Content-Type": "application/json" },
          },
        ),
      );
    }) as typeof fetch;

    await waitForDeviceActivation({
      ...waitArgs,
      pollIntervalMs: 0,
    });

    assertEquals(calls, 2);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("device activation wait reports an explicit replacement review", async () => {
  const originalFetch = globalThis.fetch;
  const identity = await deriveDeviceIdentity(new Uint8Array(32).fill(10));
  const waitArgs = bootstrapWaitArgs(identity);
  let calls = 0;

  try {
    globalThis.fetch =
      ((_input: URL | Request | string, _init?: RequestInit) => {
        calls += 1;
        const serverNow = Date.now();
        return Promise.resolve(
          new Response(
            JSON.stringify({
              state: "activation_pending",
              serverNow,
              activation: {
                state: "pending",
                reviewId: calls === 1 ? "dar_123" : "dar_replacement",
                activationUrl: "https://trellis.example.com/devices/activate",
                expiresAt: serverNow + 1_000,
                retryAfterMs: 0,
              },
            }),
            { status: 200, headers: { "Content-Type": "application/json" } },
          ),
        );
      }) as typeof fetch;

    await assertRejects(
      () => waitForDeviceActivation({ ...waitArgs, pollIntervalMs: 0 }),
      Error,
      "device activation review expired",
    );
    assertEquals(calls, 2);
  } finally {
    globalThis.fetch = originalFetch;
  }
});

Deno.test("device activation client wrappers hide method strings", async () => {
  const calls: Array<
    { kind: "operation" | "request"; method: string; input: unknown }
  > = [];
  function operation(method: "Auth.DeviceUserAuthorities.Resolve") {
    return {
      input(input: AuthResolveDeviceUserAuthoritiesInput) {
        calls.push({ kind: "operation", method, input });
        return {
          start(): AsyncResult<
            AuthResolveDeviceUserAuthoritiesOperation,
            UnexpectedError
          > {
            return AsyncResult.ok({
              id: "op_123",
              service: "trellis",
              operation: method,
              get() {
                return AsyncResult.ok(
                  {
                    id: "op_123",
                    service: "trellis",
                    operation: method,
                    revision: 1,
                    state: "completed",
                    createdAt: "2026-04-08T11:55:00Z",
                    updatedAt: "2026-04-08T12:00:00Z",
                    completedAt: "2026-04-08T12:00:00Z",
                    output: {
                      status: "activated",
                      instanceId: "dev_123",
                      deploymentId: "reader.default",
                      activatedAt: "2026-04-08T12:00:00Z",
                    },
                  } satisfies OperationSnapshot<
                    unknown,
                    AuthResolveDeviceUserAuthoritiesOutput
                  >,
                );
              },
              wait() {
                return AsyncResult.ok(
                  {
                    id: "op_123",
                    service: "trellis",
                    operation: method,
                    revision: 1,
                    state: "completed",
                    createdAt: "2026-04-08T11:55:00Z",
                    updatedAt: "2026-04-08T12:00:00Z",
                    completedAt: "2026-04-08T12:00:00Z",
                    output: {
                      status: "activated",
                      instanceId: "dev_123",
                      deploymentId: "reader.default",
                      activatedAt: "2026-04-08T12:00:00Z",
                    },
                  } satisfies TerminalOperation<
                    AuthResolveDeviceUserAuthoritiesProgress,
                    AuthResolveDeviceUserAuthoritiesOutput
                  >,
                );
              },
              watch() {
                return AsyncResult.ok((async function* (): AsyncIterable<
                  OperationEvent<
                    AuthResolveDeviceUserAuthoritiesProgress,
                    AuthResolveDeviceUserAuthoritiesOutput
                  >
                > {
                  yield {
                    type: "progress" as const,
                    progress: {
                      status: "pending_review" as const,
                      reviewId: "dar_123",
                      instanceId: "dev_123",
                      deploymentId: "reader.default",
                      requestedAt: "2026-04-08T11:55:00Z",
                    },
                    snapshot: {
                      id: "op_123",
                      service: "trellis",
                      operation: method,
                      revision: 1,
                      state: "running" as const,
                      createdAt: "2026-04-08T11:55:00Z",
                      updatedAt: "2026-04-08T11:55:00Z",
                      progress: {
                        status: "pending_review" as const,
                        reviewId: "dar_123",
                        instanceId: "dev_123",
                        deploymentId: "reader.default",
                        requestedAt: "2026-04-08T11:55:00Z",
                      },
                    },
                  };
                })());
              },
              cancel() {
                return unsupportedActivationOperationControl();
              },
              signal(
                _signal: string,
                _input?: unknown,
              ): AsyncResult<
                OperationSignalAck<
                  AuthResolveDeviceUserAuthoritiesProgress,
                  AuthResolveDeviceUserAuthoritiesOutput
                >,
                UnexpectedError
              > {
                return unsupportedActivationOperationControl();
              },
            });
          },
        };
      },
    };
  }
  function request(
    method: "Auth.DeviceUserAuthorities.List",
    input: AuthDeviceUserAuthoritiesListInput,
    _opts?: unknown,
  ): AsyncResult<AuthDeviceUserAuthoritiesListOutput, UnexpectedError>;
  function request(
    method: "Auth.DeviceUserAuthorities.Revoke",
    input: AuthDeviceUserAuthoritiesRevokeInput,
    _opts?: unknown,
  ): AsyncResult<AuthDeviceUserAuthoritiesRevokeResponse, UnexpectedError>;
  function request(
    method: string,
    input: unknown,
  ): AsyncResult<unknown, UnexpectedError> {
    calls.push({ kind: "request", method, input });
    switch (method) {
      case "Auth.DeviceUserAuthorities.List":
        return AsyncResult.ok({
          entries: [],
          count: 0,
          offset: 0,
          limit: 50,
        });
      case "Auth.DeviceUserAuthorities.Revoke":
        return AsyncResult.ok({ success: true });
      default:
        throw new Error(`Unexpected method ${method}`);
    }
  }

  const transport: DeviceActivationTransport = {
    authDeviceUserAuthoritiesResolve: (input) =>
      operation("Auth.DeviceUserAuthorities.Resolve").input(input),
    authDeviceUserAuthoritiesList: (input) =>
      request("Auth.DeviceUserAuthorities.List", input),
    authDeviceUserAuthoritiesRevoke: (input) =>
      request("Auth.DeviceUserAuthorities.Revoke", input),
  };
  const client = createDeviceActivationClient(transport);

  const activation = await client.resolveDeviceUserAuthorities({
    confirmationCode: "ABCDEFGH",
    flowId: "flow_123",
  });
  assertEquals(activation.id, "op_123");

  const watch = await activation.watch().orThrow();
  const watchEvents = [] as Array<{
    type: string;
    progress?: AuthResolveDeviceUserAuthoritiesProgress;
  }>;
  for await (const event of watch) {
    watchEvents.push({
      type: event.type,
      ...(event.type === "progress" ? { progress: event.progress } : {}),
    });
  }
  assertEquals(watchEvents, [{
    type: "progress",
    progress: {
      status: "pending_review",
      reviewId: "dar_123",
      instanceId: "dev_123",
      deploymentId: "reader.default",
      requestedAt: "2026-04-08T11:55:00Z",
    },
  }]);

  const pendingStatus = await activation.wait().orThrow();
  if (pendingStatus.output?.status !== "activated") {
    throw new Error(
      `Expected activated output, received ${
        pendingStatus.output?.status ?? "missing"
      }`,
    );
  }
  assertEquals(pendingStatus.output, {
    status: "activated",
    instanceId: "dev_123",
    deploymentId: "reader.default",
    activatedAt: "2026-04-08T12:00:00Z",
  });
  assertEquals(
    await client.listDeviceActivations({ limit: 50 }),
    { entries: [], count: 0, offset: 0, limit: 50 },
  );
  assertEquals(
    await client.revokeDeviceActivation({ instanceId: "dev_123" }),
    { success: true },
  );
  assertEquals(calls.map((entry) => [entry.kind, entry.method]), [
    ["operation", "Auth.DeviceUserAuthorities.Resolve"],
    ["request", "Auth.DeviceUserAuthorities.List"],
    ["request", "Auth.DeviceUserAuthorities.Revoke"],
  ]);
});
