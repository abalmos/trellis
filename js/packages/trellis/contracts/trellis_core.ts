import {
  defineServiceContract,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "@qlever-llc/trellis";

const schemas = {
  TrellisSurfaceStatusRequest: TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponse: TrellisSurfaceStatusResponseSchema,
} as const;

export const trellisCore = defineServiceContract(
  { schemas },
  (ref) => ({
    id: "trellis.core@v1",
    displayName: "Trellis Core",
    description:
      "Trellis runtime RPCs available to all connected participants.",
    docs: {
      summary: "Runtime authority and binding APIs.",
      markdown:
        "Exposes runtime bindings and surface availability checks used by platform participants.",
    },
    capabilities: {
      "authority.read": {
        displayName: "Read participant authority",
        description: "Inspect native participant surface authority.",
      },
    },
    rpc: {
      "Trellis.Surface.Status": {
        version: "v1",
        input: ref.schema("TrellisSurfaceStatusRequest"),
        output: ref.schema("TrellisSurfaceStatusResponse"),
        capabilities: { call: ["authority.read"] },
        errors: [ref.error("ValidationError"), ref.error("UnexpectedError")],
        docs: {
          summary: "Inspect surface availability.",
          markdown:
            "Reports capability and deployment authority status for a contract-owned surface.",
        },
      },
    },
  }),
);

export const CONTRACT_ID = trellisCore.CONTRACT_ID;
export const API = trellisCore.API;
export const API_DIGEST = trellisCore.API_DIGEST;
export default trellisCore;
