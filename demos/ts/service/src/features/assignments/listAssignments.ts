import { ASSIGNED_INSPECTIONS } from "../../../../shared/field_data.ts";
import { ok } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type { participant } from "../../../.trellis/ts/participants/demo-service/mod.ts";

type Handler = RpcHandler<typeof participant, "Assignments.List">;

export const listAssignments: Handler = ({ input }) => {
  const offset = input.offset ?? 0;
  const count = ASSIGNED_INSPECTIONS.length;
  return ok({
    entries: ASSIGNED_INSPECTIONS.slice(offset, offset + input.limit),
    count,
    offset,
    limit: input.limit,
    ...(input.limit > 0 && offset + input.limit < count
      ? { nextOffset: offset + input.limit }
      : {}),
  });
};
