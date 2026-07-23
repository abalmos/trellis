import { assertObjectMatch } from "@std/assert";

import { HealthQuery } from "../sdk/_generated/health/descriptors.ts";
import { defineAppContract } from "./mod.ts";
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
