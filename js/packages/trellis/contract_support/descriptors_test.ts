import { assertEquals, assertThrows } from "@std/assert";
import { Type } from "typebox";

import {
  as,
  defineAppContract,
  eventActions,
  optional,
  rpcAction,
} from "./mod.ts";
import { schema } from "./runtime.ts";
import type { CallerRuntime } from "../caller.ts";

const Empty = schema(Type.Object({}));
const OrdersGet = rpcAction(
  "orders@v1",
  "Orders.Get",
  {
    subject: "rpc.v1.Orders.Get",
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  },
  "OrdersGet",
);
const OrdersChanged = eventActions(
  "orders@v1",
  "Orders.Changed",
  {
    subject: "events.v1.Orders.Changed",
    event: Empty,
    publishCapabilities: [],
    subscribeCapabilities: [],
  },
  "OrdersChanged",
  false,
);
const HealthQuery = rpcAction(
  "trellis.health@v1",
  "Health.Query",
  {
    subject: "rpc.v1.Health.Query",
    input: Empty,
    output: Empty,
    callerCapabilities: [],
  },
  "HealthQuery",
);

Deno.test("direct action descriptors emit deterministic canonical uses", () => {
  const contract = defineAppContract(() => ({
    id: "storefront@v1",
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

  assertEquals(contract.CONTRACT.uses, {
    required: {
      "orders@v1": {
        contract: "orders@v1",
        rpc: { call: ["Orders.Get"] },
        events: { subscribe: ["Orders.Changed"] },
      },
    },
    optional: {
      "trellis.health@v1": {
        contract: "trellis.health@v1",
        rpc: { call: ["Health.Query"] },
      },
    },
  });
});

Deno.test("local aliases do not change canonical contract identity", () => {
  const original = defineAppContract(() => ({
    id: "storefront@v1",
    displayName: "Storefront",
    description: "Storefront descriptor test.",
    uses: [OrdersGet],
  }));
  const aliased = defineAppContract(() => ({
    id: "storefront@v1",
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

  assertEquals(aliased.CONTRACT, original.CONTRACT);
  assertEquals(aliased.CONTRACT_DIGEST, original.CONTRACT_DIGEST);
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
