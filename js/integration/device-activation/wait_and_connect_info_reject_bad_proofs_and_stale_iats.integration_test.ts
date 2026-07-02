import { assert, assertEquals } from "@std/assert";
import { join } from "@std/path";
import { Kvm } from "@nats-io/kv";
import { connect, credsAuthenticator } from "@nats-io/transport-deno";
import {
  deriveDeviceIdentity,
  signDeviceWaitRequest,
} from "@qlever-llc/trellis/auth";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createDeviceActivationFixture } from "./_fixture.ts";

const CASE_ID =
  "device-activation.wait-and-connect-info-reject-bad-proofs-and-stale-iats" as const;
const fixture = createDeviceActivationFixture(CASE_ID);

liveTrellisTest({
  name:
    "device-activation.wait-and-connect-info-reject-bad-proofs-and-stale-iats rejects bad pre-auth proofs",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
    );
    const { identity } = await fixture.setupProvisionedDevice(
      admin,
      deploymentId,
    );
    const { nonce, flowId } = await fixture.setupActivationRequest(
      runtime,
      identity,
    );
    const digest = fixture.deviceContract.CONTRACT_DIGEST;
    const staleIat = Math.floor(Date.now() / 1_000) - 3_600;

    const staleWait = await postWait(runtime.trellisUrl, {
      ...(await signDeviceWaitRequest({
        flowId,
        publicIdentityKey: identity.publicIdentityKey,
        nonce,
        identitySeed: identity.identitySeed,
        contractDigest: digest,
        iat: staleIat,
      })),
    });
    assertReason(staleWait, 400, "iat_out_of_range");
    assert(
      typeof staleWait.body.serverNow === "number",
      "stale wait proof should return serverNow",
    );

    const signedWait = await signDeviceWaitRequest({
      flowId,
      publicIdentityKey: identity.publicIdentityKey,
      nonce,
      identitySeed: identity.identitySeed,
      contractDigest: digest,
    });
    assertReason(
      await postWait(runtime.trellisUrl, {
        ...signedWait,
        sig: corruptSignature(signedWait.sig),
      }),
      400,
      "invalid_signature",
    );

    assertReason(
      await postWait(
        runtime.trellisUrl,
        await signDeviceWaitRequest({
          flowId,
          publicIdentityKey: identity.publicIdentityKey,
          nonce: `${nonce}-wrong`,
          identitySeed: identity.identitySeed,
          contractDigest: digest,
        }),
      ),
      400,
      "invalid_request",
    );

    const wrongIdentity = await deriveDeviceIdentity(
      crypto.getRandomValues(new Uint8Array(32)),
    );
    assertReason(
      await postWait(
        runtime.trellisUrl,
        await signDeviceWaitRequest({
          flowId,
          publicIdentityKey: wrongIdentity.publicIdentityKey,
          nonce,
          identitySeed: wrongIdentity.identitySeed,
          contractDigest: digest,
        }),
      ),
      400,
      "invalid_request",
    );

    const missing = await postWait(
      runtime.trellisUrl,
      await signDeviceWaitRequest({
        flowId: crypto.randomUUID(),
        publicIdentityKey: identity.publicIdentityKey,
        nonce,
        identitySeed: identity.identitySeed,
        contractDigest: digest,
      }),
    );
    assertEquals(missing.status, 200);
    assertEquals(missing.body.status, "rejected");
    assertEquals(missing.body.reason, "device_activation_flow_not_found");

    await expireDeviceActivationFlow(runtime, flowId);
    const expired = await postWait(runtime.trellisUrl, signedWait);
    assertEquals(expired.status, 200);
    assertEquals(expired.body.status, "rejected");
    assertEquals(expired.body.reason, "device_flow_expired");

    const staleConnect = await postConnectInfo(runtime.trellisUrl, {
      publicIdentityKey: identity.publicIdentityKey,
      contractDigest: digest,
      iat: staleIat,
      sig: (await signDeviceWaitRequest({
        flowId: "connect-info",
        publicIdentityKey: identity.publicIdentityKey,
        nonce: "connect-info",
        identitySeed: identity.identitySeed,
        contractDigest: digest,
        iat: staleIat,
      })).sig,
    });
    assertReason(staleConnect, 400, "iat_out_of_range");
    assert(
      typeof staleConnect.body.serverNow === "number",
      "stale connect-info proof should return serverNow",
    );

    const signedConnect = await signDeviceWaitRequest({
      flowId: "connect-info",
      publicIdentityKey: identity.publicIdentityKey,
      nonce: "connect-info",
      identitySeed: identity.identitySeed,
      contractDigest: digest,
    });
    assertReason(
      await postConnectInfo(runtime.trellisUrl, {
        publicIdentityKey: identity.publicIdentityKey,
        contractDigest: digest,
        iat: signedConnect.iat,
        sig: corruptSignature(signedConnect.sig),
      }),
      400,
      "invalid_signature",
    );

    const unauthorizedDigest = "unauthorized_device_contract";
    const unauthorized = await signDeviceWaitRequest({
      flowId: "connect-info",
      publicIdentityKey: identity.publicIdentityKey,
      nonce: "connect-info",
      identitySeed: identity.identitySeed,
      contractDigest: unauthorizedDigest,
    });
    assertReason(
      await postConnectInfo(runtime.trellisUrl, {
        publicIdentityKey: identity.publicIdentityKey,
        contractDigest: unauthorizedDigest,
        iat: unauthorized.iat,
        sig: unauthorized.sig,
      }),
      403,
      "contract_digest_not_allowed",
    );
  },
});

type JsonResponse = {
  readonly status: number;
  readonly body: Record<string, unknown>;
};

async function postWait(
  trellisUrl: string,
  body: unknown,
): Promise<JsonResponse> {
  return await postJson(trellisUrl, "/auth/devices/activate/wait", body);
}

async function postConnectInfo(
  trellisUrl: string,
  body: unknown,
): Promise<JsonResponse> {
  return await postJson(trellisUrl, "/auth/devices/connect-info", body);
}

async function postJson(
  trellisUrl: string,
  path: string,
  body: unknown,
): Promise<JsonResponse> {
  const response = await fetch(new URL(path, trellisUrl), {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const json: unknown = await response.json();
  assert(isRecord(json), "expected object JSON response");
  return { status: response.status, body: json };
}

function assertReason(
  response: JsonResponse,
  status: number,
  reason: string,
): void {
  assertEquals(response.status, status);
  assertEquals(response.body.reason, reason);
}

function corruptSignature(sig: string): string {
  return `${sig.startsWith("A") ? "B" : "A"}${sig.slice(1)}`;
}

async function expireDeviceActivationFlow(
  runtime: LiveTrellisRuntime,
  flowId: string,
): Promise<void> {
  const nc = await connect({
    servers: runtime.natsUrl,
    authenticator: credsAuthenticator(
      await Deno.readFile(
        join(runtime.workdir, "nats", "creds", "auth-auth.creds"),
      ),
    ),
  });
  try {
    const kv = await new Kvm(nc).open("trellis_browser_flows");
    const entry = await kv.get(flowId);
    assert(entry, "device activation flow should exist in browser-flow KV");
    const value = entry.json();
    assert(isRecord(value), "device activation flow should be a JSON object");
    await kv.put(
      flowId,
      JSON.stringify({
        ...value,
        expiresAt: "2000-01-01T00:00:00.000Z",
      }),
    );
  } finally {
    await nc.close().catch(() => undefined);
  }
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
