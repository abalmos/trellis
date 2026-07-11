// Generated from ./generated/contracts/manifests/trellis.health@v1.json
import type { TrellisAPI } from "../../../contracts.ts";
import { schema } from "../../../contracts.ts";
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

export const OWNED_API = {
  rpc: {
    "Health.Inspect": {
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
    "Health.Metrics": {
      subject: "rpc.v1.Health.Metrics",
      input: schema<Types.HealthMetricsInput>(HealthMetricsRequestSchema),
      output: schema<Types.HealthMetricsOutput>(HealthMetricsResponseSchema),
      callerCapabilities: ["trellis.health::read"] as const,
      errors: ["UnexpectedError", "ValidationError"] as const,
      declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
    },
    "Health.Query": {
      subject: "rpc.v1.Health.Query",
      input: schema<Types.HealthQueryInput>(HealthQueryRequestSchema),
      output: schema<Types.HealthQueryOutput>(HealthQueryResponseSchema),
      callerCapabilities: ["trellis.health::read"] as const,
      errors: ["UnexpectedError", "ValidationError"] as const,
      declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
    },
  },
  operations: {},
  events: {
    "Health.StatusChanged": {
      subject: "events.v1.Health.StatusChanged",
      event: schema<Types.HealthStatusChangedEvent>(
        HealthStatusChangedEventSchema,
      ),
      publishCapabilities: [] as const,
      subscribeCapabilities: ["trellis.health::read"] as const,
    },
  },
  feeds: {
    "Health.Watch": {
      subject: "feed.v1.Health.Watch",
      input: schema<Types.HealthWatchInput>(HealthWatchRequestSchema),
      event: schema<Types.HealthWatchEvent>(HealthWatchFrameSchema),
      subscribeCapabilities: ["trellis.health::read"] as const,
    },
  },
  subjects: {},
} satisfies TrellisAPI;
