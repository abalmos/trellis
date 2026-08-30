// Generated from ./ts/packages/trellis/.trellis/generated/protocol/apis/trellis.core@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import * as Types from "./types.ts";
import {
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "./schemas.ts";
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.core@v1" as const;

export const TrellisSurfaceStatus = rpcAction(
  API_ID,
  "Trellis.Surface.Status",
  {
    subject: "rpc.v1.Trellis.Surface.Status",
    permission: Object.freeze({
      apiId: "trellis.core@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Trellis.Surface.Status",
      action: "call",
    }),
    input: schema<Types.TrellisSurfaceStatusInput>(
      TrellisSurfaceStatusRequestSchema,
    ),
    output: schema<Types.TrellisSurfaceStatusOutput>(
      TrellisSurfaceStatusResponseSchema,
    ),
    callerCapabilities: ["trellis.core::authority.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
    runtimeErrors: [
      {
        type: "UnexpectedError",
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "TrellisSurfaceStatus",
  ACTION_SOURCE,
);
