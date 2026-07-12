import { isErr, ok } from "@qlever-llc/trellis";
import type { FieldOpsService } from "../../deps.ts";

type Handler = Parameters<FieldOpsService["handleSitesGet"]>[0];

export const getSite: Handler = async ({ input, client }) => {
  const entry = await client.kv.siteSummaries.get(input.siteId).take();

  return ok({ site: isErr(entry) ? undefined : entry.value });
};
