import { ok } from "@qlever-llc/trellis";
import type { RpcHandler } from "@qlever-llc/trellis/service";
import type contract from "../../../contract.ts";

type Handler = RpcHandler<typeof contract, "Evidence.List">;

function evidenceIdForKey(
  key: string,
  metadata: Record<string, string>,
): string {
  return metadata.evidenceId || key;
}

/** Lists image evidence staged in the demo object store. */
export const listEvidence: Handler = async ({ input, client }) => {
  const uploads = await client.store.uploads.open().orThrow();
  const page = await uploads.list({
    prefix: input.prefix ?? "evidence/",
    offset: input.offset,
    limit: input.limit,
  }).orThrow();
  const evidence = [];

  for (const info of page.entries) {
    evidence.push({
      evidenceId: evidenceIdForKey(info.key, info.metadata),
      key: info.key,
      size: info.size,
      ...(info.contentType ? { contentType: info.contentType } : {}),
      evidenceType: info.metadata.evidenceType || "image",
      ...(info.metadata.fileName ? { fileName: info.metadata.fileName } : {}),
      uploadedAt: info.updatedAt,
    });
  }

  evidence.sort((left, right) =>
    right.uploadedAt.localeCompare(left.uploadedAt)
  );

  return ok({ ...page, entries: evidence });
};
