import { assertEquals } from "@std/assert";

import { API, API_DIGEST } from "./_generated/core/api.ts";
import { apiDigest } from "../contract_support/protocol_artifacts.ts";

Deno.test("Rust and TypeScript core SDKs embed the canonical native API", async () => {
  const canonical = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../generated/protocol/apis/trellis.core@v1.json",
        import.meta.url,
      ),
    ),
  );
  const rust = await Deno.readTextFile(
    new URL(
      "../../../../rust/crates/trellis/src/sdk/core/api.rs",
      import.meta.url,
    ),
  );
  const rustJson = rust.match(/pub const API_JSON: &str = (".*");/)?.[1];
  const rustDigest = rust.match(/pub const API_DIGEST: &str = "([^"]+)";/)?.[1];
  if (!rustJson || !rustDigest) {
    throw new Error("Rust core API constants are missing");
  }

  assertEquals(API, canonical);
  assertEquals(JSON.parse(JSON.parse(rustJson)), canonical);
  assertEquals(API_DIGEST, apiDigest(canonical));
  assertEquals(rustDigest, API_DIGEST);
});
