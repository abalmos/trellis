import { assertEquals, assertObjectMatch } from "@std/assert";
import { Type } from "typebox";

import { HealthQuery } from "../sdk/_generated/health/descriptors.ts";
import { defineAppContract, defineServiceContract } from "./mod.ts";
import { compileProtocolArtifacts } from "./protocol_artifacts.ts";

Deno.test("generated actions preserve canonical source artifact identity", async () => {
  const compiled = await compileProtocolArtifacts(defineAppContract(() => ({
    id: "trellis.test.health-observer@v1",
    displayName: "Health observer",
    description: "Checks generated dependency identity.",
    uses: [HealthQuery],
  })));

  assertObjectMatch(compiled.participant, {
    uses: {
      required: {
        "trellis.health@v1": {
          apiDigest: "xJBLtH2AFDOBjUIo0ejwkX8yJ5ejcldJv0qAOQVLJmQ",
        },
      },
    },
  });
});

Deno.test("inline actions preserve their provider API artifact", async () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.inline-provider@v1",
      displayName: "Inline provider",
      description: "Checks inline action source identity.",
      capabilities: {
        call: { displayName: "Call", description: "Call the provider." },
      },
      rpc: {
        "Inline.Call": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          capabilities: { call: ["call"] },
          errors: [],
        },
      },
    }),
  );
  const consumer = defineAppContract(() => ({
    id: "trellis.test.inline-consumer@v1",
    displayName: "Inline consumer",
    description: "Uses the inline provider.",
    uses: [provider.InlineCall],
  }));

  const provided = await compileProtocolArtifacts(provider);
  const consumed = await compileProtocolArtifacts(consumer);
  assertEquals(consumed.referencedApis, [provided.api]);
});
