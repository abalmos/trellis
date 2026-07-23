// Generated from ./generated/contracts/manifests/trellis.jobs@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import * as Types from "./types.ts";
import {
  JobsCancelRequestSchema,
  JobsCancelResponseSchema,
  JobsDismissDLQRequestSchema,
  JobsDismissDLQResponseSchema,
  JobsGetKeyRequestSchema,
  JobsGetKeyResponseSchema,
  JobsInspectRequestSchema,
  JobsInspectResponseSchema,
  JobsListDLQRequestSchema,
  JobsListDLQResponseSchema,
  JobsListServicesRequestSchema,
  JobsListServicesResponseSchema,
  JobsMetricsRequestSchema,
  JobsMetricsResponseSchema,
  JobsQueryRequestSchema,
  JobsQueryResponseSchema,
  JobsReplayDLQRequestSchema,
  JobsReplayDLQResponseSchema,
  JobsRetryRequestSchema,
  JobsRetryResponseSchema,
  JobsWatchFrameSchema,
  JobsWatchRequestSchema,
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

const CONTRACT_ID = "trellis.jobs@v1" as const;

export const JobsCancel = rpcAction(
  CONTRACT_ID,
  "Jobs.Cancel",
  {
    subject: "rpc.v1.Jobs.Cancel",
    input: schema<Types.JobsCancelInput>(JobsCancelRequestSchema),
    output: schema<Types.JobsCancelOutput>(JobsCancelResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.mutate"] as const,
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
  "JobsCancel",
  ACTION_SOURCE,
);

export const JobsDismissDLQ = rpcAction(
  CONTRACT_ID,
  "Jobs.DismissDLQ",
  {
    subject: "rpc.v1.Jobs.DismissDLQ",
    input: schema<Types.JobsDismissDLQInput>(JobsDismissDLQRequestSchema),
    output: schema<Types.JobsDismissDLQOutput>(JobsDismissDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.mutate"] as const,
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
  "JobsDismissDLQ",
  ACTION_SOURCE,
);

export const JobsGetKey = rpcAction(
  CONTRACT_ID,
  "Jobs.GetKey",
  {
    subject: "rpc.v1.Jobs.GetKey",
    input: schema<Types.JobsGetKeyInput>(JobsGetKeyRequestSchema),
    output: schema<Types.JobsGetKeyOutput>(JobsGetKeyResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
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
  "JobsGetKey",
  ACTION_SOURCE,
);

export const JobsInspect = rpcAction(
  CONTRACT_ID,
  "Jobs.Inspect",
  {
    subject: "rpc.v1.Jobs.Inspect",
    input: schema<Types.JobsInspectInput>(JobsInspectRequestSchema),
    output: schema<Types.JobsInspectOutput>(JobsInspectResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
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
  "JobsInspect",
  ACTION_SOURCE,
);

export const JobsListDLQ = rpcAction(
  CONTRACT_ID,
  "Jobs.ListDLQ",
  {
    subject: "rpc.v1.Jobs.ListDLQ",
    input: schema<Types.JobsListDLQInput>(JobsListDLQRequestSchema),
    output: schema<Types.JobsListDLQOutput>(JobsListDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "JobsListDLQ",
  ACTION_SOURCE,
);

export const JobsListServices = rpcAction(
  CONTRACT_ID,
  "Jobs.ListServices",
  {
    subject: "rpc.v1.Jobs.ListServices",
    input: schema<Types.JobsListServicesInput>(JobsListServicesRequestSchema),
    output: schema<Types.JobsListServicesOutput>(
      JobsListServicesResponseSchema,
    ),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "JobsListServices",
  ACTION_SOURCE,
);

export const JobsMetrics = rpcAction(
  CONTRACT_ID,
  "Jobs.Metrics",
  {
    subject: "rpc.v1.Jobs.Metrics",
    input: schema<Types.JobsMetricsInput>(JobsMetricsRequestSchema),
    output: schema<Types.JobsMetricsOutput>(JobsMetricsResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "JobsMetrics",
  ACTION_SOURCE,
);

export const JobsQuery = rpcAction(
  CONTRACT_ID,
  "Jobs.Query",
  {
    subject: "rpc.v1.Jobs.Query",
    input: schema<Types.JobsQueryInput>(JobsQueryRequestSchema),
    output: schema<Types.JobsQueryOutput>(JobsQueryResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "JobsQuery",
  ACTION_SOURCE,
);

export const JobsReplayDLQ = rpcAction(
  CONTRACT_ID,
  "Jobs.ReplayDLQ",
  {
    subject: "rpc.v1.Jobs.ReplayDLQ",
    input: schema<Types.JobsReplayDLQInput>(JobsReplayDLQRequestSchema),
    output: schema<Types.JobsReplayDLQOutput>(JobsReplayDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.mutate"] as const,
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
  "JobsReplayDLQ",
  ACTION_SOURCE,
);

export const JobsRetry = rpcAction(
  CONTRACT_ID,
  "Jobs.Retry",
  {
    subject: "rpc.v1.Jobs.Retry",
    input: schema<Types.JobsRetryInput>(JobsRetryRequestSchema),
    output: schema<Types.JobsRetryOutput>(JobsRetryResponseSchema),
    callerCapabilities: ["trellis.jobs::admin.mutate"] as const,
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
  "JobsRetry",
  ACTION_SOURCE,
);

export const JobsWatch = feedAction(
  CONTRACT_ID,
  "Jobs.Watch",
  {
    subject: "feeds.v1.Jobs.Watch",
    input: schema<Types.JobsWatchInput>(JobsWatchRequestSchema),
    event: schema<Types.JobsWatchEvent>(JobsWatchFrameSchema),
    subscribeCapabilities: ["trellis.jobs::admin.stream"] as const,
  },
  "JobsWatch",
  ACTION_SOURCE,
);
