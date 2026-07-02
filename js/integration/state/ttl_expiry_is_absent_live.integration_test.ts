import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createStateFixture } from "./_fixture.ts";

const CASE_ID = "state.ttl-expiry-is-absent-live" as const;
const fixture = createStateFixture(CASE_ID);

liveTrellisTest({
  name: "state.ttl-expiry-is-absent-live treats expired TTL entries as absent",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const client = await runtime.connectClient({
      name: fixture.clientName,
      contract: fixture.clientContract,
    });

    const createdPreferences = await client.state.preferences.put(
      { theme: "dark", density: "comfortable" },
      { expectedRevision: null, ttlMs: 100 },
    ).orThrow();
    assertEquals(createdPreferences.applied, true);
    if (
      !createdPreferences.applied || createdPreferences.entry === undefined
    ) {
      throw new Error("expected preferences create to return an entry");
    }
    if ("migrationRequired" in createdPreferences.entry) {
      throw new Error("expected current preferences entry");
    }
    assertEquals(typeof createdPreferences.entry.revision, "string");

    const drafts = client.state.drafts.prefix(fixture.draftPrefix);
    const createdDraft = await drafts.put(
      fixture.draftKey,
      { title: "TTL Draft", body: "from integration test" },
      { expectedRevision: null, ttlMs: 100 },
    ).orThrow();
    assertEquals(createdDraft.applied, true);
    if (!createdDraft.applied || createdDraft.entry === undefined) {
      throw new Error("expected draft create to return an entry");
    }
    if ("migrationRequired" in createdDraft.entry) {
      throw new Error("expected current draft entry");
    }

    await new Promise((resolve) => setTimeout(resolve, 250));

    assertEquals(await client.state.preferences.get().orThrow(), {
      found: false,
    });

    const listed = await drafts.list({ limit: 10 }).orThrow();
    assertEquals(
      listed.entries.some((entry) =>
        !("migrationRequired" in entry) &&
        entry.key === `${fixture.draftPrefix}/${fixture.draftKey}`
      ),
      false,
    );

    const createdOverExpired = await client.state.preferences.put(
      { theme: "light", density: "compact" },
      { expectedRevision: null },
    ).orThrow();
    assertEquals(createdOverExpired.applied, true);

    const deletedExpired = await drafts.delete(fixture.draftKey, {
      expectedRevision: createdDraft.entry.revision,
    }).orThrow();
    assertEquals(deletedExpired.deleted, false);
  },
});
