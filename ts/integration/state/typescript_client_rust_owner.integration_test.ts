import { defineAppContract, state } from "@qlever-llc/trellis";
import { ValidationError } from "@qlever-llc/trellis/errors";
import { assert, assertEquals, assertInstanceOf } from "@std/assert";
import { Type } from "typebox";

import { integrationSlug } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";

const CASE_ID = "state.client-rust-owner" as const;

const contract = defineAppContract(
  {
    schemas: {
      Preferences: Type.Object({ theme: Type.String() }),
      Draft: Type.Object({ title: Type.String() }),
    },
  },
  (ref) => ({
    id: "trellis.integration.state-typescript.state-client-rust-owner@v1",
    apiId: "trellis.integration.state-typescript.state-client-rust-owner@v1",
    displayName: "Trellis Integration TypeScript State Client",
    description: "Exercises the Rust-owned State runtime from TypeScript.",
    uses: [state({
      preferences: {
        kind: "value",
        schema: ref.schema("Preferences"),
      },
      drafts: {
        kind: "map",
        schema: ref.schema("Draft"),
      },
    })],
  }),
);

liveTrellisTest({
  name: "state.client-rust-owner uses generated app State facades",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const client = await runtime.connectClient({
      name: `state-typescript-client-${integrationSlug(CASE_ID)}`,
      contract,
    });
    const created = await client.state.preferences.put(
      { theme: "dark" },
      { expectedRevision: null },
    ).orThrow();
    assertEquals(created.applied, true);
    if (!created.applied || "migrationRequired" in created.entry) {
      throw new Error("new State write unexpectedly requires migration");
    }
    const read = await client.state.preferences.get().orThrow();
    if ("migrationRequired" in read) {
      throw new Error("new State value unexpectedly requires migration");
    }
    assertEquals(read.found, true);
    if (read.found) assertEquals(read.entry.value.theme, "dark");
    const deleted = await client.state.preferences.delete({
      expectedRevision: created.entry.revision,
    }).orThrow();
    assertEquals(deleted.deleted, true);

    for (const invalid of ["/a", "a/", "a//b"]) {
      const result = await client.state.drafts.get(invalid);
      assert(result.isErr());
      assertInstanceOf(result.error, ValidationError);
      assert(String(result.error).includes("/key"));
      const put = await client.state.drafts.put(invalid, { title: "invalid" });
      assert(put.isErr());
      assertInstanceOf(put.error, ValidationError);
      assert(String(put.error).includes("/key"));
      const deleted = await client.state.drafts.delete(invalid);
      assert(deleted.isErr());
      assertInstanceOf(deleted.error, ValidationError);
      assert(String(deleted.error).includes("/key"));
    }
    for (const invalid of ["/a", "a/"]) {
      const result = await client.state.drafts.prefix(invalid).get("open");
      assert(result.isErr());
      assertInstanceOf(result.error, ValidationError);
      assert(String(result.error).includes("/key"));
      const put = await client.state.drafts.prefix(invalid).put("open", {
        title: "invalid",
      });
      assert(put.isErr());
      assertInstanceOf(put.error, ValidationError);
      assert(String(put.error).includes("/key"));
      const deleted = await client.state.drafts.prefix(invalid).delete("open");
      assert(deleted.isErr());
      assertInstanceOf(deleted.error, ValidationError);
      assert(String(deleted.error).includes("/key"));
    }
    const invalidPrefixedPut = await client.state.drafts.prefix("a").put(
      "/b",
      { title: "invalid" },
    );
    assert(invalidPrefixedPut.isErr());
    assertInstanceOf(invalidPrefixedPut.error, ValidationError);
    assert(String(invalidPrefixedPut.error).includes("/key"));
    const invalidPrefixedDelete = await client.state.drafts.prefix("a").delete(
      "/b",
    );
    assert(invalidPrefixedDelete.isErr());
    assertInstanceOf(invalidPrefixedDelete.error, ValidationError);
    assert(String(invalidPrefixedDelete.error).includes("/key"));
    const invalidList = await client.state.drafts.prefix("/inspection").list();
    assert(invalidList.isErr());
    assert(String(invalidList.error).includes("/prefix"));

    const nested = client.state.drafts.prefix("inspection").prefix("active");
    await nested.put("open", { title: "Open" }).orThrow();
    const nestedRead = await nested.get("open").orThrow();
    assert(!("migrationRequired" in nestedRead));
    assertEquals(nestedRead.found, true);
  },
});
