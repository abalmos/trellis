// Generated from trellis.core@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../generated.ts";
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

const __TrellisSurfaceStatusDescriptor = {
  subject: "rpc.v1.Trellis.Surface.Status",
  permission: Object.freeze({
    apiId: "trellis.core@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Trellis.Surface.Status",
    action: "call",
  }) as {
    readonly apiId: "trellis.core@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Trellis.Surface.Status";
    readonly action: "call";
  },
  input: schema<Types.TrellisSurfaceStatusInput>(
    TrellisSurfaceStatusRequestSchema,
  ) as ReturnType<typeof schema<Types.TrellisSurfaceStatusInput>>,
  output: schema<Types.TrellisSurfaceStatusOutput>(
    TrellisSurfaceStatusResponseSchema,
  ) as ReturnType<typeof schema<Types.TrellisSurfaceStatusOutput>>,
  callerCapabilities: ["trellis.core::authority.read"] as const,
  errors: ["UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  runtimeErrors: [
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const TrellisSurfaceStatus: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Trellis.Surface.Status",
    typeof __TrellisSurfaceStatusDescriptor
  >
> = rpcAction(
  API_ID,
  "Trellis.Surface.Status",
  __TrellisSurfaceStatusDescriptor,
  "TrellisSurfaceStatus",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Trellis.Surface.Status": TrellisSurfaceStatus as typeof TrellisSurfaceStatus,
} as const;
