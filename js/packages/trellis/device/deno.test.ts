import { assertEquals } from "@std/assert";

import { defineDeviceContract } from "../contract.ts";
import { checkDeviceActivation } from "./deno.ts";

const deviceContract = defineDeviceContract(() => ({
  id: "example.device@v1",
  displayName: "Example Device",
  description: "Test device contract.",
}));
const rootSecret = new Uint8Array(32).fill(1);
const identity = {
  deploymentId: "reader.default",
  instanceId: "dev_123",
  principalId: "device_123",
  participantId: deviceContract.CONTRACT_ID,
  participantArtifactDigest: deviceContract.CONTRACT_DIGEST,
  participantNeedsDigest: deviceContract.CONTRACT_DIGEST,
};

Deno.test("checkDeviceActivation persists provisioned activation state", async () => {
  const stateDir = await Deno.makeTempDir({ prefix: "trellis-device-state-" });
  const originalFetch = globalThis.fetch;
  let bootstrapCalls = 0;
  try {
    globalThis.fetch = ((input: URL | Request | string) => {
      assertEquals(
        String(input),
        "https://trellis.example.com/bootstrap/device",
      );
      bootstrapCalls += 1;
      return Promise.resolve(Response.json({
        state: "activation_pending",
        serverNow: Date.now(),
        activation: {
          reviewId: "review_123",
          activationUrl:
            "https://trellis.example.com/_trellis/portal/devices/activate?flowId=review_123",
        },
      }));
    }) as typeof fetch;

    const status = await checkDeviceActivation({
      trellisUrl: "https://trellis.example.com",
      contract: deviceContract,
      identity,
      rootSecret,
      stateDir,
    });

    assertEquals(status.status, "activation_required");
    assertEquals(bootstrapCalls, 1);
    assertEquals((await Array.fromAsync(Deno.readDir(stateDir))).length, 1);
  } finally {
    globalThis.fetch = originalFetch;
    await Deno.remove(stateDir, { recursive: true });
  }
});

Deno.test("checkDeviceActivation reports an already active device", async () => {
  const stateDir = await Deno.makeTempDir({ prefix: "trellis-device-state-" });
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = (() =>
      Promise.resolve(Response.json({
        state: "ready",
        serverNow: Date.now(),
        session: {
          sessionId: "session_123",
          inboxPrefix: "_INBOX.session_123",
        },
        authorization: {
          participantArtifactDigest: deviceContract.CONTRACT_DIGEST,
        },
        nats: { jwt: "jwt", servers: ["nats://127.0.0.1:4222"] },
      }))) as typeof fetch;

    assertEquals(
      await checkDeviceActivation({
        trellisUrl: "https://trellis.example.com",
        contract: deviceContract,
        identity,
        rootSecret,
        stateDir,
      }),
      { status: "activated" },
    );
  } finally {
    globalThis.fetch = originalFetch;
    await Deno.remove(stateDir, { recursive: true });
  }
});

Deno.test("checkDeviceActivation preserves rejected activation state", async () => {
  const stateDir = await Deno.makeTempDir({ prefix: "trellis-device-state-" });
  const originalFetch = globalThis.fetch;
  try {
    globalThis.fetch = (() =>
      Promise.resolve(Response.json({
        state: "activation_rejected",
        serverNow: Date.now(),
      }))) as typeof fetch;

    assertEquals(
      await checkDeviceActivation({
        trellisUrl: "https://trellis.example.com",
        contract: deviceContract,
        identity,
        rootSecret,
        stateDir,
      }),
      { status: "not_ready", reason: "activation_rejected" },
    );
  } finally {
    globalThis.fetch = originalFetch;
    await Deno.remove(stateDir, { recursive: true });
  }
});
