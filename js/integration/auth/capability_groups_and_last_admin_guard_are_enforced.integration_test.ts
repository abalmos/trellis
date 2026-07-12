import { assert, assertArrayIncludes, assertEquals } from "@std/assert";
import { AuthError } from "@qlever-llc/trellis";
import { isErr } from "@qlever-llc/result";
import { caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.capability-groups-and-last-admin-guard-are-enforced";
const fixture = createAuthLocalLoginFixture(CASE_ID);

liveTrellisTest({
  name:
    "auth.capability-groups-and-last-admin-guard-are-enforced validates capability groups and last-admin guardrails",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const groupKey = caseScopedName("auth-capability-group", CASE_ID);
    try {
      const me = await admin.authSessionsMe({}).orThrow();
      assert(me.user, "expected admin session user");
      const adminUserId = me.user.userId;

      const put = await admin.authCapabilityGroupsPut({
        groupKey,
        displayName: "Integration Capability Group",
        description: "Grants the auth local-login fixture capability.",
        capabilities: [fixture.pingCapability],
      }).orThrow();
      assertEquals(put.group.groupKey, groupKey);
      assertArrayIncludes(put.group.capabilities, [fixture.pingCapability]);

      const listed = await admin.authCapabilityGroupsList({ limit: 500 })
        .orThrow();
      assert(
        listed.entries.some((group) => group.groupKey === groupKey),
        "expected custom capability group in list",
      );
      const got = await admin.authCapabilityGroupsGet({ groupKey })
        .orThrow();
      assertEquals(got.group.groupKey, groupKey);

      await admin.authUsersUpdate({
        userId: adminUserId,
        capabilities: ["admin"],
        capabilityGroups: [groupKey],
      }).orThrow();
      const groupedMe = await admin.authSessionsMe({}).orThrow();
      assert(groupedMe.user, "expected grouped admin session user");
      assertArrayIncludes(groupedMe.user.capabilities, [
        fixture.pingCapability,
      ]);

      await assertAuthErrorReason(
        admin.authCapabilityGroupsPut({
          groupKey: `${groupKey}.invalid`,
          displayName: "Invalid Capability Group",
          description: "References an unknown capability.",
          capabilities: [`${fixture.pingCapability}.unknown`],
        }),
        "invalid_request",
      );
      await assertAuthErrorReason(
        admin.authCapabilityGroupsPut({
          groupKey: "admin",
          displayName: "Blocked Admin Group",
          description: "Built-in group mutation must fail.",
        }),
        "invalid_request",
      );
      await assertAuthErrorReason(
        admin.authCapabilityGroupsDelete({ groupKey: "admin" }),
        "invalid_request",
      );

      await assertAuthErrorReason(
        admin.authUsersUpdate({ userId: adminUserId, active: false }),
        "last_admin_required",
      );
      await assertAuthErrorReason(
        admin.authUsersUpdate({
          userId: adminUserId,
          capabilities: [],
          capabilityGroups: [groupKey],
        }),
        "last_admin_required",
      );

      const adminUser = await admin.authUsersGet({ userId: adminUserId })
        .orThrow();
      assert(adminUser.user.identities.length > 0, "expected admin identity");
      await assertAuthErrorReason(
        admin.authUserIdentitiesUnlink({
          userId: adminUserId,
          identityId: adminUser.user.identities[0].identityId,
        }),
        "last_admin_required",
      );

      await admin.authUsersCreate({
        username: caseScopedName("auth-capability-second-admin", CASE_ID),
        name: "Capability Group Second Admin",
        email: "capability-group-second-admin@example.test",
        active: true,
        capabilityGroups: ["admin"],
      }).orThrow();
      const permitted = await admin.authUsersUpdate({
        userId: adminUserId,
        capabilities: [],
        capabilityGroups: [groupKey],
      }).orThrow();
      assertEquals(permitted.success, true);
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
});

async function assertAuthErrorReason(
  result: { take(): Promise<unknown> },
  reason: AuthError["reason"],
): Promise<void> {
  const value = await result.take();
  assert(isErr(value));
  assert(value.error instanceof AuthError, "expected AuthError");
  assertEquals(value.error.reason, reason);
}
