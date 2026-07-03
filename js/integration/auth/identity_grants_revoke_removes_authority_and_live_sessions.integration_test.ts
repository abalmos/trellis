import { assert, assertEquals } from "@std/assert";
import {
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  type ConnectedTrellisClient,
  TrellisClient,
} from "@qlever-llc/trellis";
import type { AuthIdentityGrantsListOutput } from "@qlever-llc/trellis/sdk/auth";
import { waitFor } from "@qlever-llc/trellis-test";
import { caseScopedName } from "../_support/names.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID =
  "auth.identity-grants-revoke-removes-authority-and-live-sessions" as const;
const fixture = createAuthLocalLoginFixture(CASE_ID);
const nonOwnerUsername = caseScopedName(
  "auth-identity-grants-non-owner",
  CASE_ID,
);
const nonOwnerPassword =
  `trellis-integration-${CASE_ID}-non-owner-password-2026`;
type SessionAdminClient = Awaited<ReturnType<typeof fixture.setupSessionAdmin>>;
type ControlPlaneSqlite = NonNullable<
  LiveTrellisRuntime["controlPlane"]
>["sqlite"];
type IdentityGrant = AuthIdentityGrantsListOutput["entries"][number];

liveTrellisTest({
  name:
    "auth.identity-grants-revoke-removes-authority-and-live-sessions revokes grant sessions and denies old calls",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = requireControlPlaneSqlite(runtime);
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const firstRegistration = await fixture.setupClientRegistration(runtime);
    const secondRegistration = await runtime.registerClient({
      name: `${fixture.clientName}-sibling`,
      contract: fixture.clientContract,
    });
    const firstAgentRegistration = await runtime.registerClient({
      name: `${fixture.clientName}-agent`,
      contract: fixture.agentContract,
    });
    const secondAgentRegistration = await runtime.registerClient({
      name: `${fixture.clientName}-agent-sibling`,
      contract: fixture.agentContract,
    });
    const secondAuth = runtime.clientAuth(secondRegistration);
    const firstAgentAuth = runtime.clientAuth(firstAgentRegistration);
    const secondAgentAuth = runtime.clientAuth(secondAgentRegistration);
    let firstApp:
      | ConnectedTrellisClient<typeof fixture.clientContract>
      | undefined;
    let secondApp:
      | ConnectedTrellisClient<typeof fixture.clientContract>
      | undefined;
    let firstAgent:
      | ConnectedTrellisClient<typeof fixture.agentContract>
      | undefined;
    let secondAgent:
      | ConnectedTrellisClient<typeof fixture.agentContract>
      | undefined;
    let nonOwner:
      | ConnectedTrellisClient<typeof fixture.sessionAdminContract>
      | undefined;

    try {
      firstApp = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: firstRegistration.clientAuth.auth,
        onAuthRequired: async (ctx) =>
          await firstRegistration.clientAuth.onAuthRequired(ctx),
      }).orThrow();
      secondApp = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: `${fixture.clientName}-sibling`,
        contract: fixture.clientContract,
        auth: secondAuth.auth,
        onAuthRequired: async (ctx) => await secondAuth.onAuthRequired(ctx),
      }).orThrow();
      firstAgent = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: `${fixture.clientName}-agent`,
        contract: fixture.agentContract,
        auth: firstAgentAuth.auth,
        onAuthRequired: async (ctx) => await firstAgentAuth.onAuthRequired(ctx),
      }).orThrow();
      secondAgent = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: `${fixture.clientName}-agent-sibling`,
        contract: fixture.agentContract,
        auth: secondAgentAuth.auth,
        onAuthRequired: async (ctx) =>
          await secondAgentAuth.onAuthRequired(ctx),
      }).orThrow();

      await firstApp.rpc.authLogin.ping({ message: `${fixture.pingMessage}-1` })
        .orThrow();
      await secondApp.rpc.authLogin.ping({
        message: `${fixture.pingMessage}-2`,
      })
        .orThrow();
      await firstAgent.rpc.authLogin.ping({
        message: `${fixture.pingMessage}-agent-1`,
      })
        .orThrow();
      await secondAgent.rpc.authLogin.ping({
        message: `${fixture.pingMessage}-agent-2`,
      })
        .orThrow();
      await userSessionFor(
        admin,
        firstRegistration.clientKey.sessionKey,
        "app",
      );
      await userSessionFor(admin, secondRegistration.sessionKey, "app");
      await userSessionFor(admin, firstAgentRegistration.sessionKey, "agent");
      await userSessionFor(admin, secondAgentRegistration.sessionKey, "agent");
      await waitForConnection(admin, firstRegistration.clientKey.sessionKey);
      await waitForConnection(admin, secondRegistration.sessionKey);
      await waitForConnection(admin, firstAgentRegistration.sessionKey);
      await waitForConnection(admin, secondAgentRegistration.sessionKey);

      const appIdentityGrantId = await sharedIdentityGrantId(
        sqlite,
        firstRegistration.clientKey.sessionKey,
        secondRegistration.sessionKey,
      );
      const agentIdentityGrantId = await sharedIdentityGrantId(
        sqlite,
        firstAgentRegistration.sessionKey,
        secondAgentRegistration.sessionKey,
      );
      assert(await grantExists(sqlite, appIdentityGrantId));
      assert(await grantExists(sqlite, agentIdentityGrantId));
      assertGrantListed(
        await listGrant(admin, appIdentityGrantId),
        appIdentityGrantId,
      );
      assertGrantListed(
        await listGrant(admin, agentIdentityGrantId),
        agentIdentityGrantId,
      );

      nonOwner = await connectNonOwnerAdmin(runtime, admin);
      const denied = await nonOwner.rpc.auth.identityGrantsRevoke({
        identityGrantId: appIdentityGrantId,
      });
      assert(denied.isErr(), "expected non-owner grant revoke to be denied");

      const appRevoked = await admin.rpc.auth.identityGrantsRevoke({
        identityGrantId: appIdentityGrantId,
      }).orThrow();
      assertEquals(appRevoked.success, true);
      const agentRevoked = await admin.rpc.auth.identityGrantsRevoke({
        identityGrantId: agentIdentityGrantId,
      }).orThrow();
      assertEquals(agentRevoked.success, true);

      await waitForSessionAbsent(admin, firstRegistration.clientKey.sessionKey);
      await waitForSessionAbsent(admin, secondRegistration.sessionKey);
      await waitForSessionAbsent(admin, firstAgentRegistration.sessionKey);
      await waitForSessionAbsent(admin, secondAgentRegistration.sessionKey);
      await waitForConnectionsAbsent(
        admin,
        firstRegistration.clientKey.sessionKey,
      );
      await waitForConnectionsAbsent(admin, secondRegistration.sessionKey);
      await waitForConnectionsAbsent(admin, firstAgentRegistration.sessionKey);
      await waitForConnectionsAbsent(admin, secondAgentRegistration.sessionKey);
      await waitFor(async () =>
        !(await grantExists(sqlite, appIdentityGrantId)) &&
        !(await grantExists(sqlite, agentIdentityGrantId))
      );
      await waitFor(async () =>
        await listGrant(admin, appIdentityGrantId) === undefined &&
        await listGrant(admin, agentIdentityGrantId) === undefined
      );
      await waitFor(async () =>
        (await firstApp!.rpc.auth.sessionsMe({})).isErr() &&
        (await secondApp!.rpc.auth.sessionsMe({})).isErr() &&
        (await firstAgent!.rpc.auth.sessionsMe({})).isErr() &&
        (await secondAgent!.rpc.auth.sessionsMe({})).isErr()
      );
    } finally {
      await nonOwner?.connection.close().catch(() => undefined);
      await firstApp?.connection.close().catch(() => undefined);
      await secondApp?.connection.close().catch(() => undefined);
      await firstAgent?.connection.close().catch(() => undefined);
      await secondAgent?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
});

function requireControlPlaneSqlite(
  runtime: LiveTrellisRuntime,
): ControlPlaneSqlite {
  const sqlite = runtime.controlPlane?.sqlite;
  assert(sqlite, "live runtime must expose control-plane SQLite");
  return sqlite;
}

async function connectNonOwnerAdmin(
  runtime: LiveTrellisRuntime,
  admin: SessionAdminClient,
) {
  const user = await admin.rpc.auth.usersCreate({
    username: nonOwnerUsername,
    name: "Identity Grant Revoke Non Owner",
    email: `${nonOwnerUsername}@example.test`,
    active: true,
    capabilityGroups: ["admin"],
  }).orThrow();
  const reset = await admin.rpc.auth.usersPasswordResetCreate({
    userId: user.user.userId,
  }).orThrow();
  await completeLocalPasswordAccountFlow({
    trellisUrl: runtime.trellisUrl,
    flowId: reset.flowId,
    username: nonOwnerUsername,
    password: nonOwnerPassword,
  });

  const name = caseScopedName("auth-identity-grants-non-owner-client", CASE_ID);
  const key = await runtime.registerClient({
    name,
    contract: fixture.sessionAdminContract,
  });
  return await TrellisClient.connect({
    trellisUrl: runtime.trellisUrl,
    name,
    contract: fixture.sessionAdminContract,
    auth: {
      mode: "session_key",
      sessionKeySeed: key.seed,
      redirectTo: `${runtime.trellisUrl}/_trellis/test/non-owner-auth`,
    },
    onAuthRequired: async (ctx) =>
      await completeLocalLoginFlow({
        admin,
        trellisUrl: runtime.trellisUrl,
        userId: user.user.userId,
        username: nonOwnerUsername,
        password: nonOwnerPassword,
        ctx,
      }),
  }).orThrow();
}

async function userSessionFor(
  admin: SessionAdminClient,
  sessionKey: string,
  participantKind: "app" | "agent",
) {
  const sessions = await admin.rpc.auth.sessionsList({ limit: 500 }).orThrow();
  const session = sessions.entries.find((entry) =>
    entry.participantKind === participantKind && entry.sessionKey === sessionKey
  );
  assert(
    session,
    `expected Auth.Sessions.List to include ${participantKind} session`,
  );
  return session;
}

async function waitForSessionAbsent(
  admin: SessionAdminClient,
  sessionKey: string,
) {
  await waitFor(async () => {
    const sessions = await admin.rpc.auth.sessionsList({ limit: 500 })
      .orThrow();
    return sessions.entries.every((entry) => entry.sessionKey !== sessionKey);
  });
}

async function waitForConnection(
  admin: SessionAdminClient,
  sessionKey: string,
) {
  await waitFor(async () => {
    const connections = await admin.rpc.auth.connectionsList({
      sessionKey,
      limit: 500,
    }).orThrow();
    return connections.entries.length > 0;
  });
}

async function waitForConnectionsAbsent(
  admin: SessionAdminClient,
  sessionKey: string,
) {
  await waitFor(async () => {
    const connections = await admin.rpc.auth.connectionsList({
      sessionKey,
      limit: 500,
    }).orThrow();
    return connections.entries.length === 0;
  });
}

async function listGrant(
  admin: SessionAdminClient,
  identityGrantId: string,
): Promise<IdentityGrant | undefined> {
  const grants = await admin.rpc.auth.identityGrantsList({ limit: 500 })
    .orThrow();
  return grants.entries.find((entry) =>
    entry.identityGrantId === identityGrantId
  );
}

function assertGrantListed(
  grant: IdentityGrant | undefined,
  identityGrantId: string,
) {
  assert(grant, "expected Auth.IdentityGrants.List to include grant");
  assertEquals(grant.identityGrantId, identityGrantId);
}

async function sharedIdentityGrantId(
  sqlite: ControlPlaneSqlite,
  firstSessionKey: string,
  secondSessionKey: string,
): Promise<string> {
  const rows = await sqlite.query(
    "SELECT DISTINCT identity_grant_id AS identityGrantId FROM sessions WHERE session_key IN (?, ?)",
    [firstSessionKey, secondSessionKey],
  );
  assertEquals(rows.length, 1);
  const identityGrantId = rows[0]?.identityGrantId;
  assert(typeof identityGrantId === "string" && identityGrantId.length > 0);
  return identityGrantId;
}

async function grantExists(
  sqlite: ControlPlaneSqlite,
  identityGrantId: string,
): Promise<boolean> {
  const rows = await sqlite.query(
    "SELECT identity_grant_id FROM identity_grants WHERE identity_grant_id = ?",
    [identityGrantId],
  );
  return rows.length > 0;
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
  admin: SessionAdminClient;
  trellisUrl: string;
  userId: string;
  username: string;
  password: string;
  ctx: ClientAuthRequiredContext;
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

  let state = await fetchJson(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  assert(isRecord(state), "expected portal flow state response object");
  if (state.status === "insufficient_capabilities") {
    const missingCapabilities = stringArray(state.missingCapabilities);
    await args.admin.rpc.auth.usersUpdate({
      userId: args.userId,
      capabilities: [...new Set(["admin", ...missingCapabilities])].sort(),
    }).orThrow();
    state = await fetchJson(
      `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
    );
    assert(isRecord(state), "expected portal flow state response object");
  }
  if (state.status === "approval_required") {
    const approved = await fetchJson(
      `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}/approval`,
      {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify({ approved: true }),
      },
    );
    assert(isRecord(approved), "expected portal approval response object");
    assertEquals(approved.status, "redirect");
  } else {
    assertEquals(state.status, "redirect");
  }

  return { status: "bound", flowId };
}

async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const response = await fetch(url, init);
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

function stringArray(value: unknown): string[] {
  assert(Array.isArray(value), "expected missingCapabilities array");
  for (const entry of value) {
    assert(typeof entry === "string", "expected capability string");
  }
  return value;
}
