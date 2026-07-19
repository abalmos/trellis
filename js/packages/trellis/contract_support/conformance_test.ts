import { assertEquals, assertRejects, assertThrows } from "@std/assert";

import { canonicalizeJson, digestJson, type JsonValue } from "./canonical.ts";
import { digestContractManifest, type TrellisContractV1 } from "./mod.ts";

Deno.test("canonical json matches shared vectors", async () => {
  const fixtures = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../conformance/canonical-json/vectors.json",
        import.meta.url,
      ),
    ),
  ) as Array<{
    name: string;
    input?: JsonValue;
    inputJson?: string;
    canonical?: string;
    digest?: string;
    error?: boolean;
  }>;

  for (const fixture of fixtures) {
    if (fixture.error) {
      const input = JSON.parse(fixture.inputJson ?? "") as JsonValue;
      assertThrows(() => canonicalizeJson(input), Error);
      await assertRejects(() => digestJson(input), Error);
      continue;
    }
    if (
      fixture.input === undefined || fixture.canonical === undefined ||
      fixture.digest === undefined
    ) {
      throw new Error(`Incomplete canonical JSON fixture: ${fixture.name}`);
    }
    assertEquals(
      canonicalizeJson(fixture.input),
      fixture.canonical,
      fixture.name,
    );
    assertEquals(
      (await digestJson(fixture.input)).digest,
      fixture.digest,
      fixture.name,
    );
  }
});

Deno.test("contract digest matches shared vectors", async () => {
  const fixtures = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../conformance/contract-digest/vectors.json",
        import.meta.url,
      ),
    ),
  ) as Array<{
    name: string;
    input: TrellisContractV1;
    digest: string;
  }>;

  for (const fixture of fixtures) {
    assertEquals(
      digestContractManifest(fixture.input),
      fixture.digest,
      fixture.name,
    );
  }
});
