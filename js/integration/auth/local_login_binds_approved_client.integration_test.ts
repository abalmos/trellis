import { assert, assertArrayIncludes, assertEquals } from "@std/assert";
import { defineAppContract, TrellisClient } from "@qlever-llc/trellis";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.local-login-binds-approved-client" as const;
const fixture = createAuthLocalLoginFixture(CASE_ID);
const resolveOnlyContract = defineAppContract(() => ({
  id: caseScopedContractId("trellis.integration.auth-user-resolve", CASE_ID),
  displayName: "Trellis Integration Auth User Resolve",
  description: "Resolves explicit auth user ids without directory access.",
  uses: [trellisAuth.AuthUsersResolve],
}));

liveTrellisTest({
  name:
    "auth.local-login-binds-approved-client binds local admin session and calls service",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.setupService(runtime);
    const { clientAuth } = await fixture.setupClientRegistration(runtime);
    let authRequired = false;

    try {
      const client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: clientAuth.auth,
        onAuthRequired: async (ctx) => {
          authRequired = true;
          return await clientAuth.onAuthRequired(ctx);
        },
      }).orThrow();

      try {
        assert(authRequired, "expected local-login flow to require auth");

        const me = await client.authSessionsMe({}).orThrow();
        assertEquals(me.participantKind, "app");
        assert(me.user !== null, "expected Auth.Sessions.Me to return a user");
        assertEquals(me.user.active, true);
        assertArrayIncludes(me.user.capabilities, ["admin"]);

        const ping = await client.authLoginPing({
          message: fixture.pingMessage,
        }).orThrow();
        assertEquals(ping, { message: fixture.pingMessage, accepted: true });

        const resolveClient = await runtime.connectClient({
          name: caseScopedName("auth-users-resolve-client", CASE_ID),
          contract: resolveOnlyContract,
        });
        try {
          assert(
            !("authUsersList" in resolveClient),
            "resolve-only contract must not receive directory access",
          );
          const resolved = await resolveClient.authUsersResolve({
            userIds: [me.user.userId, "usr_missing", me.user.userId],
          }).orThrow();
          assertEquals(resolved, {
            users: [{
              userId: me.user.userId,
              displayName: me.user.name,
              email: me.user.email,
            }],
            missing: ["usr_missing"],
          });
        } finally {
          await resolveClient.connection.close();
        }
      } finally {
        await client.connection.close();
      }
    } finally {
      await service.stop();
    }
  },
});
