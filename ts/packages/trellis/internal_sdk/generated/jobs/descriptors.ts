// Generated from trellis.jobs@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../generated.ts";
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

const __JobsCancelDescriptor = {
  subject: "rpc.v1.Jobs.Cancel",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.Cancel",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.Cancel";
    readonly action: "call";
  },
  input: schema<Types.JobsCancelInput>(JobsCancelRequestSchema) as ReturnType<
    typeof schema<Types.JobsCancelInput>
  >,
  output: schema<Types.JobsCancelOutput>(
    JobsCancelResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsCancelOutput>>,
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
export const JobsCancel: ReturnType<
  typeof rpcAction<typeof API_ID, "Jobs.Cancel", typeof __JobsCancelDescriptor>
> = rpcAction(
  API_ID,
  "Jobs.Cancel",
  __JobsCancelDescriptor,
  "JobsCancel",
  ACTION_SOURCE,
);

const __JobsDismissDLQDescriptor = {
  subject: "rpc.v1.Jobs.DismissDLQ",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.DismissDLQ",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.DismissDLQ";
    readonly action: "call";
  },
  input: schema<Types.JobsDismissDLQInput>(
    JobsDismissDLQRequestSchema,
  ) as ReturnType<typeof schema<Types.JobsDismissDLQInput>>,
  output: schema<Types.JobsDismissDLQOutput>(
    JobsDismissDLQResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsDismissDLQOutput>>,
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
export const JobsDismissDLQ: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.DismissDLQ",
    typeof __JobsDismissDLQDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.DismissDLQ",
  __JobsDismissDLQDescriptor,
  "JobsDismissDLQ",
  ACTION_SOURCE,
);

const __JobsGetKeyDescriptor = {
  subject: "rpc.v1.Jobs.GetKey",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.GetKey",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.GetKey";
    readonly action: "call";
  },
  input: schema<Types.JobsGetKeyInput>(JobsGetKeyRequestSchema) as ReturnType<
    typeof schema<Types.JobsGetKeyInput>
  >,
  output: schema<Types.JobsGetKeyOutput>(
    JobsGetKeyResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsGetKeyOutput>>,
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
export const JobsGetKey: ReturnType<
  typeof rpcAction<typeof API_ID, "Jobs.GetKey", typeof __JobsGetKeyDescriptor>
> = rpcAction(
  API_ID,
  "Jobs.GetKey",
  __JobsGetKeyDescriptor,
  "JobsGetKey",
  ACTION_SOURCE,
);

const __JobsInspectDescriptor = {
  subject: "rpc.v1.Jobs.Inspect",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.Inspect",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.Inspect";
    readonly action: "call";
  },
  input: schema<Types.JobsInspectInput>(JobsInspectRequestSchema) as ReturnType<
    typeof schema<Types.JobsInspectInput>
  >,
  output: schema<Types.JobsInspectOutput>(
    JobsInspectResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsInspectOutput>>,
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
export const JobsInspect: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.Inspect",
    typeof __JobsInspectDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.Inspect",
  __JobsInspectDescriptor,
  "JobsInspect",
  ACTION_SOURCE,
);

const __JobsListDLQDescriptor = {
  subject: "rpc.v1.Jobs.ListDLQ",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.ListDLQ",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.ListDLQ";
    readonly action: "call";
  },
  input: schema<Types.JobsListDLQInput>(JobsListDLQRequestSchema) as ReturnType<
    typeof schema<Types.JobsListDLQInput>
  >,
  output: schema<Types.JobsListDLQOutput>(
    JobsListDLQResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsListDLQOutput>>,
  callerCapabilities: ["trellis.jobs::read"] as const,
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
export const JobsListDLQ: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.ListDLQ",
    typeof __JobsListDLQDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.ListDLQ",
  __JobsListDLQDescriptor,
  "JobsListDLQ",
  ACTION_SOURCE,
);

const __JobsListServicesDescriptor = {
  subject: "rpc.v1.Jobs.ListServices",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.ListServices",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.ListServices";
    readonly action: "call";
  },
  input: schema<Types.JobsListServicesInput>(
    JobsListServicesRequestSchema,
  ) as ReturnType<typeof schema<Types.JobsListServicesInput>>,
  output: schema<Types.JobsListServicesOutput>(
    JobsListServicesResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsListServicesOutput>>,
  callerCapabilities: ["trellis.jobs::read"] as const,
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
export const JobsListServices: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.ListServices",
    typeof __JobsListServicesDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.ListServices",
  __JobsListServicesDescriptor,
  "JobsListServices",
  ACTION_SOURCE,
);

const __JobsMetricsDescriptor = {
  subject: "rpc.v1.Jobs.Metrics",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.Metrics",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.Metrics";
    readonly action: "call";
  },
  input: schema<Types.JobsMetricsInput>(JobsMetricsRequestSchema) as ReturnType<
    typeof schema<Types.JobsMetricsInput>
  >,
  output: schema<Types.JobsMetricsOutput>(
    JobsMetricsResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsMetricsOutput>>,
  callerCapabilities: ["trellis.jobs::read"] as const,
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
export const JobsMetrics: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.Metrics",
    typeof __JobsMetricsDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.Metrics",
  __JobsMetricsDescriptor,
  "JobsMetrics",
  ACTION_SOURCE,
);

const __JobsQueryDescriptor = {
  subject: "rpc.v1.Jobs.Query",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.Query",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.Query";
    readonly action: "call";
  },
  input: schema<Types.JobsQueryInput>(JobsQueryRequestSchema) as ReturnType<
    typeof schema<Types.JobsQueryInput>
  >,
  output: schema<Types.JobsQueryOutput>(JobsQueryResponseSchema) as ReturnType<
    typeof schema<Types.JobsQueryOutput>
  >,
  callerCapabilities: ["trellis.jobs::read"] as const,
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
export const JobsQuery: ReturnType<
  typeof rpcAction<typeof API_ID, "Jobs.Query", typeof __JobsQueryDescriptor>
> = rpcAction(
  API_ID,
  "Jobs.Query",
  __JobsQueryDescriptor,
  "JobsQuery",
  ACTION_SOURCE,
);

const __JobsReplayDLQDescriptor = {
  subject: "rpc.v1.Jobs.ReplayDLQ",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.ReplayDLQ",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.ReplayDLQ";
    readonly action: "call";
  },
  input: schema<Types.JobsReplayDLQInput>(
    JobsReplayDLQRequestSchema,
  ) as ReturnType<typeof schema<Types.JobsReplayDLQInput>>,
  output: schema<Types.JobsReplayDLQOutput>(
    JobsReplayDLQResponseSchema,
  ) as ReturnType<typeof schema<Types.JobsReplayDLQOutput>>,
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
export const JobsReplayDLQ: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Jobs.ReplayDLQ",
    typeof __JobsReplayDLQDescriptor
  >
> = rpcAction(
  API_ID,
  "Jobs.ReplayDLQ",
  __JobsReplayDLQDescriptor,
  "JobsReplayDLQ",
  ACTION_SOURCE,
);

const __JobsRetryDescriptor = {
  subject: "rpc.v1.Jobs.Retry",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Jobs.Retry",
    action: "call",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Jobs.Retry";
    readonly action: "call";
  },
  input: schema<Types.JobsRetryInput>(JobsRetryRequestSchema) as ReturnType<
    typeof schema<Types.JobsRetryInput>
  >,
  output: schema<Types.JobsRetryOutput>(JobsRetryResponseSchema) as ReturnType<
    typeof schema<Types.JobsRetryOutput>
  >,
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
export const JobsRetry: ReturnType<
  typeof rpcAction<typeof API_ID, "Jobs.Retry", typeof __JobsRetryDescriptor>
> = rpcAction(
  API_ID,
  "Jobs.Retry",
  __JobsRetryDescriptor,
  "JobsRetry",
  ACTION_SOURCE,
);

const __JobsWatchDescriptor = {
  subject: "feed.v1.Jobs.Watch",
  permission: Object.freeze({
    apiId: "trellis.jobs@v1",
    apiVersion: "v1",
    surfaceKind: "feed",
    surfaceName: "Jobs.Watch",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.jobs@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "feed";
    readonly surfaceName: "Jobs.Watch";
    readonly action: "subscribe";
  },
  input: schema<Types.JobsWatchInput>(JobsWatchRequestSchema) as ReturnType<
    typeof schema<Types.JobsWatchInput>
  >,
  event: schema<Types.JobsWatchEvent>(JobsWatchFrameSchema) as ReturnType<
    typeof schema<Types.JobsWatchEvent>
  >,
  subscribeCapabilities: ["trellis.jobs::stream"] as const,
} as const;
export const JobsWatch: ReturnType<
  typeof feedAction<typeof API_ID, "Jobs.Watch", typeof __JobsWatchDescriptor>
> = feedAction(
  API_ID,
  "Jobs.Watch",
  __JobsWatchDescriptor,
  "JobsWatch",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Jobs.Cancel": JobsCancel as typeof JobsCancel,
  "Jobs.DismissDLQ": JobsDismissDLQ as typeof JobsDismissDLQ,
  "Jobs.GetKey": JobsGetKey as typeof JobsGetKey,
  "Jobs.Inspect": JobsInspect as typeof JobsInspect,
  "Jobs.ListDLQ": JobsListDLQ as typeof JobsListDLQ,
  "Jobs.ListServices": JobsListServices as typeof JobsListServices,
  "Jobs.Metrics": JobsMetrics as typeof JobsMetrics,
  "Jobs.Query": JobsQuery as typeof JobsQuery,
  "Jobs.ReplayDLQ": JobsReplayDLQ as typeof JobsReplayDLQ,
  "Jobs.Retry": JobsRetry as typeof JobsRetry,
  "Jobs.Watch": JobsWatch as typeof JobsWatch,
} as const;
