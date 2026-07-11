// Generated from ./generated/contracts/manifests/trellis.health@v1.json
import type {
  AsyncResult,
  BaseError,
  EventListenerContext,
  HandlerTrellis,
  MaybeAsync,
  Result,
  RpcHandlerContext,
  SessionCaller,
  TrellisErrorInstance,
  TrellisEventMessage,
  UnexpectedError,
  ValidationError,
} from "../../../index.ts";

import type { Api } from "./api.ts";

import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";

import { NotFoundErrorDataSchema } from "./schemas.ts";

export type HandlerClient = HandlerTrellis<Api>;

export const CONTRACT_ID = "trellis.health@v1" as const;
export const CONTRACT_DIGEST =
  "cQEMecQZAVYyUTdZFhXd2VRIG7962g_ae9TSkPmUoPk" as const;

export type HealthHeartbeatSample = {
  checks: Array<
    {
      error?: string;
      info?: {};
      latencyMs: number;
      name: string;
      status: "ok" | "failed";
      summary?: string;
    }
  >;
  participant: {
    contractDigest: string;
    contractId: string;
    info?: {};
    instanceId: string;
    kind: "service" | "device";
    name: string;
    publishIntervalMs: number;
    runtime: "deno" | "node" | "rust" | "unknown";
    runtimeVersion?: string;
    startedAt: string;
    version?: string;
  };
  reportedStatus: "healthy" | "degraded" | "unhealthy";
  sample: { id: string; time: string };
  summary?: string;
};

export type HealthInspectInput = {
  contractId: string;
  historyLimit?: number;
  historySince?: string;
  instanceId?: string;
  participantKind: "service" | "device";
};
export type HealthInspectOutput = {
  asOf: string;
  history: Array<
    {
      checks: Array<{ name: string; status: "ok" | "failed" }>;
      effectiveStatus: "healthy" | "degraded" | "unhealthy" | "offline";
      endedAt?: string;
      instanceId: string;
      intervalId: number;
      reason:
        | "first-sample"
        | "heartbeat-change"
        | "heartbeat-resumed"
        | "deadline-expired";
      reportedStatus: "healthy" | "degraded" | "unhealthy";
      startedAt: string;
    }
  >;
  instances: Array<
    {
      ageMs: number;
      contractDigest: string;
      deploymentId: string;
      effectiveStatus: "healthy" | "degraded" | "unhealthy" | "offline";
      heartbeatDeadline: string;
      instanceId: string;
      latestSample: HealthHeartbeatSample;
      observedAt: string;
      reportedStatus: "healthy" | "degraded" | "unhealthy";
      startedAt: string;
    }
  >;
  participant: {
    contractId: string;
    effectiveStatus: "healthy" | "degraded" | "unhealthy" | "offline";
    offlineInstances: number;
    onlineInstances: number;
    participantKind: "service" | "device";
    participantName: string;
  };
  projection: {
    completeSince?: string;
    gapDetected: boolean;
    lastStreamSequence: number;
    retainedFrom?: string;
    revision: number;
  };
};

export type HealthMetricsInput = {
  checkNames?: Array<string>;
  contractId: string;
  end: string;
  instanceIds?: Array<string>;
  participantKind: "service" | "device";
  start: string;
  stepMs: number;
};
export type HealthMetricsOutput = {
  asOf: string;
  projection: {
    completeSince?: string;
    gapDetected: boolean;
    lastStreamSequence: number;
    retainedFrom?: string;
    revision: number;
  };
  series: Array<
    {
      buckets: Array<
        {
          checks: Array<
            {
              failedCount: number;
              latencyAverageMs: number;
              latencyMaxMs: number;
              name: string;
              okCount: number;
              sampleCount: number;
            }
          >;
          degradedMs: number;
          end: string;
          healthyMs: number;
          observedMs: number;
          offlineMs: number;
          sampleCount: number;
          start: string;
          unhealthyMs: number;
        }
      >;
      contractId: string;
      instanceId: string;
      participantKind: "service" | "device";
    }
  >;
  summary: {
    availability?: number;
    observedMs: number;
    onlineMs: number;
    sampleCount: number;
    transitions: number;
  };
};

export type HealthQueryInput = {
  contractIds?: Array<string>;
  deploymentIds?: Array<string>;
  limit?: number;
  offset?: number;
  participantKinds?: Array<"service" | "device">;
  search?: string;
  statuses?: Array<"healthy" | "degraded" | "unhealthy" | "offline">;
};
export type HealthQueryOutput = {
  asOf: string;
  count: number;
  entries: Array<
    {
      contractDigests: Array<string>;
      contractId: string;
      deploymentIds: Array<string>;
      effectiveStatus: "healthy" | "degraded" | "unhealthy" | "offline";
      lastSeenAt: string;
      offlineInstances: number;
      onlineInstances: number;
      participantKind: "service" | "device";
      participantName: string;
      runtimes: Array<string>;
      versions: Array<string>;
    }
  >;
  limit: number;
  offset: number;
  projection: {
    completeSince?: string;
    gapDetected: boolean;
    lastStreamSequence: number;
    retainedFrom?: string;
    revision: number;
  };
};

export type HealthStatusChangedEvent = {
  changedAt: string;
  header: { id: string; time: string };
  lastSeenAt: string;
  participant: {
    contractId: string;
    deploymentId: string;
    instanceId: string;
    kind: "service" | "device";
    name: string;
  };
  previousStatus: "healthy" | "degraded" | "unhealthy" | "offline";
  reason: "heartbeat-change" | "heartbeat-resumed" | "deadline-expired";
  reportedStatus: "healthy" | "degraded" | "unhealthy";
  status: "healthy" | "degraded" | "unhealthy" | "offline";
  summary?: string;
};
export type HealthStatusChangedEventMessage = TrellisEventMessage<
  HealthStatusChangedEvent
>;
export type HealthStatusChangedEventHandler = (
  args: {
    event: HealthStatusChangedEvent;
    context: EventListenerContext;
    client: HandlerClient;
  },
) => MaybeAsync<void, BaseError>;

export type HealthWatchInput = {
  contractIds?: Array<string>;
  deploymentIds?: Array<string>;
  instanceIds?: Array<string>;
  participantKinds?: Array<"service" | "device">;
};
export type HealthWatchEvent = { projectionRevision: number; type: "ready" } | {
  changes?: Array<
    {
      contractId: string;
      deploymentId: string;
      instanceId: string;
      participantKind: "service" | "device";
    }
  >;
  projectionRevision: number;
  type: "healthInvalidated";
};
export type HealthWatchFeedHandler = (
  context: {
    input: HealthWatchInput;
    caller: SessionCaller;
    signal: AbortSignal;
    emit(
      event: HealthWatchEvent,
    ): AsyncResult<void, ValidationError | UnexpectedError>;
    client: HandlerClient;
  },
) => unknown | Promise<unknown>;

export type NotFoundErrorData = {
  context?: { [k: string]: unknown };
  id: string;
  message: string;
  resource: string;
  traceId?: string;
  type: "NotFoundError";
};
export class NotFoundError extends TrellisError<NotFoundErrorData> {
  static readonly schema = NotFoundErrorDataSchema;
  override readonly name = "NotFoundError" as const;
  readonly data: NotFoundErrorData;

  constructor(data: NotFoundErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }

  static fromSerializable(data: NotFoundErrorData): NotFoundError {
    return new NotFoundError(data);
  }

  override toSerializable(): NotFoundErrorData {
    return this.data;
  }
}

export interface RpcMap {
  "Health.Inspect": { input: HealthInspectInput; output: HealthInspectOutput };
  "Health.Metrics": { input: HealthMetricsInput; output: HealthMetricsOutput };
  "Health.Query": { input: HealthQueryInput; output: HealthQueryOutput };
}

export type HealthInspectHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type HealthInspectHandlerResult = Result<
  HealthInspectOutput,
  HealthInspectHandlerError
>;
export type HealthInspectHandler = (
  args: {
    input: HealthInspectInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => HealthInspectHandlerResult | Promise<HealthInspectHandlerResult>;
export type HealthMetricsHandlerError = TrellisErrorInstance;
export type HealthMetricsHandlerResult = Result<
  HealthMetricsOutput,
  HealthMetricsHandlerError
>;
export type HealthMetricsHandler = (
  args: {
    input: HealthMetricsInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => HealthMetricsHandlerResult | Promise<HealthMetricsHandlerResult>;
export type HealthQueryHandlerError = TrellisErrorInstance;
export type HealthQueryHandlerResult = Result<
  HealthQueryOutput,
  HealthQueryHandlerError
>;
export type HealthQueryHandler = (
  args: {
    input: HealthQueryInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => HealthQueryHandlerResult | Promise<HealthQueryHandlerResult>;

export interface EventMap {
  "Health.StatusChanged": { event: HealthStatusChangedEvent };
}

export interface FeedMap {
  "Health.Watch": { input: HealthWatchInput; event: HealthWatchEvent };
}

export interface SubjectMap {
}
