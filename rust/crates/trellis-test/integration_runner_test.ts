import { assertEquals, assertThrows } from "@std/assert";
import {
  parseIntegrationRunnerArgs,
  testNamesFromList,
} from "./integration_runner.ts";

Deno.test("Rust integration runner parses worker and libtest arguments", () => {
  assertEquals(
    parseIntegrationRunnerArgs(["--jobs", "3", "--", "--nocapture"]),
    { jobs: 3, testArgs: ["--nocapture"] },
  );
  assertThrows(
    () => parseIntegrationRunnerArgs(["--jobs=0"]),
    Error,
    "positive integer",
  );
});

Deno.test("Rust integration runner extracts test tenant names", () => {
  assertEquals(
    testNamesFromList(
      "rpc::success: test\nrust_integration_manifest_conforms_to_shared_matrix: test\n\n2 tests, 0 benchmarks\n",
    ),
    ["rpc::success", "rust_integration_manifest_conforms_to_shared_matrix"],
  );
});
