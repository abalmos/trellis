import { assert, assertEquals } from "@std/assert";
import {
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  TrellisClient,
} from "@qlever-llc/trellis";
import type { AuthDeploymentAuthorityGrantOverridesPutInput } from "@qlever-llc/trellis/sdk/auth";
import { caseScopedName } from "../_support/names.ts";
import {
  type LiveTrellisRuntime,
  liveTrellisTest,
  runtimeScopeForCase,
} from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.grant-overrides-bind-without-user-capability" as const;
const fixture = createAuthLocalLoginFixture(CASE_ID);
const username = caseScopedName("auth-grant-overrides-user", CASE_ID);
const password = `trellis-integration-${CASE_ID}-password-2026`;
type GrantOverride =
  AuthDeploymentAuthorityGrantOverridesPutInput["overrides"][number];

liveTrellisTest({
  name:
    "auth.grant-overrides-bind-without-user-capability binds through grant override without user capability",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.setupService(runtime, fixture.deploymentId);
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const user = await admin.authUsersCreate({
        username,
        name: "Grant Override Local Login User",
        email: `${username}@example.test`,
        active: true,
        capabilities: [],
        capabilityGroups: [],
      }).orThrow();
      const reset = await admin.authUsersPasswordResetCreate({
        userId: user.user.userId,
      }).orThrow();
      await completeLocalPasswordAccountFlow({
        trellisUrl: runtime.trellisUrl,
        flowId: reset.flowId,
        username,
        password,
      });

      const webRow: GrantOverride = {
        deploymentId: fixture.deploymentId,
        identityKind: "web",
        grantKind: "capability",
        contractId: fixture.clientContract.CONTRACT.id,
        origin: new URL(runtime.trellisUrl).origin,
        sessionPublicKey: null,
        capability: fixture.pingCapability,
        capabilityGroupKey: null,
      };
      const wrongOriginRow = { ...webRow, origin: "https://wrong.example" };
      await putAndAssert(admin, wrongOriginRow);
      await assertListed(admin, wrongOriginRow);
      await assertInsufficientConnect(runtime, username, password);

      await putAndAssert(admin, webRow);
      await assertListed(admin, webRow);
      const webClient = await connectPlainUser(runtime, {
        username,
        password,
      });
      try {
        const me = await webClient.authSessionsMe({}).orThrow();
        assertEquals(me.user?.capabilities, []);
        const ping = await webClient.authLoginPing({
          message: fixture.pingMessage,
        }).orThrow();
        assertEquals(ping, { message: fixture.pingMessage, accepted: true });
      } finally {
        await webClient.connection.close();
      }

      const { clientKey } = await fixture.setupClientRegistration(runtime);
      const sessionRow: GrantOverride = {
        deploymentId: fixture.deploymentId,
        identityKind: "session",
        grantKind: "capability",
        contractId: fixture.clientContract.CONTRACT.id,
        origin: null,
        sessionPublicKey: clientKey.sessionKey,
        capability: fixture.pingCapability,
        capabilityGroupKey: null,
      };
      await putAndAssert(admin, sessionRow);
      await assertListed(admin, sessionRow);
      await assertInsufficientConnect(runtime, username, password, {
        redirectTo: "/_trellis/test/client-auth",
      });

      const sessionClient = await connectPlainUser(runtime, {
        username,
        password,
        sessionKeySeed: clientKey.seed,
        redirectTo: "/_trellis/test/client-auth",
      });
      try {
        const ping = await sessionClient.authLoginPing({
          message: fixture.pingMessage,
        }).orThrow();
        assertEquals(ping, { message: fixture.pingMessage, accepted: true });
      } finally {
        await sessionClient.connection.close();
      }

      const removed = await admin.authDeploymentAuthorityGrantOverridesRemove({
          deploymentId: fixture.deploymentId,
          overrides: [sessionRow],
      }).orThrow();
      assertEquals(removed.grantOverrides, []);
      await assertListed(admin);
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
});

async function putAndAssert(
  admin: Awaited<ReturnType<typeof fixture.setupSessionAdmin>>,
  row: GrantOverride,
): Promise<void> {
  const put = await admin.authDeploymentAuthorityGrantOverridesPut({
    deploymentId: fixture.deploymentId,
    overrides: [row],
  }).orThrow();
  assertEquals(put.grantOverrides, [row]);
}

async function assertListed(
  admin: Awaited<ReturnType<typeof fixture.setupSessionAdmin>>,
  row?: GrantOverride,
): Promise<void> {
  const listed = await admin.authDeploymentAuthorityGrantOverridesList({
    limit: 500,
  }).orThrow();
  const rows = listed.entries.filter((entry) =>
    entry.deploymentId === fixture.deploymentId
  );
  assertEquals(rows, row ? [row] : []);
}

async function connectPlainUser(
  runtime: LiveTrellisRuntime,
  args: {
    username: string;
    password: string;
    sessionKeySeed?: string;
    redirectTo?: string;
  },
) {
  const sessionKeySeed = args.sessionKeySeed ??
    (await fixture.setupClientRegistration(runtime)).clientKey.seed;
  const trellisUrl = runtime.trellisUrl;
  const auth = {
    mode: "session_key" as const,
    sessionKeySeed,
    redirectTo: args.redirectTo ?? `${trellisUrl}/_trellis/test/client-auth`,
  };
  return await TrellisClient.connect({
    trellisUrl,
    name: fixture.clientName,
    contract: fixture.clientContract,
    auth,
    onAuthRequired: async (ctx) =>
      await completeLocalLoginFlow({
        trellisUrl,
        ctx,
        expectedStatus: "redirect",
        ...args,
      }),
  }).orThrow();
}

async function assertInsufficientConnect(
  runtime: LiveTrellisRuntime,
  username: string,
  password: string,
  options: { redirectTo?: string } = {},
): Promise<void> {
  const { clientKey } = await fixture.setupClientRegistration(runtime);
  const trellisUrl = runtime.trellisUrl;
  const result = await TrellisClient.connect({
    trellisUrl,
    name: fixture.clientName,
    contract: fixture.clientContract,
    auth: {
      mode: "session_key",
      sessionKeySeed: clientKey.seed,
      redirectTo: options.redirectTo ??
        `${trellisUrl}/_trellis/test/client-auth`,
    },
    onAuthRequired: async (ctx) =>
      await completeLocalLoginFlow({
        trellisUrl,
        username,
        password,
        ctx,
        expectedStatus: "insufficient_capabilities",
      }),
  });
  assert(result.isErr());
  assertEquals(
    Reflect.get(result.error, "code"),
    "trellis.auth.insufficient_capabilities",
  );
}

async function completeLocalPasswordAccountFlow(args: {
  trellisUrl: string;
  flowId: string;
  username: string;
  password: string;
}): Promise<void> {
  const response = await fetch(
    `${args.trellisUrl}/auth/account-flow/${
      encodeURIComponent(args.flowId)
    }/local-password`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        username: args.username,
        password: args.password,
      }),
    },
  );
  const body = await response.text();
  assertEquals(response.status, 200, body);
  const payload: unknown = JSON.parse(body);
  assert(isRecord(payload), "expected account-flow completion response object");
  assertEquals(payload.status, "created");
}

async function completeLocalLoginFlow(args: {
  trellisUrl: string;
  username: string;
  password: string;
  ctx: ClientAuthRequiredContext;
  expectedStatus: "redirect" | "insufficient_capabilities";
}): Promise<ClientAuthContinuation> {
  const flowId = flowIdFromUrl(args.ctx.loginUrl);
  const loginResponse = await fetch(`${args.trellisUrl}/auth/login/local`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      flowId,
      username: args.username,
      password: args.password,
    }),
  });
  const body = await loginResponse.text();
  assertEquals(loginResponse.status, 200, body);
  const state = await fetchJson(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  assert(isRecord(state), "expected portal flow state response object");
  assertEquals(state.status, args.expectedStatus, JSON.stringify(state));
  return { status: "bound", flowId };
}

async function fetchJson(url: string): Promise<unknown> {
  const response = await fetch(url);
  const body = await response.text();
  assertEquals(response.status, 200, body);
  return JSON.parse(body);
}

function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}
