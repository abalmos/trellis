// Generated from ./rust/crates/runtime/.trellis/artifacts/apis/trellis.health@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "@qlever-llc/trellis";
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
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.health@v1" as const;

export const HealthInspect = rpcAction(
  API_ID,
  "Health.Inspect",
  {
    subject: "rpc.v1.Health.Inspect",
    permission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Health.Inspect",
      action: "call",
    }),
    input: schema<Types.HealthInspectInput>(HealthInspectRequestSchema),
    output: schema<Types.HealthInspectOutput>(HealthInspectResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
    errors: ["NotFoundError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "NotFoundError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
    runtimeErrors: [
      {
        type: "NotFoundError",
        schema: schema<Types.NotFoundErrorData>(NotFoundErrorDataSchema),
        fromSerializable: Types.NotFoundError.fromSerializable,
      },
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
  "HealthInspect",
  ACTION_SOURCE,
);

export const HealthMetrics = rpcAction(
  API_ID,
  "Health.Metrics",
  {
    subject: "rpc.v1.Health.Metrics",
    permission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Health.Metrics",
      action: "call",
    }),
    input: schema<Types.HealthMetricsInput>(HealthMetricsRequestSchema),
    output: schema<Types.HealthMetricsOutput>(HealthMetricsResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
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
  "HealthMetrics",
  ACTION_SOURCE,
);

export const HealthQuery = rpcAction(
  API_ID,
  "Health.Query",
  {
    subject: "rpc.v1.Health.Query",
    permission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Health.Query",
      action: "call",
    }),
    input: schema<Types.HealthQueryInput>(HealthQueryRequestSchema),
    output: schema<Types.HealthQueryOutput>(HealthQueryResponseSchema),
    callerCapabilities: ["trellis.health::read"] as const,
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
  "HealthQuery",
  ACTION_SOURCE,
);

export const HealthStatusChanged = eventActions(
  API_ID,
  "Health.StatusChanged",
  {
    subject: "events.v1.Health.StatusChanged",
    publishPermission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "event",
      surfaceName: "Health.StatusChanged",
      action: "publish",
    }),
    subscribePermission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "event",
      surfaceName: "Health.StatusChanged",
      action: "subscribe",
    }),
    event: schema<Types.HealthStatusChangedEvent>(
      HealthStatusChangedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: ["trellis.health::read"] as const,
  },
  "HealthStatusChanged",
  true,
  ACTION_SOURCE,
);

export const HealthWatch = feedAction(
  API_ID,
  "Health.Watch",
  {
    subject: "feed.v1.Health.Watch",
    permission: Object.freeze({
      apiId: "trellis.health@v1",
      apiVersion: "v1",
      surfaceKind: "feed",
      surfaceName: "Health.Watch",
      action: "subscribe",
    }),
    input: schema<Types.HealthWatchInput>(HealthWatchRequestSchema),
    event: schema<Types.HealthWatchEvent>(HealthWatchFrameSchema),
    subscribeCapabilities: ["trellis.health::read"] as const,
  },
  "HealthWatch",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Health.Inspect": HealthInspect,
  "Health.Metrics": HealthMetrics,
  "Health.Query": HealthQuery,
  "Health.StatusChanged": HealthStatusChanged,
  "Health.Watch": HealthWatch,
} as const;
