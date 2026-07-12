import { assert, assertEquals } from "@std/assert";
import {
  type CallerRuntime,
  isErr,
  TrellisClient,
} from "@qlever-llc/trellis";
import { waitFor } from "@qlever-llc/trellis-test";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthLocalLoginFixture } from "./_fixture.ts";

const CASE_ID = "auth.session-revoke-denies-reconnect" as const;
const fixture = createAuthLocalLoginFixture(CASE_ID);

liveTrellisTest({
  name:
    "auth.session-revoke-denies-reconnect revokes an app session and denies reuse",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.setupService(runtime);
    const admin = await fixture.setupSessionAdmin(runtime);
    const { clientKey, clientAuth } = await fixture.setupClientRegistration(
      runtime,
    );

    let client:
      | CallerRuntime<typeof fixture.clientContract>
      | undefined;

    try {
      client = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: clientAuth.auth,
        onAuthRequired: async (ctx) => await clientAuth.onAuthRequired(ctx),
      }).orThrow();

      await client.authLoginPing({
        message: fixture.pingMessage,
      }).orThrow();

      const before = await admin.authSessionsList({ limit: 500 })
        .orThrow();
      const targetSession = before.entries.find((entry) =>
        entry.participantKind === "app" &&
        entry.sessionKey === clientKey.sessionKey
      );
      assert(
        targetSession,
        "expected Auth.Sessions.List to include app session",
      );

      const revoked = await admin.authSessionsRevoke({
        sessionKey: targetSession.sessionKey,
      }).orThrow();
      assertEquals(revoked.success, true);

      await waitFor(async () => {
        const after = await admin.authSessionsList({ limit: 500 })
          .orThrow();
        return after.entries.every((entry) =>
          entry.sessionKey !== targetSession.sessionKey
        );
      });

      await waitFor(async () => {
        const result = await client!.authSessionsMe({});
        return result.isErr();
      });

      const reconnect = await TrellisClient.connect({
        trellisUrl: runtime.trellisUrl,
        name: fixture.clientName,
        contract: fixture.clientContract,
        auth: clientAuth.auth,
      });
      const reconnectValue = reconnect.take();
      if (!isErr(reconnectValue)) {
        await reconnectValue.connection.close();
      }
      assert(
        isErr(reconnectValue),
        "expected revoked session to fail reconnect without a new auth flow",
      );
    } finally {
      await client?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop();
    }
  },
});
