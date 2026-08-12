import { assertEquals } from "@std/assert";

import { apiDigest } from "../contract_support/protocol_artifacts.ts";
import { API, API_DIGEST, API_ID } from "./_generated/core/api.ts";

Deno.test("generated TypeScript core API matches canonical JSON and digest", async () => {
  const canonical = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../generated/protocol/apis/trellis.core@v1.json",
        import.meta.url,
      ),
    ),
  );
  assertEquals(API_ID, "trellis.core@v1");
  assertEquals(API, canonical);
  assertEquals(API_DIGEST, apiDigest(canonical));
});
