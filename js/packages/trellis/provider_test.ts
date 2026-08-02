import { assertEquals } from "jsr:@std/assert";
import { AsyncResult, Result } from "@qlever-llc/result";
import { Type } from "typebox";
import { defineServiceContract } from "./contract.ts";
import { rpcAction } from "./contracts.ts";
import { createProviderRuntime, PROVIDER_CALLER } from "./provider.ts";

const Empty = Type.Object({});
const ThreadsCreate = rpcAction("ai@v1", "AI.ThreadsCreate", {
  subject: "rpc.v1.AI.ThreadsCreate",
  permission: Object.freeze({
    apiId: "ai@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "AI.ThreadsCreate",
    action: "call",
  }),
  input: Empty,
  output: Empty,
  callerCapabilities: [],
}, "AIThreadsCreate");

const contract = defineServiceContract(
  { schemas: { Empty } },
  (ref) => ({
    id: "provider-test@v1",
    displayName: "Provider Test",
    description: "Verifies flat provider action binding.",
    events: {
      "AI.RunActivity": {
        version: "v1",
        event: ref.schema("Empty"),
      },
    },
    uses: [ThreadsCreate],
  }),
);

Deno.test("provider binds selected and owned actions without legacy nested lookup", async () => {
  const calls: string[] = [];
  let listener: ((event: unknown, context: unknown) => unknown) | undefined;
  const session = {
    connection: {},
    state: {},
    publishPrepared: () => AsyncResult.ok(undefined),
    transfer: () => AsyncResult.ok(undefined),
    wait: () => AsyncResult.ok(undefined),
    request: (name: string) => {
      calls.push(name);
      return AsyncResult.ok({});
    },
    listenEvent: (
      name: string,
      _subjectData: Record<string, unknown>,
      handler: (event: unknown, context: unknown) => unknown,
    ) => {
      calls.push(name);
      listener = handler;
      return AsyncResult.ok(undefined);
    },
    publish: (name: string) => {
      calls.push(name);
      return AsyncResult.ok(undefined);
    },
    prepare: (name: string) => {
      calls.push(name);
      return Result.ok({});
    },
  };
  const legacy = {
    get ai(): {
      runActivity: {
        listen(
          handler: (event: unknown, context: unknown) => unknown,
          subjectData?: Record<string, unknown>,
          options?: unknown,
        ): unknown;
        publish(event: Record<string, unknown>): unknown;
        prepare(event: Record<string, unknown>): Result<unknown, never>;
      };
    } {
      throw new Error("legacy nested surface accessed");
    },
  };
  const service = {
    kv: {},
    store: {},
    jobs: {},
    health: {},
    connection: {},
    name: "provider-test",
    createSqlOutbox() {},
    createTransfer() {},
    publishPrepared() {},
    wait: async () => {},
    stop: async () => {},
    handle: { rpc: {}, operation: {}, feed: {} },
    event: legacy,
    [PROVIDER_CALLER]: session,
  };

  const provider = createProviderRuntime(service, contract);
  await provider.aiThreadsCreate({}).orThrow();
  const onRunActivity = Reflect.get(provider, "onAiRunActivity") as (
    handler: (args: { event: unknown }) => void,
  ) => AsyncResult<void, never>;
  const publishRunActivity = Reflect.get(provider, "publishAiRunActivity") as
    & ((event: Record<string, unknown>) => AsyncResult<void, never>)
    & {
      prepare(event: Record<string, unknown>): Result<unknown, never>;
    };
  await onRunActivity(({ event }) => {
    assertEquals(event, {});
  }).orThrow();
  await listener?.({}, {});
  await publishRunActivity({}).orThrow();
  publishRunActivity.prepare({}).orThrow();

  assertEquals(calls, [
    "AI.ThreadsCreate",
    "AI.RunActivity",
    "AI.RunActivity",
    "AI.RunActivity",
  ]);
});
