// Generated from ./generated/contracts/manifests/trellis.jobs@v1.json
import type {
  AsyncResult,
  BaseError,
  HandlerTrellis,
  Result,
  RpcHandlerContext,
  SessionCaller,
  TrellisErrorInstance,
  UnexpectedError,
  ValidationError,
} from "../../../index.ts";

import type { Api } from "./api.ts";

import { type SerializableErrorData, TrellisError } from "../../../index.ts";

import { NotFoundErrorDataSchema } from "./schemas.ts";

export type HandlerClient = HandlerTrellis<Api>;

export const CONTRACT_ID = "trellis.jobs@v1" as const;
export const CONTRACT_DIGEST =
  "w3tzV0hwJkaIXCeSlpZQq7hgsEPDlghrc0iOs_Bpxpg" as const;

export type JobsCancelInput = { id: string; reason?: string };
export type JobsCancelOutput = {
  job: {
    completedAt?: string;
    concurrency?: {
      heartbeatAt?: string;
      key: string;
      keyHash: string;
      leaseExpiresAt?: string;
      staleTakeoverCount?: number;
    };
    context: {
      requestId: string;
      traceId: string;
      traceparent: string;
      tracestate?: string;
    };
    createdAt: string;
    deadline?: string;
    errorDetail?: {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    };
    id: string;
    lastError?: string;
    lineage?: {
      operationId?: string;
      parentJobId?: string;
      relatedKeys?: Array<string>;
      rootJobId?: string;
    };
    logs?: Array<
      { level: "info" | "warn" | "error"; message: string; timestamp: string }
    >;
    maxTries: number;
    payload: unknown;
    progress?: {
      current?: number;
      message?: string;
      step?: string;
      total?: number;
    };
    queuePolicy?: {
      existingJobId?: string;
      outcome: string;
      reason?: string;
      replacedJobId?: string;
    };
    result?: unknown;
    service: string;
    startedAt?: string;
    state:
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed";
    tries: number;
    trigger?: {
      id?: string;
      kind:
        | "schedule"
        | "operation"
        | "rpc"
        | "event"
        | "manualReplay"
        | "serviceCode"
        | "parentJob";
      operationId?: string;
      parentJobId?: string;
      requestId?: string;
      subject?: string;
      traceId?: string;
    };
    type: string;
    updatedAt: string;
  };
};

export type JobsDismissDLQInput = { id: string; reason?: string };
export type JobsDismissDLQOutput = {
  job: {
    completedAt?: string;
    concurrency?: {
      heartbeatAt?: string;
      key: string;
      keyHash: string;
      leaseExpiresAt?: string;
      staleTakeoverCount?: number;
    };
    context: {
      requestId: string;
      traceId: string;
      traceparent: string;
      tracestate?: string;
    };
    createdAt: string;
    deadline?: string;
    errorDetail?: {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    };
    id: string;
    lastError?: string;
    lineage?: {
      operationId?: string;
      parentJobId?: string;
      relatedKeys?: Array<string>;
      rootJobId?: string;
    };
    logs?: Array<
      { level: "info" | "warn" | "error"; message: string; timestamp: string }
    >;
    maxTries: number;
    payload: unknown;
    progress?: {
      current?: number;
      message?: string;
      step?: string;
      total?: number;
    };
    queuePolicy?: {
      existingJobId?: string;
      outcome: string;
      reason?: string;
      replacedJobId?: string;
    };
    result?: unknown;
    service: string;
    startedAt?: string;
    state:
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed";
    tries: number;
    trigger?: {
      id?: string;
      kind:
        | "schedule"
        | "operation"
        | "rpc"
        | "event"
        | "manualReplay"
        | "serviceCode"
        | "parentJob";
      operationId?: string;
      parentJobId?: string;
      requestId?: string;
      subject?: string;
      traceId?: string;
    };
    type: string;
    updatedAt: string;
  };
};

export type JobsGetKeyInput = { key: string; service: string; type: string };
export type JobsGetKeyOutput = {
  active: Array<
    {
      heartbeatAgeMs: number;
      heartbeatAt: string;
      instanceId: string;
      jobId: string;
      leaseExpiresAt: string;
      startedAt: string;
    }
  >;
  key: string;
  keyHash: string;
  latestPolicyReason?: string;
  queued: Array<{ createdAt: string; jobId: string }>;
  queuedDepth: number;
  service: string;
  staleTakeoverCount: number;
  type: string;
};

export type JobsInspectInput = { id: string };
export type JobsInspectOutput = {
  attempts: Array<
    {
      endedAt?: string;
      error?: {
        causes?: Array<{}>;
        fingerprint: string;
        firstSeen?: string;
        message: string;
        occurrenceCount?: number;
        stack?: string;
        type?: string;
        worker?: {
          instanceId?: string;
          runtime?: string;
          service?: string;
          version?: string;
        };
      };
      startedAt: string;
      state?:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      try: number;
    }
  >;
  errors: Array<
    {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    }
  >;
  job: {
    completedAt?: string;
    concurrency?: {
      heartbeatAt?: string;
      key: string;
      keyHash: string;
      leaseExpiresAt?: string;
      staleTakeoverCount?: number;
    };
    context: {
      requestId: string;
      traceId: string;
      traceparent: string;
      tracestate?: string;
    };
    createdAt: string;
    deadline?: string;
    errorDetail?: {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    };
    id: string;
    lastError?: string;
    lineage?: {
      operationId?: string;
      parentJobId?: string;
      relatedKeys?: Array<string>;
      rootJobId?: string;
    };
    logs?: Array<
      { level: "info" | "warn" | "error"; message: string; timestamp: string }
    >;
    maxTries: number;
    payload: unknown;
    progress?: {
      current?: number;
      message?: string;
      step?: string;
      total?: number;
    };
    queuePolicy?: {
      existingJobId?: string;
      outcome: string;
      reason?: string;
      replacedJobId?: string;
    };
    result?: unknown;
    service: string;
    startedAt?: string;
    state:
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed";
    tries: number;
    trigger?: {
      id?: string;
      kind:
        | "schedule"
        | "operation"
        | "rpc"
        | "event"
        | "manualReplay"
        | "serviceCode"
        | "parentJob";
      operationId?: string;
      parentJobId?: string;
      requestId?: string;
      subject?: string;
      traceId?: string;
    };
    type: string;
    updatedAt: string;
  };
  lineage?: {
    operationId?: string;
    parentJobId?: string;
    relatedKeys?: Array<string>;
    rootJobId?: string;
  };
  related: Array<
    {
      completedAt?: string;
      context?: {
        requestId: string;
        traceId: string;
        traceparent: string;
        tracestate?: string;
      };
      createdAt: string;
      errorFingerprint?: string;
      id: string;
      lastError?: string;
      lineage?: {
        operationId?: string;
        parentJobId?: string;
        relatedKeys?: Array<string>;
        rootJobId?: string;
      };
      matchedBy?: "trace" | "parent" | "root" | "operation" | "concurrency";
      maxTries: number;
      progress?: {
        current?: number;
        message?: string;
        step?: string;
        total?: number;
      };
      queueAgeMs?: number;
      queueKey?: string;
      runtimeBand?: string;
      runtimeMs?: number;
      service: string;
      startedAt?: string;
      state:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      tries: number;
      trigger?: {
        id?: string;
        kind:
          | "schedule"
          | "operation"
          | "rpc"
          | "event"
          | "manualReplay"
          | "serviceCode"
          | "parentJob";
        operationId?: string;
        parentJobId?: string;
        requestId?: string;
        subject?: string;
        traceId?: string;
      };
      type: string;
      updatedAt: string;
    }
  >;
  timeline: Array<
    {
      error?: string;
      errorDetail?: {
        causes?: Array<{}>;
        fingerprint: string;
        firstSeen?: string;
        message: string;
        occurrenceCount?: number;
        stack?: string;
        type?: string;
        worker?: {
          instanceId?: string;
          runtime?: string;
          service?: string;
          version?: string;
        };
      };
      logs?: Array<
        { level: "info" | "warn" | "error"; message: string; timestamp: string }
      >;
      message?: string;
      previousState?:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      progress?: {
        current?: number;
        message?: string;
        step?: string;
        total?: number;
      };
      projected?: boolean;
      rawEvent?: unknown;
      reason?: string;
      sequence: number;
      state:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      timestamp: string;
      tries?: number;
      type: string;
      workerInstanceId?: string;
    }
  >;
  trigger?: {
    id?: string;
    kind:
      | "schedule"
      | "operation"
      | "rpc"
      | "event"
      | "manualReplay"
      | "serviceCode"
      | "parentJob";
    operationId?: string;
    parentJobId?: string;
    requestId?: string;
    subject?: string;
    traceId?: string;
  };
};

export type JobsListDLQInput = {
  limit: number;
  offset?: number;
  service?: string;
  since?: string;
  type?: string;
};
export type JobsListDLQOutput = {
  count: number;
  entries: Array<
    {
      completedAt?: string;
      concurrency?: {
        heartbeatAt?: string;
        key: string;
        keyHash: string;
        leaseExpiresAt?: string;
        staleTakeoverCount?: number;
      };
      context: {
        requestId: string;
        traceId: string;
        traceparent: string;
        tracestate?: string;
      };
      createdAt: string;
      deadline?: string;
      errorDetail?: {
        causes?: Array<{}>;
        fingerprint: string;
        firstSeen?: string;
        message: string;
        occurrenceCount?: number;
        stack?: string;
        type?: string;
        worker?: {
          instanceId?: string;
          runtime?: string;
          service?: string;
          version?: string;
        };
      };
      id: string;
      lastError?: string;
      lineage?: {
        operationId?: string;
        parentJobId?: string;
        relatedKeys?: Array<string>;
        rootJobId?: string;
      };
      logs?: Array<
        { level: "info" | "warn" | "error"; message: string; timestamp: string }
      >;
      maxTries: number;
      payload: unknown;
      progress?: {
        current?: number;
        message?: string;
        step?: string;
        total?: number;
      };
      queuePolicy?: {
        existingJobId?: string;
        outcome: string;
        reason?: string;
        replacedJobId?: string;
      };
      result?: unknown;
      service: string;
      startedAt?: string;
      state:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      tries: number;
      trigger?: {
        id?: string;
        kind:
          | "schedule"
          | "operation"
          | "rpc"
          | "event"
          | "manualReplay"
          | "serviceCode"
          | "parentJob";
        operationId?: string;
        parentJobId?: string;
        requestId?: string;
        subject?: string;
        traceId?: string;
      };
      type: string;
      updatedAt: string;
    }
  >;
  limit: number;
  nextOffset?: number;
  offset: number;
};

export type JobsListServicesInput = { limit: number; offset?: number };
export type JobsListServicesOutput = {
  count: number;
  entries: Array<
    {
      healthy: boolean;
      name: string;
      workers: Array<
        {
          concurrency?: number;
          instanceId: string;
          jobType: string;
          service: string;
          timestamp: string;
          version?: string;
        }
      >;
    }
  >;
  limit: number;
  nextOffset?: number;
  offset: number;
};

export type JobsMetricsInput = {
  groupBy: "type" | "service" | "queueKey" | "state" | "trigger";
  queueKey?: string;
  service?: string;
  state?: Array<
    (
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed"
    )
  >;
  step: "1m" | "5m" | "15m" | "1h" | "6h" | "1d";
  trigger?: string;
  type?: string;
  window: "15m" | "1h" | "6h" | "24h" | "7d";
};
export type JobsMetricsOutput = {
  buckets: Array<
    {
      end: string;
      groups: Array<
        {
          cancelled: number;
          completed: number;
          dead: number;
          dismissed: number;
          failed: number;
          key: string;
          label: string;
          queueWait: {
            count: number;
            maxMs?: number;
            p50Ms?: number;
            p95Ms?: number;
          };
          retried: number;
          runtime: {
            count: number;
            maxMs?: number;
            p50Ms?: number;
            p95Ms?: number;
          };
          started: number;
          submitted: number;
        }
      >;
      start: string;
    }
  >;
  generatedAt: string;
  groupBy: string;
  step: string;
  summary: Array<
    {
      byState: { [k: string]: number };
      dead?: number;
      failed?: number;
      failureRate?: number;
      key: string;
      label: string;
      latestUpdatedAt?: string;
      oldestCreatedAt?: string;
      queueWait: {
        count: number;
        maxMs?: number;
        p50Ms?: number;
        p95Ms?: number;
      };
      queued?: number;
      running?: number;
      runtime: {
        count: number;
        maxMs?: number;
        p50Ms?: number;
        p95Ms?: number;
      };
      slow?: number;
      total: number;
    }
  >;
  window: string;
};

export type JobsQueryInput = {
  groupBy?:
    | "service"
    | "type"
    | "state"
    | "queueKey"
    | "trigger"
    | "runtimeBand";
  limit: number;
  offset?: number;
  queueKey?: string;
  runtimeBand?: "queued" | "running" | "slow" | "terminal";
  search?: string;
  service?: string;
  sort?: {
    direction?: "asc" | "desc";
    field:
      | "updatedAt"
      | "queueAge"
      | "runtime"
      | "failureRate"
      | "retries"
      | "depth";
  };
  state?: Array<
    (
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed"
    )
  >;
  trigger?: string;
  type?: string;
  window?: "1h" | "24h" | "7d";
};
export type JobsQueryOutput = {
  count: number;
  entries: Array<
    {
      completedAt?: string;
      context?: {
        requestId: string;
        traceId: string;
        traceparent: string;
        tracestate?: string;
      };
      createdAt: string;
      errorFingerprint?: string;
      id: string;
      lastError?: string;
      lineage?: {
        operationId?: string;
        parentJobId?: string;
        relatedKeys?: Array<string>;
        rootJobId?: string;
      };
      maxTries: number;
      progress?: {
        current?: number;
        message?: string;
        step?: string;
        total?: number;
      };
      queueAgeMs?: number;
      queueKey?: string;
      runtimeBand?: string;
      runtimeMs?: number;
      service: string;
      startedAt?: string;
      state:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
      tries: number;
      trigger?: {
        id?: string;
        kind:
          | "schedule"
          | "operation"
          | "rpc"
          | "event"
          | "manualReplay"
          | "serviceCode"
          | "parentJob";
        operationId?: string;
        parentJobId?: string;
        requestId?: string;
        subject?: string;
        traceId?: string;
      };
      type: string;
      updatedAt: string;
    }
  >;
  groups: Array<
    {
      count: number;
      depth?: number;
      failureRate?: number;
      key: string;
      label: string;
      latestUpdatedAt?: string;
      oldestCreatedAt?: string;
      state?:
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed";
    }
  >;
  limit: number;
  nextOffset?: number;
  offset: number;
  stats: {
    byState: { [k: string]: number };
    dead?: number;
    failed?: number;
    queued?: number;
    running?: number;
    slow?: number;
    total: number;
  };
};

export type JobsReplayDLQInput = { id: string; reason?: string };
export type JobsReplayDLQOutput = {
  job: {
    completedAt?: string;
    concurrency?: {
      heartbeatAt?: string;
      key: string;
      keyHash: string;
      leaseExpiresAt?: string;
      staleTakeoverCount?: number;
    };
    context: {
      requestId: string;
      traceId: string;
      traceparent: string;
      tracestate?: string;
    };
    createdAt: string;
    deadline?: string;
    errorDetail?: {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    };
    id: string;
    lastError?: string;
    lineage?: {
      operationId?: string;
      parentJobId?: string;
      relatedKeys?: Array<string>;
      rootJobId?: string;
    };
    logs?: Array<
      { level: "info" | "warn" | "error"; message: string; timestamp: string }
    >;
    maxTries: number;
    payload: unknown;
    progress?: {
      current?: number;
      message?: string;
      step?: string;
      total?: number;
    };
    queuePolicy?: {
      existingJobId?: string;
      outcome: string;
      reason?: string;
      replacedJobId?: string;
    };
    result?: unknown;
    service: string;
    startedAt?: string;
    state:
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed";
    tries: number;
    trigger?: {
      id?: string;
      kind:
        | "schedule"
        | "operation"
        | "rpc"
        | "event"
        | "manualReplay"
        | "serviceCode"
        | "parentJob";
      operationId?: string;
      parentJobId?: string;
      requestId?: string;
      subject?: string;
      traceId?: string;
    };
    type: string;
    updatedAt: string;
  };
};

export type JobsRetryInput = { id: string; reason?: string };
export type JobsRetryOutput = {
  job: {
    completedAt?: string;
    concurrency?: {
      heartbeatAt?: string;
      key: string;
      keyHash: string;
      leaseExpiresAt?: string;
      staleTakeoverCount?: number;
    };
    context: {
      requestId: string;
      traceId: string;
      traceparent: string;
      tracestate?: string;
    };
    createdAt: string;
    deadline?: string;
    errorDetail?: {
      causes?: Array<{}>;
      fingerprint: string;
      firstSeen?: string;
      message: string;
      occurrenceCount?: number;
      stack?: string;
      type?: string;
      worker?: {
        instanceId?: string;
        runtime?: string;
        service?: string;
        version?: string;
      };
    };
    id: string;
    lastError?: string;
    lineage?: {
      operationId?: string;
      parentJobId?: string;
      relatedKeys?: Array<string>;
      rootJobId?: string;
    };
    logs?: Array<
      { level: "info" | "warn" | "error"; message: string; timestamp: string }
    >;
    maxTries: number;
    payload: unknown;
    progress?: {
      current?: number;
      message?: string;
      step?: string;
      total?: number;
    };
    queuePolicy?: {
      existingJobId?: string;
      outcome: string;
      reason?: string;
      replacedJobId?: string;
    };
    result?: unknown;
    service: string;
    startedAt?: string;
    state:
      | "pending"
      | "active"
      | "retry"
      | "completed"
      | "failed"
      | "cancelled"
      | "skipped"
      | "stale"
      | "expired"
      | "dead"
      | "dismissed";
    tries: number;
    trigger?: {
      id?: string;
      kind:
        | "schedule"
        | "operation"
        | "rpc"
        | "event"
        | "manualReplay"
        | "serviceCode"
        | "parentJob";
      operationId?: string;
      parentJobId?: string;
      requestId?: string;
      subject?: string;
      traceId?: string;
    };
    type: string;
    updatedAt: string;
  };
};

export type JobsWatchInput = {
  includeInitial?: boolean;
  jobId?: string;
  query?: {
    groupBy?:
      | "service"
      | "type"
      | "state"
      | "queueKey"
      | "trigger"
      | "runtimeBand";
    limit: number;
    offset?: number;
    queueKey?: string;
    runtimeBand?: "queued" | "running" | "slow" | "terminal";
    search?: string;
    service?: string;
    sort?: {
      direction?: "asc" | "desc";
      field:
        | "updatedAt"
        | "queueAge"
        | "runtime"
        | "failureRate"
        | "retries"
        | "depth";
    };
    state?: Array<
      (
        | "pending"
        | "active"
        | "retry"
        | "completed"
        | "failed"
        | "cancelled"
        | "skipped"
        | "stale"
        | "expired"
        | "dead"
        | "dismissed"
      )
    >;
    trigger?: string;
    type?: string;
    window?: "1h" | "24h" | "7d";
  };
};
export type JobsWatchEvent = { kind: "ready"; timestamp: string } | {
  id: string;
  kind: "jobChanged";
  service: string;
  state:
    | "pending"
    | "active"
    | "retry"
    | "completed"
    | "failed"
    | "cancelled"
    | "skipped"
    | "stale"
    | "expired"
    | "dead"
    | "dismissed";
  type: string;
  updatedAt: string;
} | {
  kind: "queryInvalidated";
  reason: "matched-job-changed" | "unknown-match";
  timestamp: string;
} | { id: string; kind: "jobInspectChanged"; timestamp: string };
export type JobsWatchFeedHandler = (
  context: {
    input: JobsWatchInput;
    caller: SessionCaller;
    signal: AbortSignal;
    emit(
      event: JobsWatchEvent,
    ): AsyncResult<void, ValidationError | UnexpectedError>;
    client: HandlerClient;
  },
) => unknown | Promise<unknown>;

export type NotFoundErrorData = {
  context?: { [k: string]: unknown };
  id: string;
  jobId?: string;
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
  "Jobs.Cancel": { input: JobsCancelInput; output: JobsCancelOutput };
  "Jobs.DismissDLQ": {
    input: JobsDismissDLQInput;
    output: JobsDismissDLQOutput;
  };
  "Jobs.GetKey": { input: JobsGetKeyInput; output: JobsGetKeyOutput };
  "Jobs.Inspect": { input: JobsInspectInput; output: JobsInspectOutput };
  "Jobs.ListDLQ": { input: JobsListDLQInput; output: JobsListDLQOutput };
  "Jobs.ListServices": {
    input: JobsListServicesInput;
    output: JobsListServicesOutput;
  };
  "Jobs.Metrics": { input: JobsMetricsInput; output: JobsMetricsOutput };
  "Jobs.Query": { input: JobsQueryInput; output: JobsQueryOutput };
  "Jobs.ReplayDLQ": { input: JobsReplayDLQInput; output: JobsReplayDLQOutput };
  "Jobs.Retry": { input: JobsRetryInput; output: JobsRetryOutput };
}

export type JobsCancelHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsCancelHandlerResult = Result<
  JobsCancelOutput,
  JobsCancelHandlerError
>;
export type JobsCancelHandler = (
  args: {
    input: JobsCancelInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsCancelHandlerResult | Promise<JobsCancelHandlerResult>;
export type JobsDismissDLQHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsDismissDLQHandlerResult = Result<
  JobsDismissDLQOutput,
  JobsDismissDLQHandlerError
>;
export type JobsDismissDLQHandler = (
  args: {
    input: JobsDismissDLQInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsDismissDLQHandlerResult | Promise<JobsDismissDLQHandlerResult>;
export type JobsGetKeyHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsGetKeyHandlerResult = Result<
  JobsGetKeyOutput,
  JobsGetKeyHandlerError
>;
export type JobsGetKeyHandler = (
  args: {
    input: JobsGetKeyInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsGetKeyHandlerResult | Promise<JobsGetKeyHandlerResult>;
export type JobsInspectHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsInspectHandlerResult = Result<
  JobsInspectOutput,
  JobsInspectHandlerError
>;
export type JobsInspectHandler = (
  args: {
    input: JobsInspectInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsInspectHandlerResult | Promise<JobsInspectHandlerResult>;
export type JobsListDLQHandlerError = TrellisErrorInstance;
export type JobsListDLQHandlerResult = Result<
  JobsListDLQOutput,
  JobsListDLQHandlerError
>;
export type JobsListDLQHandler = (
  args: {
    input: JobsListDLQInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsListDLQHandlerResult | Promise<JobsListDLQHandlerResult>;
export type JobsListServicesHandlerError = TrellisErrorInstance;
export type JobsListServicesHandlerResult = Result<
  JobsListServicesOutput,
  JobsListServicesHandlerError
>;
export type JobsListServicesHandler = (
  args: {
    input: JobsListServicesInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsListServicesHandlerResult | Promise<JobsListServicesHandlerResult>;
export type JobsMetricsHandlerError = TrellisErrorInstance;
export type JobsMetricsHandlerResult = Result<
  JobsMetricsOutput,
  JobsMetricsHandlerError
>;
export type JobsMetricsHandler = (
  args: {
    input: JobsMetricsInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsMetricsHandlerResult | Promise<JobsMetricsHandlerResult>;
export type JobsQueryHandlerError = TrellisErrorInstance;
export type JobsQueryHandlerResult = Result<
  JobsQueryOutput,
  JobsQueryHandlerError
>;
export type JobsQueryHandler = (
  args: {
    input: JobsQueryInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsQueryHandlerResult | Promise<JobsQueryHandlerResult>;
export type JobsReplayDLQHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsReplayDLQHandlerResult = Result<
  JobsReplayDLQOutput,
  JobsReplayDLQHandlerError
>;
export type JobsReplayDLQHandler = (
  args: {
    input: JobsReplayDLQInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsReplayDLQHandlerResult | Promise<JobsReplayDLQHandlerResult>;
export type JobsRetryHandlerError =
  | TrellisErrorInstance
  | BaseError<NotFoundErrorData>;
export type JobsRetryHandlerResult = Result<
  JobsRetryOutput,
  JobsRetryHandlerError
>;
export type JobsRetryHandler = (
  args: {
    input: JobsRetryInput;
    context: RpcHandlerContext;
    client: HandlerClient;
  },
) => JobsRetryHandlerResult | Promise<JobsRetryHandlerResult>;

export interface EventMap {
}

export interface FeedMap {
  "Jobs.Watch": { input: JobsWatchInput; event: JobsWatchEvent };
}

export interface SubjectMap {
}
