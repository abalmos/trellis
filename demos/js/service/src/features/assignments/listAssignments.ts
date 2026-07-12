import { ASSIGNED_INSPECTIONS } from "../../../../shared/field_data.ts";
import { ok } from "@qlever-llc/trellis";
import type { FieldOpsService } from "../../deps.ts";

type Handler = Parameters<FieldOpsService["handleAssignmentsList"]>[0];

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
