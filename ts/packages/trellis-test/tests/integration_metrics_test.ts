import { assertEquals } from "@std/assert";
import { summarizeTrellisTestDurations } from "../src/integration/metrics.ts";

Deno.test("integration metrics report slowest durations first", () => {
  assertEquals(
    summarizeTrellisTestDurations([
      { event: "duration", metric: "fast", durationMs: 2 },
      { event: "process-start", process: "trellis" },
      { event: "duration", metric: "slow", durationMs: 7 },
    ], 1),
    [{
      metric: "slow",
      attributes: undefined,
      count: 1,
      averageMs: 7,
      maxMs: 7,
    }],
  );
});
