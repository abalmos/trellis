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

const CONTRACT_ID = "trellis.eventlog@v1" as const;

export const EventLogConsumersInspect = rpcAction(
  CONTRACT_ID,
  "EventLog.Consumers.Inspect",
  {
    subject: "rpc.v1.EventLog.Consumers.Inspect",
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
);

export const EventLogConsumersQuery = rpcAction(
  CONTRACT_ID,
  "EventLog.Consumers.Query",
  {
    subject: "rpc.v1.EventLog.Consumers.Query",
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
);

export const EventLogInspect = rpcAction(CONTRACT_ID, "EventLog.Inspect", {
  subject: "rpc.v1.EventLog.Inspect",
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
}, "EventLogInspect");

export const EventLogMetrics = rpcAction(CONTRACT_ID, "EventLog.Metrics", {
  subject: "rpc.v1.EventLog.Metrics",
  input: schema<Types.EventLogMetricsInput>(EventLogMetricsRequestSchema),
  output: schema<Types.EventLogMetricsOutput>(EventLogMetricsResponseSchema),
  callerCapabilities: ["trellis.eventlog::events.read"] as const,
  errors: ["UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
}, "EventLogMetrics");

export const EventLogQuery = rpcAction(CONTRACT_ID, "EventLog.Query", {
  subject: "rpc.v1.EventLog.Query",
  input: schema<Types.EventLogQueryInput>(EventLogQueryRequestSchema),
  output: schema<Types.EventLogQueryOutput>(EventLogQueryResponseSchema),
  callerCapabilities: ["trellis.eventlog::events.read"] as const,
  errors: ["UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
}, "EventLogQuery");

export const EventLogWatch = feedAction(CONTRACT_ID, "EventLog.Watch", {
  subject: "feeds.v1.EventLog.Watch",
  input: schema<Types.EventLogWatchInput>(EventLogWatchRequestSchema),
  event: schema<Types.EventLogWatchEvent>(EventLogWatchFrameSchema),
  subscribeCapabilities: ["trellis.eventlog::events.stream"] as const,
}, "EventLogWatch");
