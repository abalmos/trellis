import { isErr, ok } from "@qlever-llc/trellis";
import type { SiteSummary } from "../../../../shared/field_data.ts";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type contract from "../../../contract.ts";

type Handler = RpcHandler<typeof contract, "Sites.List">;

export const listSites: Handler = async ({ input, client }) => {
  const sites: SiteSummary[] = [];
  const keys = await client.kv.siteSummaries.keys(">").orThrow();

  for await (const key of keys) {
    const entry = await client.kv.siteSummaries.get(key).take();
    if (!isErr(entry)) {
      sites.push(entry.value);
    }
  }

  sites.sort((left, right) => left.siteName.localeCompare(right.siteName));
  const offset = input.offset ?? 0;
  const count = sites.length;

  return ok({
    entries: sites.slice(offset, offset + input.limit),
    count,
    offset,
    limit: input.limit,
    ...(input.limit > 0 && offset + input.limit < count
      ? { nextOffset: offset + input.limit }
      : {}),
  });
};
