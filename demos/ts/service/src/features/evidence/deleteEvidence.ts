import { ok } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type contract from "../../../contract.ts";
import { recordActivity } from "../activity/index.ts";

type Handler = RpcHandler<typeof contract, "Evidence.Delete">;

/** Deletes a stored evidence object from the demo evidence locker. */
export const deleteEvidence: Handler = async ({ input, client }) => {
  const uploads = await client.store.uploads.open().orThrow();
  await uploads.delete(input.key).orThrow();
  await recordActivity(client, {
    kind: "evidence-deleted",
    message: `Deleted evidence upload ${input.key}`,
  });

  return ok({ key: input.key, deleted: true });
};
