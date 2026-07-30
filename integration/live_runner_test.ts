import { assertEquals, assertThrows } from "@std/assert";
import {
  allocateWorkers,
  orchestrateWorkerLanes,
  selectTypeScriptCases,
  type WorkerLane,
} from "./live_runner.ts";

Deno.test("live runner rejects empty TypeScript parent filters", () => {
  const cases = [
    { id: "rpc.one", completion: { typescript: "implemented" } },
    { id: "rpc.pending", completion: { typescript: "planned" } },
  ];
  assertEquals(selectTypeScriptCases(cases, "rpc.one"), [cases[0]]);
  assertThrows(
    () => selectTypeScriptCases(cases, "rpc.unknown"),
    Error,
    "does not name an implemented TypeScript case",
  );
  assertThrows(
    () => selectTypeScriptCases(cases, undefined, "feeds."),
    Error,
    "selects no implemented TypeScript cases",
  );
});

Deno.test("live worker allocation shares one global budget", () => {
  assertEquals(allocateWorkers(8, 2), [4, 4]);
  assertEquals(allocateWorkers(9, 2), [4, 5]);
  assertEquals(allocateWorkers(8, 1), [8]);
  assertEquals(allocateWorkers(9, 3).reduce((sum, jobs) => sum + jobs), 9);
});

Deno.test("live lanes overlap and teardown follows both", async () => {
  const events: string[] = [];
  const first = Promise.withResolvers<void>();
  const second = Promise.withResolvers<void>();
  const lanes: WorkerLane[] = [
    {
      name: "typescript",
      run: async (jobs) => {
        events.push(`typescript:${jobs}`);
        await first.promise;
        events.push("typescript:done");
        return 0;
      },
    },
    {
      name: "rust",
      run: async (jobs) => {
        events.push(`rust:${jobs}`);
        await second.promise;
        events.push("rust:done");
        return 0;
      },
    },
  ];

  const running = orchestrateWorkerLanes(lanes, 8, async () => {
    events.push("host:stop");
  });
  assertEquals(events, ["typescript:4", "rust:4"]);
  second.resolve();
  await Promise.resolve();
  assertEquals(events.includes("host:stop"), false);
  first.resolve();
  assertEquals(await running, 0);
  assertEquals(events.at(-1), "host:stop");
});

Deno.test("live lane failure drains its sibling deterministically", async () => {
  const events: string[] = [];
  const sibling = Promise.withResolvers<void>();
  const running = orchestrateWorkerLanes(
    [
      {
        name: "typescript",
        run: async () => {
          events.push("typescript:failed");
          return 7;
        },
      },
      {
        name: "rust",
        run: async () => {
          await sibling.promise;
          events.push("rust:drained");
          return 9;
        },
      },
    ],
    8,
    async () => {
      events.push("host:stop");
    },
  );

  await Promise.resolve();
  assertEquals(events, ["typescript:failed"]);
  sibling.resolve();
  assertEquals(await running, 7);
  assertEquals(events, ["typescript:failed", "rust:drained", "host:stop"]);
});

Deno.test("one live worker serializes lanes", async () => {
  const events: string[] = [];
  const lanes = ["typescript", "rust"].map((name): WorkerLane => ({
    name,
    run: async (jobs) => {
      events.push(`${name}:${jobs}`);
      return 0;
    },
  }));
  assertEquals(await orchestrateWorkerLanes(lanes, 1), 0);
  assertEquals(events, ["typescript:1", "rust:1"]);
});
