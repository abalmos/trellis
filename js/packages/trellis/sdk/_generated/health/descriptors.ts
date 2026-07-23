// Generated from ./generated/contracts/manifests/trellis.health@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import * as Types from "./types.ts";
import {
  HealthInspectRequestSchema,
  HealthInspectResponseSchema,
  HealthMetricsRequestSchema,
  HealthMetricsResponseSchema,
  HealthQueryRequestSchema,
  HealthQueryResponseSchema,
  HealthStatusChangedEventSchema,
  HealthWatchFrameSchema,
  HealthWatchRequestSchema,
  NotFoundErrorDataSchema,
} from "./schemas.ts";
import {
  CONTRACT as ACTION_ARTIFACT,
  CONTRACT_DIGEST as ACTION_DIGEST,
} from "./manifest.ts";

const ACTION_SOURCE = {
  artifact: ACTION_ARTIFACT,
  digest: ACTION_DIGEST,
} as const;

const CONTRACT_ID = "trellis.health@v1" as const;

export const HealthInspect = rpcAction(
  CONTRACT_ID,
  "Health.Inspect",
  {
    subject: "rpc.v1.Health.Inspect",
    input: schema<Types.HealthInspectInput>(HealthInspectRequestSchema),
    output: schema<Types.HealthInspectOutput>(HealthInspectResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
    errors: ["UnexpectedError", "ValidationError", "NotFoundError"] as const,
    declaredErrorTypes: [
      "UnexpectedError",
      "ValidationError",
      "NotFoundError",
    ] as const,
    runtimeErrors: [
      {
        type: "NotFoundError",
        schema: schema<Types.NotFoundErrorData>(NotFoundErrorDataSchema),
        fromSerializable: Types.NotFoundError.fromSerializable,
      },
    ] as const,
  },
  "HealthInspect",
  ACTION_SOURCE,
);

export const HealthMetrics = rpcAction(
  CONTRACT_ID,
  "Health.Metrics",
  {
    subject: "rpc.v1.Health.Metrics",
    input: schema<Types.HealthMetricsInput>(HealthMetricsRequestSchema),
    output: schema<Types.HealthMetricsOutput>(HealthMetricsResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "HealthMetrics",
  ACTION_SOURCE,
);

export const HealthQuery = rpcAction(
  CONTRACT_ID,
  "Health.Query",
  {
    subject: "rpc.v1.Health.Query",
    input: schema<Types.HealthQueryInput>(HealthQueryRequestSchema),
    output: schema<Types.HealthQueryOutput>(HealthQueryResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "HealthQuery",
  ACTION_SOURCE,
);

export const HealthStatusChanged = eventActions(
  CONTRACT_ID,
  "Health.StatusChanged",
  {
    subject: "events.v1.Health.StatusChanged",
    event: schema<Types.HealthStatusChangedEvent>(
      HealthStatusChangedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: ["trellis.health::read"] as const,
  },
  "HealthStatusChanged",
  false,
  ACTION_SOURCE,
);

export const HealthWatch = feedAction(
  CONTRACT_ID,
  "Health.Watch",
  {
    subject: "feed.v1.Health.Watch",
    input: schema<Types.HealthWatchInput>(HealthWatchRequestSchema),
    event: schema<Types.HealthWatchEvent>(HealthWatchFrameSchema),
    subscribeCapabilities: ["trellis.health::read"] as const,
  },
  "HealthWatch",
  ACTION_SOURCE,
);
