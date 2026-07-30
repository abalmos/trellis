import { assertEquals, assertThrows } from "@std/assert";
import { trellisRepoRuntimeOptions } from "./runtime.ts";

Deno.test("repo runtime uses supplied prebuilt server without Cargo", () => {
  const names = [
    "TRELLIS_TEST_PREBUILT_ONLY",
    "TRELLIS_TEST_SERVER_BIN",
  ] as const;
  const previous = new Map(names.map((name) => [name, Deno.env.get(name)]));
  try {
    Deno.env.set("TRELLIS_TEST_PREBUILT_ONLY", "1");
    Deno.env.set("TRELLIS_TEST_SERVER_BIN", "/tmp/trellis-server");
    assertEquals(trellisRepoRuntimeOptions().trellis.command, {
      cmd: "/tmp/trellis-server",
      args: ["--config", "{config}", "all"],
    });

    Deno.env.delete("TRELLIS_TEST_SERVER_BIN");
    assertThrows(
      trellisRepoRuntimeOptions,
      Error,
      "refusing Cargo fallback",
    );

    Deno.env.delete("TRELLIS_TEST_PREBUILT_ONLY");
    assertEquals(trellisRepoRuntimeOptions().trellis.command.cmd, "cargo");
  } finally {
    for (const [name, value] of previous) {
      if (value === undefined) Deno.env.delete(name);
      else Deno.env.set(name, value);
    }
  }
});
