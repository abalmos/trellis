import { assert, assertEquals, assertRejects } from "@std/assert";
import {
  AuthError,
  type ClientAuthContinuation,
  type ClientAuthRequiredContext,
  type ConnectedTrellisClient,
  TrellisClient,
} from "@qlever-llc/trellis";
import { isErr } from "@qlever-llc/result";
import { caseScopedName } from "../_support/names.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.users-identities-admin-surfaces-page-and-scope";
const fixture = createAuthLocalLoginFixture(CASE_ID);
const targetUsername = caseScopedName("auth-users-identities-target", CASE_ID);
const observerUsername = caseScopedName(
  "auth-users-identities-observer",
  CASE_ID,
);
const observerPassword =
  `trellis-integration-${CASE_ID}-observer-password-2026`;

type SessionAdminAppClient = ConnectedTrellisClient<
  typeof fixture.sessionAdminContract
>;
type ControlPlaneSqlite = NonNullable<
  LiveTrellisRuntime["controlPlane"]
>["sqlite"];

liveTrellisTest({
  name:
    "auth.users-identities-admin-surfaces-page-and-scope exercises user and identity admin RPC surfaces",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = requireControlPlaneSqlite(runtime);
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);

    try {
      const target = await admin.rpc.auth.usersCreate({
        username: targetUsername,
        name: "Users Identities Target",
        email: `${targetUsername}@example.test`,
        active: true,
      }).orThrow();
      const observerUser = await admin.rpc.auth.usersCreate({
        username: observerUsername,
        name: "Users Identities Observer",
        email: `${observerUsername}@example.test`,
        active: true,
      }).orThrow();
      const reset = await admin.rpc.auth.usersPasswordResetCreate({
        userId: observerUser.user.userId,
      }).orThrow();
      await completeLocalPasswordAccountFlow({
        trellisUrl: runtime.trellisUrl,
        flowId: reset.flowId,
        username: observerUsername,
        password: observerPassword,
      });
      await seedExtraIdentity(sqlite, target.user.userId);

      const firstPage = await admin.rpc.auth.usersList({ limit: 1 }).orThrow();
      assert(firstPage.count >= 3, "expected bootstrap and created users");
      assertEquals(firstPage.offset, 0);
      assertEquals(firstPage.limit, 1);
      assertEquals(firstPage.entries.length, 1);
      assertEquals(firstPage.nextOffset, 1);
      const repeatedFirstPage = await admin.rpc.auth.usersList({ limit: 1 })
        .orThrow();
      assertEquals(repeatedFirstPage.entries[0], firstPage.entries[0]);
      const secondPage = await admin.rpc.auth.usersList({ offset: 1, limit: 1 })
        .orThrow();
      assertEquals(secondPage.offset, 1);
      assertEquals(secondPage.limit, 1);
      assertEquals(secondPage.entries.length, 1);
      assert(
        firstPage.entries[0].userId.localeCompare(
          secondPage.entries[0].userId,
        ) <
          0,
        "expected Auth.Users.List pages to preserve stable userId ordering",
      );

      const got = await admin.rpc.auth.usersGet({
        userId: target.user.userId,
      }).orThrow();
      assertEquals(got.user.userId, target.user.userId);
      assert(got.user.identities.length >= 2, "expected target identities");

      const identities = await admin.rpc.auth.userIdentitiesList({
        userId: target.user.userId,
        limit: 1,
      }).orThrow();
      assertEquals(identities.count, got.user.identities.length);
      assertEquals(identities.offset, 0);
      assertEquals(identities.limit, 1);
      assertEquals(identities.entries.length, 1);
      assertEquals(identities.nextOffset, 1);
      assertEquals(
        identities.entries[0]?.identityId,
        got.user.identities[0]?.identityId,
      );
      const nextIdentities = await admin.rpc.auth.userIdentitiesList({
        userId: target.user.userId,
        offset: 1,
        limit: 1,
      }).orThrow();
      assertEquals(nextIdentities.count, got.user.identities.length);
      assertEquals(nextIdentities.offset, 1);
      assertEquals(nextIdentities.limit, 1);
      assertEquals(nextIdentities.entries.length, 1);
      assertEquals(
        nextIdentities.entries[0]?.identityId,
        got.user.identities[1]?.identityId,
      );

      await assertAuthErrorReason(
        admin.rpc.auth.userIdentitiesUnlink({
          userId: target.user.userId,
          identityId: `${target.user.userId}:missing`,
        }),
        "identity_not_found",
      );

      let missingCapabilities: string[] = [];
      await assertRejects(
        () =>
          connectObserver(runtime, (missing) => {
            missingCapabilities = missing;
          }),
        Error,
      );
      assert(
        missingCapabilities.includes("admin"),
        "expected non-admin observer to be denied admin-scoped user inspection",
      );
    } finally {
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

async function seedExtraIdentity(
  sqlite: ControlPlaneSqlite,
  userId: string,
): Promise<void> {
  const identityId = `idn_extra_${userId}`;
  await sqlite.execute(
    `INSERT INTO user_identities
      (id, identity_id, user_id, provider, subject, display_name, email, email_verified, linked_at, last_login_at)
      VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)`,
    [
      identityId,
      identityId,
      userId,
      "integration",
      userId,
      "Extra Identity",
      `${userId}@extra.example.test`,
      1,
      new Date(0).toISOString(),
      null,
    ],
  );
}

async function connectObserver(
  runtime: Parameters<typeof fixture.setupSessionAdmin>[0],
  onMissingCapabilities: (capabilities: string[]) => void,
): Promise<SessionAdminAppClient> {
  const registration = await runtime.registerClient({
    name: caseScopedName("auth-users-identities-observer-client", CASE_ID),
    contract: fixture.sessionAdminContract,
  });
  const auth = runtime.clientAuth(registration);
  return await TrellisClient.connect({
    trellisUrl: runtime.trellisUrl,
    name: caseScopedName("auth-users-identities-observer-client", CASE_ID),
    contract: fixture.sessionAdminContract,
    auth: auth.auth,
    onAuthRequired: async (ctx) => {
      return await completeLocalLoginFlow({
        trellisUrl: runtime.trellisUrl,
        ctx,
        onMissingCapabilities,
        username: observerUsername,
        password: observerPassword,
      });
    },
  }).orThrow();
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
}

async function completeLocalLoginFlow(args: {
  trellisUrl: string;
  ctx: ClientAuthRequiredContext;
  onMissingCapabilities: (capabilities: string[]) => void;
  username: string;
  password: string;
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
  if (!loginResponse.ok) {
    throw new Error(`local login failed (${loginResponse.status})`);
  }

  const state = await fetchJson(
    `${args.trellisUrl}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  assert(isRecord(state), "expected portal flow state response object");
  if (state.status === "insufficient_capabilities") {
    const missingCapabilities = stringArray(state.missingCapabilities);
    args.onMissingCapabilities(missingCapabilities);
    throw new Error(
      `observer local login was denied for missing capabilities: ${
        missingCapabilities.join(", ")
      }`,
    );
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

async function assertAuthErrorReason(
  result: { take(): Promise<unknown> },
  reason: AuthError["reason"],
): Promise<void> {
  const value = await result.take();
  assert(isErr(value));
  assert(value.error instanceof AuthError, "expected AuthError");
  assertEquals(value.error.reason, reason);
}

function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

async function fetchJson(url: string, init?: RequestInit): Promise<unknown> {
  const response = await fetch(url, init);
  if (!response.ok) {
    throw new Error(`HTTP request failed (${response.status}) for ${url}`);
  }
  return await response.json();
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function stringArray(value: unknown): string[] {
  assert(Array.isArray(value), "expected capability array");
  for (const entry of value) {
    assert(typeof entry === "string", "expected capability string");
  }
  return value;
}
