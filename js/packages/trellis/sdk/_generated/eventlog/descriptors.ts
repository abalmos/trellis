// Generated from ./generated/contracts/manifests/trellis.eventlog@v1.json
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
import {
  CONTRACT as ACTION_ARTIFACT,
  CONTRACT_DIGEST as ACTION_DIGEST,
} from "./manifest.ts";

const ACTION_SOURCE = {
  artifact: ACTION_ARTIFACT,
  digest: ACTION_DIGEST,
} as const;

const CONTRACT_ID = "trellis.eventlog@v1" as const;

export const EventLogConsumersInspect = rpcAction(
  CONTRACT_ID,
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
  "EventLogConsumersInspect",
  ACTION_SOURCE,
);

export const EventLogConsumersQuery = rpcAction(
  CONTRACT_ID,
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
  },
  "EventLogConsumersQuery",
  ACTION_SOURCE,
);

export const EventLogInspect = rpcAction(
  CONTRACT_ID,
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
  "EventLogInspect",
  ACTION_SOURCE,
);

export const EventLogMetrics = rpcAction(
  CONTRACT_ID,
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
  },
  "EventLogMetrics",
  ACTION_SOURCE,
);

export const EventLogQuery = rpcAction(
  CONTRACT_ID,
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
  },
  "EventLogQuery",
  ACTION_SOURCE,
);

export const EventLogWatch = feedAction(
  CONTRACT_ID,
  "EventLog.Watch",
  {
    subject: "feeds.v1.EventLog.Watch",
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
