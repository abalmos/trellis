// Generated from ./generated/protocol/apis/trellis.health@v1.json
import type { SerializableErrorData } from "../../../contracts.ts";
import { TrellisError } from "../../../errors/index.ts";
import { NotFoundErrorDataSchema } from "./schemas.ts";

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

export type NotFoundErrorData =
  & SerializableErrorData
  & ({
    context?: { [k: string]: unknown };
    id: string;
    message: string;
    resource: string;
    traceId?: string;
    type: "NotFoundError";
  });
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

export type UnexpectedErrorData = SerializableErrorData;
export class UnexpectedError extends TrellisError<UnexpectedErrorData> {
  override readonly name = "UnexpectedError" as const;
  readonly data: UnexpectedErrorData;
  constructor(data: UnexpectedErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: UnexpectedErrorData): UnexpectedError {
    return new UnexpectedError(data);
  }
  override toSerializable(): UnexpectedErrorData {
    return this.data;
  }
}

export type ValidationErrorData = SerializableErrorData;
export class ValidationError extends TrellisError<ValidationErrorData> {
  override readonly name = "ValidationError" as const;
  readonly data: ValidationErrorData;
  constructor(data: ValidationErrorData) {
    super(data.message, {
      id: data.id,
      ...(data.context !== undefined ? { context: data.context } : {}),
    });
    this.data = data;
  }
  static fromSerializable(data: ValidationErrorData): ValidationError {
    return new ValidationError(data);
  }
  override toSerializable(): ValidationErrorData {
    return this.data;
  }
}
