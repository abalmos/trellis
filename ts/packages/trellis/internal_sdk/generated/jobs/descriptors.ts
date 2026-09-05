// Generated from ./rust/crates/jobs-runtime/.trellis/artifacts/apis/trellis.jobs@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "@qlever-llc/trellis";
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
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.jobs@v1" as const;

export const JobsCancel = rpcAction(
  API_ID,
  "Jobs.Cancel",
  {
    subject: "rpc.v1.Jobs.Cancel",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.Cancel",
      action: "call",
    }),
    input: schema<Types.JobsCancelInput>(JobsCancelRequestSchema),
    output: schema<Types.JobsCancelOutput>(JobsCancelResponseSchema),
    callerCapabilities: ["trellis.jobs::mutate"] as const,
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
  "JobsCancel",
  ACTION_SOURCE,
);

export const JobsDismissDLQ = rpcAction(
  API_ID,
  "Jobs.DismissDLQ",
  {
    subject: "rpc.v1.Jobs.DismissDLQ",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.DismissDLQ",
      action: "call",
    }),
    input: schema<Types.JobsDismissDLQInput>(JobsDismissDLQRequestSchema),
    output: schema<Types.JobsDismissDLQOutput>(JobsDismissDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::mutate"] as const,
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
  "JobsDismissDLQ",
  ACTION_SOURCE,
);

export const JobsGetKey = rpcAction(
  API_ID,
  "Jobs.GetKey",
  {
    subject: "rpc.v1.Jobs.GetKey",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.GetKey",
      action: "call",
    }),
    input: schema<Types.JobsGetKeyInput>(JobsGetKeyRequestSchema),
    output: schema<Types.JobsGetKeyOutput>(JobsGetKeyResponseSchema),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsGetKey",
  ACTION_SOURCE,
);

export const JobsInspect = rpcAction(
  API_ID,
  "Jobs.Inspect",
  {
    subject: "rpc.v1.Jobs.Inspect",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.Inspect",
      action: "call",
    }),
    input: schema<Types.JobsInspectInput>(JobsInspectRequestSchema),
    output: schema<Types.JobsInspectOutput>(JobsInspectResponseSchema),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsInspect",
  ACTION_SOURCE,
);

export const JobsListDLQ = rpcAction(
  API_ID,
  "Jobs.ListDLQ",
  {
    subject: "rpc.v1.Jobs.ListDLQ",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.ListDLQ",
      action: "call",
    }),
    input: schema<Types.JobsListDLQInput>(JobsListDLQRequestSchema),
    output: schema<Types.JobsListDLQOutput>(JobsListDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsListDLQ",
  ACTION_SOURCE,
);

export const JobsListServices = rpcAction(
  API_ID,
  "Jobs.ListServices",
  {
    subject: "rpc.v1.Jobs.ListServices",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.ListServices",
      action: "call",
    }),
    input: schema<Types.JobsListServicesInput>(JobsListServicesRequestSchema),
    output: schema<Types.JobsListServicesOutput>(
      JobsListServicesResponseSchema,
    ),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsListServices",
  ACTION_SOURCE,
);

export const JobsMetrics = rpcAction(
  API_ID,
  "Jobs.Metrics",
  {
    subject: "rpc.v1.Jobs.Metrics",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.Metrics",
      action: "call",
    }),
    input: schema<Types.JobsMetricsInput>(JobsMetricsRequestSchema),
    output: schema<Types.JobsMetricsOutput>(JobsMetricsResponseSchema),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsMetrics",
  ACTION_SOURCE,
);

export const JobsQuery = rpcAction(
  API_ID,
  "Jobs.Query",
  {
    subject: "rpc.v1.Jobs.Query",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.Query",
      action: "call",
    }),
    input: schema<Types.JobsQueryInput>(JobsQueryRequestSchema),
    output: schema<Types.JobsQueryOutput>(JobsQueryResponseSchema),
    callerCapabilities: ["trellis.jobs::read"] as const,
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
  "JobsQuery",
  ACTION_SOURCE,
);

export const JobsReplayDLQ = rpcAction(
  API_ID,
  "Jobs.ReplayDLQ",
  {
    subject: "rpc.v1.Jobs.ReplayDLQ",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.ReplayDLQ",
      action: "call",
    }),
    input: schema<Types.JobsReplayDLQInput>(JobsReplayDLQRequestSchema),
    output: schema<Types.JobsReplayDLQOutput>(JobsReplayDLQResponseSchema),
    callerCapabilities: ["trellis.jobs::mutate"] as const,
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
  "JobsReplayDLQ",
  ACTION_SOURCE,
);

export const JobsRetry = rpcAction(
  API_ID,
  "Jobs.Retry",
  {
    subject: "rpc.v1.Jobs.Retry",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Jobs.Retry",
      action: "call",
    }),
    input: schema<Types.JobsRetryInput>(JobsRetryRequestSchema),
    output: schema<Types.JobsRetryOutput>(JobsRetryResponseSchema),
    callerCapabilities: ["trellis.jobs::mutate"] as const,
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
  "JobsRetry",
  ACTION_SOURCE,
);

export const JobsWatch = feedAction(
  API_ID,
  "Jobs.Watch",
  {
    subject: "feed.v1.Jobs.Watch",
    permission: Object.freeze({
      apiId: "trellis.jobs@v1",
      apiVersion: "v1",
      surfaceKind: "feed",
      surfaceName: "Jobs.Watch",
      action: "subscribe",
    }),
    input: schema<Types.JobsWatchInput>(JobsWatchRequestSchema),
    event: schema<Types.JobsWatchEvent>(JobsWatchFrameSchema),
    subscribeCapabilities: ["trellis.jobs::stream"] as const,
  },
  "JobsWatch",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Jobs.Cancel": JobsCancel,
  "Jobs.DismissDLQ": JobsDismissDLQ,
  "Jobs.GetKey": JobsGetKey,
  "Jobs.Inspect": JobsInspect,
  "Jobs.ListDLQ": JobsListDLQ,
  "Jobs.ListServices": JobsListServices,
  "Jobs.Metrics": JobsMetrics,
  "Jobs.Query": JobsQuery,
  "Jobs.ReplayDLQ": JobsReplayDLQ,
  "Jobs.Retry": JobsRetry,
  "Jobs.Watch": JobsWatch,
} as const;
