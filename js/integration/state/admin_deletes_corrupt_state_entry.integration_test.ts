import { assertEquals, assertExists, assertRejects } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createStateFixture } from "./_fixture.ts";

const CASE_ID = "state.admin-deletes-corrupt-state-entry" as const;
const fixture = createStateFixture(CASE_ID);

liveTrellisTest({
  name:
    "state.admin-deletes-corrupt-state-entry removes a malformed backing KV entry",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const client = await runtime.connectClient({
      name: fixture.clientName,
      contract: fixture.clientContract,
    });
    const admin = await runtime.connectClient({
      name: fixture.adminName,
      contract: fixture.adminContract,
    });

    const sessions = await admin.rpc.auth.sessionsList({ limit: 500 })
      .orThrow();
    const session = sessions.entries.find((entry) =>
      entry.participantKind === "app" &&
      entry.contractId === fixture.clientContract.CONTRACT_ID
    );
    assertExists(
      session,
      "expected Auth.Sessions.List to include state client",
    );
    if (session.principal.type !== "user") {
      throw new Error("expected state client session to have a user principal");
    }
    if (!runtime.seedRawStateEntry) {
      throw new Error("runtime does not support raw state seeding");
    }

    const stateTarget = {
      scope: "userApp" as const,
      contractId: fixture.clientContract.CONTRACT_ID,
      contractDigest: fixture.clientContract.CONTRACT_DIGEST,
      user: {
        origin: session.principal.identity.provider,
        id: session.principal.identity.subject,
        userId: session.principal.userId,
      },
    };
    const storageKey = [
      encodeStateComponent("user"),
      encodeStateComponent(session.principal.userId),
      encodeStateComponent(fixture.clientContract.CONTRACT_ID),
      encodeStateComponent("preferences"),
      "=value",
    ].join(".");

    await runtime.seedRawStateEntry({
      key: storageKey,
      value: {
        value: { theme: "dark", density: "comfortable" },
        updatedAt: new Date().toISOString(),
        stateVersion: "preferences.v1",
      },
    });

    await assertRejects(
      () => client.state.preferences.get().orThrow(),
      Error,
    );

    const deleted = await admin.rpc.state.adminDelete({
      ...stateTarget,
      store: "preferences",
    }).orThrow();
    assertEquals(deleted.deleted, true);
    assertEquals(await client.state.preferences.get().orThrow(), {
      found: false,
    });
  },
});

function encodeStateComponent(value: string): string {
  return [...value].map((char) => {
    if (/^[A-Za-z0-9_/-]$/.test(char)) return char;
    return [...new TextEncoder().encode(char)]
      .map((byte) => `=${byte.toString(16).toUpperCase().padStart(2, "0")}`)
      .join("");
  }).join("");
}
