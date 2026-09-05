import { Result } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import { participant } from "./.trellis/ts/participants/acme-orders-service/mod.ts";

/** Returns an example order receipt; this walkthrough does not persist orders. */
export const createOrder: RpcHandler<typeof participant, "Orders.Create"> = (
  { input },
) => Result.ok({ orderId: crypto.randomUUID(), customerId: input.customerId });
