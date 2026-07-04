import { assert, assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createStateFixture } from "./_fixture.ts";

const CASE_ID = "state.value-and-map-conflict-shapes-live" as const;
const fixture = createStateFixture(CASE_ID);

liveTrellisTest({
  name:
    "state.value-and-map-conflict-shapes-live differentiates create and stale conflicts",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const client = await runtime.connectClient({
      name: fixture.clientName,
      contract: fixture.clientContract,
    });

    const preferences = await client.state.preferences.put(
      { theme: "dark", density: "comfortable" },
      { expectedRevision: null },
    ).orThrow();
    assert(preferences.applied);

    const valueCreateConflict = await client.state.preferences.put(
      { theme: "light", density: "compact" },
      { expectedRevision: null },
    ).orThrow();
    assertEquals(valueCreateConflict.applied, false);
    if (
      valueCreateConflict.entry === undefined ||
      "migrationRequired" in valueCreateConflict.entry
    ) {
      throw new Error("expected current value create conflict entry");
    }
    assertEquals(valueCreateConflict.entry.value, {
      theme: "dark",
      density: "comfortable",
    });

    const valueStaleConflict = await client.state.preferences.put(
      { theme: "light", density: "compact" },
      { expectedRevision: "stale-revision" },
    ).orThrow();
    assertEquals(valueStaleConflict.applied, false);
    if (
      valueStaleConflict.entry === undefined ||
      "migrationRequired" in valueStaleConflict.entry
    ) {
      throw new Error("expected current value stale conflict entry");
    }
    assertEquals(valueStaleConflict.entry.value, {
      theme: "dark",
      density: "comfortable",
    });

    const drafts = client.state.drafts.prefix(fixture.draftPrefix);
    const draft = await drafts.put(
      fixture.draftKey,
      { title: "Conflict Draft", body: "from integration test" },
      { expectedRevision: null },
    ).orThrow();
    assert(draft.applied);

    const mapCreateConflict = await drafts.put(
      fixture.draftKey,
      { title: "Replacement Draft", body: "should not apply" },
      { expectedRevision: null },
    ).orThrow();
    assertEquals(mapCreateConflict.applied, false);
    if (
      mapCreateConflict.entry === undefined ||
      "migrationRequired" in mapCreateConflict.entry
    ) {
      throw new Error("expected current map create conflict entry");
    }
    assertEquals(
      mapCreateConflict.entry.key,
      `${fixture.draftPrefix}/${fixture.draftKey}`,
    );
    assertEquals(mapCreateConflict.entry.value, {
      title: "Conflict Draft",
      body: "from integration test",
    });

    const mapStaleConflict = await drafts.put(
      fixture.draftKey,
      { title: "Replacement Draft", body: "should not apply" },
      { expectedRevision: "stale-revision" },
    ).orThrow();
    assertEquals(mapStaleConflict.applied, false);
    if (
      mapStaleConflict.entry === undefined ||
      "migrationRequired" in mapStaleConflict.entry
    ) {
      throw new Error("expected current map stale conflict entry");
    }
    assertEquals(
      mapStaleConflict.entry.key,
      `${fixture.draftPrefix}/${fixture.draftKey}`,
    );
    assertEquals(mapStaleConflict.entry.value, {
      title: "Conflict Draft",
      body: "from integration test",
    });
  },
});
