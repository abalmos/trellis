// Generated from ./generated/protocol/apis/trellis.eventlog@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
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

export const EventLogConsumersInspect = rpcAction(
  API_ID,
  "EventLog.Consumers.Inspect",
  {
    subject: "rpc.v1.EventLog.Consumers.Inspect",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "EventLog.Consumers.Inspect",
      action: "call",
    }),
    input: schema<Types.EventLogConsumersInspectInput>(
      EventLogConsumersInspectRequestSchema,
    ),
    output: schema<Types.EventLogConsumersInspectOutput>(
      EventLogConsumersInspectResponseSchema,
    ),
    callerCapabilities: ["trellis.eventlog::events.read"] as const,
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
  "EventLogConsumersInspect",
  ACTION_SOURCE,
);

export const EventLogConsumersQuery = rpcAction(
  API_ID,
  "EventLog.Consumers.Query",
  {
    subject: "rpc.v1.EventLog.Consumers.Query",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "EventLog.Consumers.Query",
      action: "call",
    }),
    input: schema<Types.EventLogConsumersQueryInput>(
      EventLogConsumersQueryRequestSchema,
    ),
    output: schema<Types.EventLogConsumersQueryOutput>(
      EventLogConsumersQueryResponseSchema,
    ),
    callerCapabilities: ["trellis.eventlog::events.read"] as const,
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
  "EventLogConsumersQuery",
  ACTION_SOURCE,
);

export const EventLogInspect = rpcAction(
  API_ID,
  "EventLog.Inspect",
  {
    subject: "rpc.v1.EventLog.Inspect",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "EventLog.Inspect",
      action: "call",
    }),
    input: schema<Types.EventLogInspectInput>(EventLogInspectRequestSchema),
    output: schema<Types.EventLogInspectOutput>(EventLogInspectResponseSchema),
    callerCapabilities: ["trellis.eventlog::events.read"] as const,
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
  "EventLogInspect",
  ACTION_SOURCE,
);

export const EventLogMetrics = rpcAction(
  API_ID,
  "EventLog.Metrics",
  {
    subject: "rpc.v1.EventLog.Metrics",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "EventLog.Metrics",
      action: "call",
    }),
    input: schema<Types.EventLogMetricsInput>(EventLogMetricsRequestSchema),
    output: schema<Types.EventLogMetricsOutput>(EventLogMetricsResponseSchema),
    callerCapabilities: ["trellis.eventlog::events.read"] as const,
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
  "EventLogMetrics",
  ACTION_SOURCE,
);

export const EventLogQuery = rpcAction(
  API_ID,
  "EventLog.Query",
  {
    subject: "rpc.v1.EventLog.Query",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "EventLog.Query",
      action: "call",
    }),
    input: schema<Types.EventLogQueryInput>(EventLogQueryRequestSchema),
    output: schema<Types.EventLogQueryOutput>(EventLogQueryResponseSchema),
    callerCapabilities: ["trellis.eventlog::events.read"] as const,
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
  "EventLogQuery",
  ACTION_SOURCE,
);

export const EventLogWatch = feedAction(
  API_ID,
  "EventLog.Watch",
  {
    subject: "feed.v1.EventLog.Watch",
    permission: Object.freeze({
      apiId: "trellis.eventlog@v1",
      apiVersion: "v1",
      surfaceKind: "feed",
      surfaceName: "EventLog.Watch",
      action: "subscribe",
    }),
    input: schema<Types.EventLogWatchInput>(EventLogWatchRequestSchema),
    event: schema<Types.EventLogWatchEvent>(EventLogWatchFrameSchema),
    subscribeCapabilities: ["trellis.eventlog::events.stream"] as const,
  },
  "EventLogWatch",
  ACTION_SOURCE,
);
