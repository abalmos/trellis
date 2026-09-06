// Generated from trellis.eventlog@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "@qlever-llc/trellis";
import * as Types from "./types.ts";
import {
  EventLogConsumersInspectRequestSchema,
  EventLogConsumersInspectResponseSchema,
  EventLogConsumersQueryRequestSchema,
  EventLogConsumersQueryResponseSchema,
  EventLogInspectRequestSchema,
  EventLogInspectResponseSchema,
  EventLogMetricsRequestSchema,
  EventLogMetricsResponseSchema,
  EventLogQueryRequestSchema,
  EventLogQueryResponseSchema,
  EventLogWatchFrameSchema,
  EventLogWatchRequestSchema,
  NotFoundErrorDataSchema,
} from "./schemas.ts";
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.eventlog@v1" as const;

const __EventLogConsumersInspectDescriptor = {
  subject: "rpc.v1.EventLog.Consumers.Inspect",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "EventLog.Consumers.Inspect",
    action: "call",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "EventLog.Consumers.Inspect";
    readonly action: "call";
  },
  input: schema<Types.EventLogConsumersInspectInput>(
    EventLogConsumersInspectRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogConsumersInspectInput>>,
  output: schema<Types.EventLogConsumersInspectOutput>(
    EventLogConsumersInspectResponseSchema,
  ) as ReturnType<typeof schema<Types.EventLogConsumersInspectOutput>>,
  callerCapabilities: ["trellis.eventlog::read"] as const,
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
export const EventLogConsumersInspect: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "EventLog.Consumers.Inspect",
    typeof __EventLogConsumersInspectDescriptor
  >
> = rpcAction(
  API_ID,
  "EventLog.Consumers.Inspect",
  __EventLogConsumersInspectDescriptor,
  "EventLogConsumersInspect",
  ACTION_SOURCE,
);

const __EventLogConsumersQueryDescriptor = {
  subject: "rpc.v1.EventLog.Consumers.Query",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "EventLog.Consumers.Query",
    action: "call",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "EventLog.Consumers.Query";
    readonly action: "call";
  },
  input: schema<Types.EventLogConsumersQueryInput>(
    EventLogConsumersQueryRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogConsumersQueryInput>>,
  output: schema<Types.EventLogConsumersQueryOutput>(
    EventLogConsumersQueryResponseSchema,
  ) as ReturnType<typeof schema<Types.EventLogConsumersQueryOutput>>,
  callerCapabilities: ["trellis.eventlog::read"] as const,
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
export const EventLogConsumersQuery: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "EventLog.Consumers.Query",
    typeof __EventLogConsumersQueryDescriptor
  >
> = rpcAction(
  API_ID,
  "EventLog.Consumers.Query",
  __EventLogConsumersQueryDescriptor,
  "EventLogConsumersQuery",
  ACTION_SOURCE,
);

const __EventLogInspectDescriptor = {
  subject: "rpc.v1.EventLog.Inspect",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "EventLog.Inspect",
    action: "call",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "EventLog.Inspect";
    readonly action: "call";
  },
  input: schema<Types.EventLogInspectInput>(
    EventLogInspectRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogInspectInput>>,
  output: schema<Types.EventLogInspectOutput>(
    EventLogInspectResponseSchema,
  ) as ReturnType<typeof schema<Types.EventLogInspectOutput>>,
  callerCapabilities: ["trellis.eventlog::read"] as const,
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
export const EventLogInspect: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "EventLog.Inspect",
    typeof __EventLogInspectDescriptor
  >
> = rpcAction(
  API_ID,
  "EventLog.Inspect",
  __EventLogInspectDescriptor,
  "EventLogInspect",
  ACTION_SOURCE,
);

const __EventLogMetricsDescriptor = {
  subject: "rpc.v1.EventLog.Metrics",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "EventLog.Metrics",
    action: "call",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "EventLog.Metrics";
    readonly action: "call";
  },
  input: schema<Types.EventLogMetricsInput>(
    EventLogMetricsRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogMetricsInput>>,
  output: schema<Types.EventLogMetricsOutput>(
    EventLogMetricsResponseSchema,
  ) as ReturnType<typeof schema<Types.EventLogMetricsOutput>>,
  callerCapabilities: ["trellis.eventlog::read"] as const,
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
export const EventLogMetrics: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "EventLog.Metrics",
    typeof __EventLogMetricsDescriptor
  >
> = rpcAction(
  API_ID,
  "EventLog.Metrics",
  __EventLogMetricsDescriptor,
  "EventLogMetrics",
  ACTION_SOURCE,
);

const __EventLogQueryDescriptor = {
  subject: "rpc.v1.EventLog.Query",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "EventLog.Query",
    action: "call",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "EventLog.Query";
    readonly action: "call";
  },
  input: schema<Types.EventLogQueryInput>(
    EventLogQueryRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogQueryInput>>,
  output: schema<Types.EventLogQueryOutput>(
    EventLogQueryResponseSchema,
  ) as ReturnType<typeof schema<Types.EventLogQueryOutput>>,
  callerCapabilities: ["trellis.eventlog::read"] as const,
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
export const EventLogQuery: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "EventLog.Query",
    typeof __EventLogQueryDescriptor
  >
> = rpcAction(
  API_ID,
  "EventLog.Query",
  __EventLogQueryDescriptor,
  "EventLogQuery",
  ACTION_SOURCE,
);

const __EventLogWatchDescriptor = {
  subject: "feed.v1.EventLog.Watch",
  permission: Object.freeze({
    apiId: "trellis.eventlog@v1",
    apiVersion: "v1",
    surfaceKind: "feed",
    surfaceName: "EventLog.Watch",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.eventlog@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "feed";
    readonly surfaceName: "EventLog.Watch";
    readonly action: "subscribe";
  },
  input: schema<Types.EventLogWatchInput>(
    EventLogWatchRequestSchema,
  ) as ReturnType<typeof schema<Types.EventLogWatchInput>>,
  event: schema<Types.EventLogWatchEvent>(
    EventLogWatchFrameSchema,
  ) as ReturnType<typeof schema<Types.EventLogWatchEvent>>,
  subscribeCapabilities: ["trellis.eventlog::stream"] as const,
} as const;
export const EventLogWatch: ReturnType<
  typeof feedAction<
    typeof API_ID,
    "EventLog.Watch",
    typeof __EventLogWatchDescriptor
  >
> = feedAction(
  API_ID,
  "EventLog.Watch",
  __EventLogWatchDescriptor,
  "EventLogWatch",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "EventLog.Consumers.Inspect":
    EventLogConsumersInspect as typeof EventLogConsumersInspect,
  "EventLog.Consumers.Query":
    EventLogConsumersQuery as typeof EventLogConsumersQuery,
  "EventLog.Inspect": EventLogInspect as typeof EventLogInspect,
  "EventLog.Metrics": EventLogMetrics as typeof EventLogMetrics,
  "EventLog.Query": EventLogQuery as typeof EventLogQuery,
  "EventLog.Watch": EventLogWatch as typeof EventLogWatch,
} as const;
