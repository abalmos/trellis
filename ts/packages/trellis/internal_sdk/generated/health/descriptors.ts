// Generated from trellis.health@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../generated.ts";
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

const __HealthInspectDescriptor = {
  subject: "rpc.v1.Health.Inspect",
  permission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Health.Inspect",
    action: "call",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Health.Inspect";
    readonly action: "call";
  },
  input: schema<Types.HealthInspectInput>(
    HealthInspectRequestSchema,
  ) as ReturnType<typeof schema<Types.HealthInspectInput>>,
  output: schema<Types.HealthInspectOutput>(
    HealthInspectResponseSchema,
  ) as ReturnType<typeof schema<Types.HealthInspectOutput>>,
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
      schema: schema<Types.NotFoundErrorData>(
        NotFoundErrorDataSchema,
      ) as ReturnType<typeof schema<Types.NotFoundErrorData>>,
      fromSerializable: Types.NotFoundError
        .fromSerializable as typeof Types.NotFoundError.fromSerializable,
    },
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
export const HealthInspect: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Health.Inspect",
    typeof __HealthInspectDescriptor
  >
> = rpcAction(
  API_ID,
  "Health.Inspect",
  __HealthInspectDescriptor,
  "HealthInspect",
  ACTION_SOURCE,
);

const __HealthMetricsDescriptor = {
  subject: "rpc.v1.Health.Metrics",
  permission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Health.Metrics",
    action: "call",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Health.Metrics";
    readonly action: "call";
  },
  input: schema<Types.HealthMetricsInput>(
    HealthMetricsRequestSchema,
  ) as ReturnType<typeof schema<Types.HealthMetricsInput>>,
  output: schema<Types.HealthMetricsOutput>(
    HealthMetricsResponseSchema,
  ) as ReturnType<typeof schema<Types.HealthMetricsOutput>>,
  callerCapabilities: ["trellis.health::read"] as const,
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
export const HealthMetrics: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Health.Metrics",
    typeof __HealthMetricsDescriptor
  >
> = rpcAction(
  API_ID,
  "Health.Metrics",
  __HealthMetricsDescriptor,
  "HealthMetrics",
  ACTION_SOURCE,
);

const __HealthQueryDescriptor = {
  subject: "rpc.v1.Health.Query",
  permission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Health.Query",
    action: "call",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Health.Query";
    readonly action: "call";
  },
  input: schema<Types.HealthQueryInput>(HealthQueryRequestSchema) as ReturnType<
    typeof schema<Types.HealthQueryInput>
  >,
  output: schema<Types.HealthQueryOutput>(
    HealthQueryResponseSchema,
  ) as ReturnType<typeof schema<Types.HealthQueryOutput>>,
  callerCapabilities: ["trellis.health::read"] as const,
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
export const HealthQuery: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Health.Query",
    typeof __HealthQueryDescriptor
  >
> = rpcAction(
  API_ID,
  "Health.Query",
  __HealthQueryDescriptor,
  "HealthQuery",
  ACTION_SOURCE,
);

const __HealthStatusChangedDescriptor = {
  subject: "events.v1.Health.StatusChanged",
  publishPermission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Health.StatusChanged",
    action: "publish",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Health.StatusChanged";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Health.StatusChanged",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Health.StatusChanged";
    readonly action: "subscribe";
  },
  event: schema<Types.HealthStatusChangedEvent>(
    HealthStatusChangedEventSchema,
  ) as ReturnType<typeof schema<Types.HealthStatusChangedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.health::read"] as const,
} as const;
export const HealthStatusChanged: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Health.StatusChanged",
    typeof __HealthStatusChangedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Health.StatusChanged",
  __HealthStatusChangedDescriptor,
  "HealthStatusChanged",
  true,
  ACTION_SOURCE,
);

const __HealthWatchDescriptor = {
  subject: "feed.v1.Health.Watch",
  permission: Object.freeze({
    apiId: "trellis.health@v1",
    apiVersion: "v1",
    surfaceKind: "feed",
    surfaceName: "Health.Watch",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.health@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "feed";
    readonly surfaceName: "Health.Watch";
    readonly action: "subscribe";
  },
  input: schema<Types.HealthWatchInput>(HealthWatchRequestSchema) as ReturnType<
    typeof schema<Types.HealthWatchInput>
  >,
  event: schema<Types.HealthWatchEvent>(HealthWatchFrameSchema) as ReturnType<
    typeof schema<Types.HealthWatchEvent>
  >,
  subscribeCapabilities: ["trellis.health::read"] as const,
} as const;
export const HealthWatch: ReturnType<
  typeof feedAction<
    typeof API_ID,
    "Health.Watch",
    typeof __HealthWatchDescriptor
  >
> = feedAction(
  API_ID,
  "Health.Watch",
  __HealthWatchDescriptor,
  "HealthWatch",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Health.Inspect": HealthInspect as typeof HealthInspect,
  "Health.Metrics": HealthMetrics as typeof HealthMetrics,
  "Health.Query": HealthQuery as typeof HealthQuery,
  "Health.StatusChanged": HealthStatusChanged as typeof HealthStatusChanged,
  "Health.Watch": HealthWatch as typeof HealthWatch,
} as const;
