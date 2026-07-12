import { isErr, ok } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type contract from "../../../contract.ts";

type Handler = RpcHandler<typeof contract, "Sites.Get">;

export const getSite: Handler = async ({ input, client }) => {
  const entry = await client.kv.siteSummaries.get(input.siteId).take();

  return ok({ site: isErr(entry) ? undefined : entry.value });
};
