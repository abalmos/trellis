import { assertEquals, assertExists } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createStateFixture } from "./_fixture.ts";

const CASE_ID = "state.migration-required-is-returned-live" as const;
const fixture = createStateFixture(CASE_ID);

liveTrellisTest({
  name:
    "state.migration-required-is-returned-live returns migration-required for old state versions",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const clientV1 = await runtime.connectClient({
      name: fixture.clientName,
      contract: fixture.clientContract,
    });
    const clientV2 = await runtime.connectClient({
      name: fixture.clientV2Name,
      contract: fixture.clientContractV2,
    });
    const admin = await runtime.connectClient({
      name: fixture.adminName,
      contract: fixture.adminContract,
    });

    const preferences = await clientV1.state.preferences.put(
      { theme: "dark", density: "comfortable" },
      { expectedRevision: null },
    ).orThrow();
    if (!preferences.applied || preferences.entry === undefined) {
      throw new Error("expected preferences write to apply");
    }

    const draftsV1 = clientV1.state.drafts.prefix(fixture.draftPrefix);
    const draft = await draftsV1.put(
      fixture.draftKey,
      { title: "Migration Draft", body: "from v1" },
      { expectedRevision: null },
    ).orThrow();
    if (!draft.applied || draft.entry === undefined) {
      throw new Error("expected draft write to apply");
    }

    const preferenceMigration = await clientV2.state.preferences.get()
      .orThrow();
    assertMigration(preferenceMigration, {
      value: { theme: "dark", density: "comfortable" },
      revision: preferences.entry.revision,
      stateVersion: "preferences.v1",
      currentStateVersion: "preferences.v2",
    });

    const draftMigration = await clientV2.state.drafts.prefix(
      fixture.draftPrefix,
    )
      .get(fixture.draftKey).orThrow();
    assertMigration(draftMigration, {
      value: { title: "Migration Draft", body: "from v1" },
      revision: draft.entry.revision,
      stateVersion: "drafts.v1",
      currentStateVersion: "drafts.v2",
    });

    const sessions = await admin.authSessionsList({ limit: 500 })
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

    const stateTarget = {
      scope: "userApp" as const,
      contractId: fixture.clientContract.CONTRACT_ID,
      contractDigest: fixture.clientContractV2.CONTRACT_DIGEST,
      user: {
        origin: session.principal.identity.provider,
        id: session.principal.identity.subject,
        userId: session.principal.userId,
      },
    };

    const adminPreferences = await admin.stateAdminGet({
      ...stateTarget,
      store: "preferences",
    }).orThrow();
    assertMigration(adminPreferences, {
      value: { theme: "dark", density: "comfortable" },
      revision: preferences.entry.revision,
      stateVersion: "preferences.v1",
      currentStateVersion: "preferences.v2",
    });

    const adminDrafts = await admin.stateAdminList({
      ...stateTarget,
      store: "drafts",
      prefix: fixture.draftPrefix,
      offset: 0,
      limit: 10,
    }).orThrow();
    const adminDraft = adminDrafts.entries.find((entry) =>
      "migrationRequired" in entry &&
      entry.entry.key === `${fixture.draftPrefix}/${fixture.draftKey}`
    );
    assertExists(adminDraft, "expected admin list to include migrated draft");
    assertMigration(adminDraft, {
      value: { title: "Migration Draft", body: "from v1" },
      revision: draft.entry.revision,
      stateVersion: "drafts.v1",
      currentStateVersion: "drafts.v2",
    });
  },
});

function assertMigration(
  actual: unknown,
  expected: {
    value: unknown;
    revision: string;
    stateVersion: string;
    currentStateVersion: string;
  },
) {
  if (
    actual === null || typeof actual !== "object" ||
    !("migrationRequired" in actual)
  ) {
    throw new Error("expected migration-required state result");
  }
  const migration = actual as {
    migrationRequired: true;
    entry: { value: unknown; revision: string };
    stateVersion: string;
    currentStateVersion: string;
    writerContractDigest: string;
  };
  assertEquals(migration.migrationRequired, true);
  assertEquals(migration.entry.value, expected.value);
  assertEquals(migration.entry.revision, expected.revision);
  assertEquals(migration.stateVersion, expected.stateVersion);
  assertEquals(migration.currentStateVersion, expected.currentStateVersion);
  assertEquals(
    migration.writerContractDigest,
    fixture.clientContract.CONTRACT_DIGEST,
  );
}
