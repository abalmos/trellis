import { ok } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type { participant } from "../../../.trellis/ts/participants/demo-service/mod.ts";
import { listReports as listReportRecords } from "./reportStore.ts";

type Handler = RpcHandler<typeof participant, "Reports.List">;

/** Lists completed closeout reports generated during this demo service run. */
export const listReports: Handler = ({ input }) => {
  const reports = listReportRecords();
  const offset = input.offset ?? 0;
  const count = reports.length;
  return ok({
    entries: reports.slice(offset, offset + input.limit),
    count,
    offset,
    limit: input.limit,
    ...(input.limit > 0 && offset + input.limit < count
      ? { nextOffset: offset + input.limit }
      : {}),
  });
};
