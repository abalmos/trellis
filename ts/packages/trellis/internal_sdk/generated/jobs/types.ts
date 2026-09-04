// Generated from ./rust/crates/jobs-runtime/.trellis/artifacts/apis/trellis.jobs@v1.json
import type { SerializableErrorData } from "@qlever-llc/trellis";
import { TrellisError } from "@qlever-llc/trellis";
import { NotFoundErrorDataSchema } from "./schemas.ts";

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
    waitingOn?: Array<
      {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      }
    >;
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
    waitingOn?: Array<
      {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      }
    >;
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
    waitingOn?: Array<
      {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      }
    >;
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
      matchedBy?:
        | "trace"
        | "parent"
        | "root"
        | "operation"
        | "concurrency"
        | "wait";
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
      waitingOn?: Array<
        {
          id: string;
          label?: string;
          startedAt: string;
          target: {
            id?: string;
            key?: string;
            kind: "job" | "operation" | "external";
            label?: string;
            operation?: string;
            operationId?: string;
            service?: string;
            system?: string;
            type?: string;
          };
        }
      >;
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
      waitEdge?: {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      };
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
      waitingOn?: Array<
        {
          id: string;
          label?: string;
          startedAt: string;
          target: {
            id?: string;
            key?: string;
            kind: "job" | "operation" | "external";
            label?: string;
            operation?: string;
            operationId?: string;
            service?: string;
            system?: string;
            type?: string;
          };
        }
      >;
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
      waitingOn?: Array<
        {
          id: string;
          label?: string;
          startedAt: string;
          target: {
            id?: string;
            key?: string;
            kind: "job" | "operation" | "external";
            label?: string;
            operation?: string;
            operationId?: string;
            service?: string;
            system?: string;
            type?: string;
          };
        }
      >;
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
    waitingOn?: Array<
      {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      }
    >;
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
    waitingOn?: Array<
      {
        id: string;
        label?: string;
        startedAt: string;
        target: {
          id?: string;
          key?: string;
          kind: "job" | "operation" | "external";
          label?: string;
          operation?: string;
          operationId?: string;
          service?: string;
          system?: string;
          type?: string;
        };
      }
    >;
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

export type NotFoundErrorData =
  & SerializableErrorData
  & ({
    context?: {};
    id: string;
    jobId?: string;
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
