import { assertEquals, assertThrows } from "@std/assert";
import { Type } from "typebox";

import {
  as,
  defineAppContract,
  defineServiceContract,
  eventActions,
  optional,
  rpcAction,
} from "./mod.ts";
import { apiDigest } from "./protocol_artifacts.ts";
import { schema } from "./runtime.ts";
import type { CallerRuntime } from "../caller.ts";

const Empty = schema(Type.Object({}));

function nativeApiSource(
  id: string,
  rpcName?: string,
  eventName?: string,
) {
  const artifact = {
    format: "trellis.api.v1",
    id,
    displayName: id,
    description: id,
    schemas: { Empty: { properties: {}, type: "object" } },
    ...(rpcName
      ? {
        rpc: {
          [rpcName]: {
            version: "v1",
            input: { schema: "Empty" },
            output: { schema: "Empty" },
          },
        },
      }
      : {}),
    ...(eventName
      ? {
        events: {
          [eventName]: {
            version: "v1",
            event: { schema: "Empty" },
          },
        },
      }
      : {}),
  };
  return { api: artifact, apiDigest: apiDigest(artifact) };
}

const OrdersApi = nativeApiSource(
  "orders@v1",
  "Orders.Get",
  "Orders.Changed",
);
const HealthApi = nativeApiSource("trellis.health@v1", "Health.Query");
const OrdersGet = rpcAction(
  "orders@v1",
  "Orders.Get",
  {
    subject: "rpc.v1.Orders.Get",
    permission: Object.freeze({
      apiId: "orders@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Orders.Get",
      action: "call",
    }),
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  },
  "OrdersGet",
  OrdersApi,
);
const OrdersChanged = eventActions(
  "orders@v1",
  "Orders.Changed",
  {
    subject: "events.v1.Orders.Changed",
    publishPermission: Object.freeze({
      apiId: "orders@v1",
      apiVersion: "v1",
      surfaceKind: "event",
      surfaceName: "Orders.Changed",
      action: "publish",
    }),
    subscribePermission: Object.freeze({
      apiId: "orders@v1",
      apiVersion: "v1",
      surfaceKind: "event",
      surfaceName: "Orders.Changed",
      action: "subscribe",
    }),
    event: Empty,
    publishCapabilities: [],
    subscribeCapabilities: [],
  },
  "OrdersChanged",
  false,
  OrdersApi,
);
const HealthQuery = rpcAction(
  "trellis.health@v1",
  "Health.Query",
  {
    subject: "rpc.v1.Health.Query",
    permission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Health.Query",
      action: "call",
    }),
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  },
  "HealthQuery",
  HealthApi,
);

const acronymActions = [
  rpcAction("ai@v1", "AI.GenerateJSON", {
    subject: "rpc.v1.AI.GenerateJSON",
    permission: Object.freeze({
      apiId: "ai@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "AI.GenerateJSON",
      action: "call",
    }),
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  }, "AIGenerateJSON"),
  rpcAction("ai@v1", "AI.OCR", {
    subject: "rpc.v1.AI.OCR",
    permission: Object.freeze({
      apiId: "ai@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "AI.OCR",
      action: "call",
    }),
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  }, "AIOCR"),
  rpcAction("jobs@v1", "Jobs.ListDLQ", {
    subject: "rpc.v1.Jobs.ListDLQ",
    permission: Object.freeze({
      apiId: "jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.ListDLQ",
      action: "call",
    }),
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  }, "JobsListDLQ"),
] as const;

Deno.test("connected names preserve Rust-compatible acronym word boundaries", () => {
  const [generateJson, ocr, listDlq] = acronymActions;
  const generateJsonName: "aiGenerateJson" = generateJson.connectedName;
  const ocrName: "aiOcr" = ocr.connectedName;
  const listDlqName: "jobsListDlq" = listDlq.connectedName;

  assertEquals(generateJsonName, "aiGenerateJson");
  assertEquals(ocrName, "aiOcr");
  assertEquals(listDlqName, "jobsListDlq");
});

Deno.test("direct action descriptors emit deterministic canonical uses", () => {
  const contract = defineAppContract(() => ({
    id: "storefront@v1",
    apiId: "storefront@v1",
    displayName: "Storefront",
    description: "Storefront descriptor test.",
    uses: [
      OrdersGet,
      OrdersChanged.subscribe,
      OrdersGet,
      optional(HealthQuery),
    ],
  }));
  const assertCallerTypes = (client: CallerRuntime<typeof contract>) => {
    client.ordersGet({});
    client.onOrdersChanged(() => {});
    client.healthQuery({});
    // @ts-expect-error nested generated facades are not part of the caller API
    client.rpc;
  };
  void assertCallerTypes;

  assertEquals(contract.PARTICIPANT.format, "trellis.participant.v1");
});

Deno.test("local aliases do not change canonical contract identity", () => {
  const original = defineAppContract(() => ({
    id: "storefront@v1",
    apiId: "storefront@v1",
    displayName: "Storefront",
    description: "Storefront descriptor test.",
    uses: [OrdersGet],
  }));
  const aliased = defineAppContract(() => ({
    id: "storefront@v1",
    apiId: "storefront@v1",
    displayName: "Storefront",
    description: "Storefront descriptor test.",
    uses: [as("getOrder", OrdersGet)],
  }));
  const assertAliasType = (client: CallerRuntime<typeof aliased>) => {
    client.getOrder({});
    // @ts-expect-error aliases replace the default connected name
    client.ordersGet;
  };
  void assertAliasType;

  assertEquals(aliased.API, original.API);
  assertEquals(aliased.API_DIGEST, original.API_DIGEST);
});

Deno.test("optional groups reject actions from different owners", () => {
  assertThrows(
    () => optional(OrdersGet, HealthQuery),
    Error,
    "same owner contract",
  );
});

Deno.test("owner-only events omit delegated publication", () => {
  assertEquals(OrdersChanged.publish, undefined);
  assertEquals(OrdersChanged.subscribe.kind, "event-subscribe");
});
