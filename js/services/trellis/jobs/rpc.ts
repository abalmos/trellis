import { jetstreamManager } from "@nats-io/jetstream";
import {
  headers as natsHeaders,
  type NatsConnection,
  type Subscription,
} from "@nats-io/nats-core";
import { Result } from "@qlever-llc/trellis";
import type {
  JobsCancelInput,
  JobsCancelOutput,
  JobsHealthOutput,
  JobsInspectInput,
  JobsInspectOutput,
  JobsListServicesInput,
  JobsListServicesOutput,
  JobsQueryInput,
  JobsQueryOutput,
} from "@qlever-llc/trellis/sdk/jobs";
import { NotFoundError } from "@qlever-llc/trellis/sdk/jobs";
import { type StaticDecode, Type } from "typebox";
import { Value } from "typebox/value";

const JobStateSchema = Type.Union([
  Type.Literal("pending"),
  Type.Literal("active"),
  Type.Literal("retry"),
  Type.Literal("completed"),
  Type.Literal("failed"),
  Type.Literal("cancelled"),
  Type.Literal("expired"),
  Type.Literal("skipped"),
  Type.Literal("stale"),
  Type.Literal("dead"),
  Type.Literal("dismissed"),
]);
const JobContextSchema = Type.Object({
  requestId: Type.String({ minLength: 1 }),
  traceId: Type.String({ pattern: "^[0-9a-f]{32}$" }),
  traceparent: Type.String({
    pattern: "^[0-9a-f]{2}-[0-9a-f]{32}-[0-9a-f]{16}-[0-9a-f]{2}$",
  }),
  tracestate: Type.Optional(Type.String({ minLength: 1 })),
});
const JobProgressSchema = Type.Object({
  step: Type.Optional(Type.String()),
  message: Type.Optional(Type.String()),
  current: Type.Optional(Type.Integer({ minimum: 0 })),
  total: Type.Optional(Type.Integer({ minimum: 0 })),
});
const JobLogEntrySchema = Type.Object({
  timestamp: Type.String({ format: "date-time" }),
  level: Type.Union([
    Type.Literal("info"),
    Type.Literal("warn"),
    Type.Literal("error"),
  ]),
  message: Type.String(),
});
const JobEventSchema = Type.Object({
  jobId: Type.String({ minLength: 1 }),
  service: Type.String({ minLength: 1 }),
  jobType: Type.String({ minLength: 1 }),
  eventType: Type.Union([
    Type.Literal("created"),
    Type.Literal("started"),
    Type.Literal("retry"),
    Type.Literal("progress"),
    Type.Literal("logged"),
    Type.Literal("completed"),
    Type.Literal("failed"),
    Type.Literal("cancelled"),
    Type.Literal("expired"),
    Type.Literal("skipped"),
    Type.Literal("stale"),
    Type.Literal("heartbeat"),
    Type.Literal("staleCompletionIgnored"),
    Type.Literal("retried"),
    Type.Literal("dead"),
    Type.Literal("dismissed"),
  ]),
  state: JobStateSchema,
  previousState: Type.Optional(JobStateSchema),
  context: JobContextSchema,
  tries: Type.Integer({ minimum: 0 }),
  maxTries: Type.Optional(Type.Integer({ minimum: 1 })),
  error: Type.Optional(Type.String()),
  progress: Type.Optional(JobProgressSchema),
  logs: Type.Optional(Type.Array(JobLogEntrySchema)),
  payload: Type.Optional(Type.Unknown()),
  result: Type.Optional(Type.Unknown()),
  deadline: Type.Optional(Type.String({ format: "date-time" })),
  timestamp: Type.String({ format: "date-time" }),
});
const WorkerHeartbeatSchema = Type.Object({
  service: Type.String({ minLength: 1 }),
  jobType: Type.String({ minLength: 1 }),
  instanceId: Type.String({ minLength: 1 }),
  concurrency: Type.Optional(Type.Integer({ minimum: 1 })),
  version: Type.Optional(Type.String({ minLength: 1 })),
  timestamp: Type.String({ format: "date-time" }),
});
const MaxDeliveriesAdvisorySchema = Type.Object({
  stream: Type.String({ minLength: 1 }),
  consumer: Type.String({ minLength: 1 }),
  stream_seq: Type.Optional(Type.Integer({ minimum: 1 })),
  streamSeq: Type.Optional(Type.Integer({ minimum: 1 })),
  deliveries: Type.Optional(Type.Integer({ minimum: 1 })),
  num_deliveries: Type.Optional(Type.Integer({ minimum: 1 })),
  timestamp: Type.String({ format: "date-time" }),
});

type JobEvent = StaticDecode<typeof JobEventSchema>;
type WorkerHeartbeat = StaticDecode<typeof WorkerHeartbeatSchema>;
type RawMaxDeliveriesAdvisory = StaticDecode<
  typeof MaxDeliveriesAdvisorySchema
>;
type MaxDeliveriesAdvisory = {
  stream: string;
  consumer: string;
  streamSeq: number;
  deliveries: number;
  timestamp: string;
};

type AdminJob = JobsCancelOutput["job"];
type AdminJobState = AdminJob["state"];
type AdminWorker = JobsListServicesOutput["entries"][number]["workers"][number];
type WorkbenchRow = JobsQueryOutput["entries"][number];

const textDecoder = new TextDecoder();
const textEncoder = new TextEncoder();
const workerFreshnessMs = 60_000;

type JobsAdminProjection = {
  readonly jobs: Map<string, AdminJob>;
  readonly jobKeysById: Map<string, string>;
  readonly cancelSubjects: Map<string, string>;
  readonly workers: Map<string, AdminWorker>;
  readonly stop: () => void;
};

type DirectMessageReader = {
  getMessage(
    stream: string,
    query: { seq: number },
  ): Promise<{ data: Uint8Array } | null>;
};

type JobsCancelError = NotFoundError;
type JobsInspectError = NotFoundError;

function jobKey(service: string, type: string, id: string): string {
  return `${service}\u001f${type}\u001f${id}`;
}

function workerKey(worker: WorkerHeartbeat): string {
  return `${worker.service}\u001f${worker.jobType}\u001f${worker.instanceId}`;
}

function cancelSubjectFromLifecycleSubject(
  subject: string,
): string | undefined {
  const lastDot = subject.lastIndexOf(".");
  if (lastDot < 0) return undefined;
  return `${subject.slice(0, lastDot)}.cancelled`;
}

function isJobEvent(value: unknown): value is JobEvent {
  return Value.Check(JobEventSchema, value);
}

function isWorkerHeartbeat(value: unknown): value is WorkerHeartbeat {
  return Value.Check(WorkerHeartbeatSchema, value);
}

/** Parses a NATS max-deliveries advisory into the Jobs admin projection shape. */
export function parseMaxDeliveriesAdvisory(
  value: unknown,
): MaxDeliveriesAdvisory | undefined {
  if (!Value.Check(MaxDeliveriesAdvisorySchema, value)) return undefined;
  const raw = value as RawMaxDeliveriesAdvisory;
  const streamSeq = raw.stream_seq ?? raw.streamSeq;
  const deliveries = raw.deliveries ?? raw.num_deliveries;
  if (streamSeq === undefined || deliveries === undefined) return undefined;
  return {
    stream: raw.stream,
    consumer: raw.consumer,
    streamSeq,
    deliveries,
    timestamp: raw.timestamp,
  };
}

function parseJsonPayload(data: Uint8Array): unknown {
  try {
    return JSON.parse(textDecoder.decode(data));
  } catch {
    return undefined;
  }
}

function toAdminJobState(state: JobEvent["state"]): AdminJobState | undefined {
  switch (state) {
    case "pending":
    case "active":
    case "retry":
    case "completed":
    case "failed":
    case "cancelled":
    case "expired":
    case "skipped":
    case "stale":
    case "dead":
    case "dismissed":
      return state;
  }
}

function isTerminalState(state: AdminJobState): boolean {
  return state === "completed" || state === "failed" ||
    state === "cancelled" || state === "expired" || state === "dead" ||
    state === "dismissed";
}

function applyJobEvent(
  jobs: Map<string, AdminJob>,
  jobKeysById: Map<string, string>,
  event: JobEvent,
): AdminJob | undefined {
  const state = toAdminJobState(event.state);
  if (!state) return undefined;

  const key = jobKey(event.service, event.jobType, event.jobId);
  const current = jobs.get(key);
  if (current && isTerminalState(current.state)) return current;

  const base: AdminJob = current ?? {
    id: event.jobId,
    service: event.service,
    type: event.jobType,
    state,
    context: event.context,
    payload: event.payload,
    createdAt: event.timestamp,
    updatedAt: event.timestamp,
    tries: event.tries,
    maxTries: event.maxTries ?? 1,
  };
  const nextBase: AdminJob = {
    ...base,
    state,
    updatedAt: event.timestamp,
    tries: event.tries,
    ...(event.maxTries !== undefined ? { maxTries: event.maxTries } : {}),
    ...(event.deadline !== undefined ? { deadline: event.deadline } : {}),
  };

  let next: AdminJob;
  switch (event.eventType) {
    case "created":
    case "retried":
      next = event.payload === undefined ? nextBase : {
        ...nextBase,
        payload: event.payload,
      };
      break;
    case "started":
      next = { ...nextBase, startedAt: event.timestamp };
      break;
    case "progress":
      next = event.progress === undefined ? nextBase : {
        ...nextBase,
        progress: event.progress,
      };
      break;
    case "logged":
      next = event.logs === undefined ? nextBase : {
        ...nextBase,
        logs: [...(base.logs ?? []), ...event.logs],
      };
      break;
    case "completed":
      next = {
        ...nextBase,
        completedAt: event.timestamp,
        ...(event.result !== undefined ? { result: event.result } : {}),
      };
      break;
    case "failed":
    case "cancelled":
    case "expired":
    case "staleCompletionIgnored":
    case "dead":
    case "dismissed":
      next = event.error === undefined ? nextBase : {
        ...nextBase,
        lastError: event.error,
      };
      break;
    case "retry":
    case "heartbeat":
    case "skipped":
    case "stale":
      next = nextBase;
      break;
  }

  jobs.set(key, next);
  jobKeysById.set(next.id, key);
  return next;
}

function findJobById(
  jobs: Map<string, AdminJob>,
  jobKeysById: Map<string, string>,
  id: string,
): AdminJob | undefined {
  const key = jobKeysById.get(id);
  return key === undefined ? undefined : jobs.get(key);
}

function notFound(jobId: string): NotFoundError {
  return new NotFoundError({
    id: crypto.randomUUID(),
    type: "NotFoundError",
    resource: "job",
    jobId,
    message: `Job '${jobId}' was not found`,
  });
}

function headersFromJobContext(context: JobEvent["context"]) {
  const headers = natsHeaders();
  headers.set("request-id", context.requestId);
  headers.set("traceparent", context.traceparent);
  if (context.tracestate) {
    headers.set("tracestate", context.tracestate);
  }
  return headers;
}

/** Maps an exhausted-delivery advisory to a Jobs lifecycle dead event. */
export function mapDeadEventFromAdvisory(
  current: AdminJob | undefined,
  workEvent: JobEvent,
  advisory: MaxDeliveriesAdvisory,
): JobEvent | undefined {
  if (current && isTerminalState(current.state)) return undefined;
  return {
    jobId: workEvent.jobId,
    service: workEvent.service,
    jobType: workEvent.jobType,
    eventType: "dead",
    state: "dead",
    previousState: current?.state ?? workEvent.state,
    context: workEvent.context,
    tries: Math.max(current?.tries ?? workEvent.tries, advisory.deliveries),
    maxTries: current?.maxTries ?? workEvent.maxTries,
    error:
      `max deliveries exceeded: stream=${advisory.stream} consumer=${advisory.consumer} deliveries=${advisory.deliveries}`,
    timestamp: advisory.timestamp,
  };
}

function jobRuntimeMs(job: AdminJob): number | undefined {
  if (!job.startedAt) return undefined;
  const start = Date.parse(job.startedAt);
  const end = job.completedAt ? Date.parse(job.completedAt) : Date.now();
  if (Number.isNaN(start) || Number.isNaN(end) || end < start) return undefined;
  return end - start;
}

function runtimeBand(job: AdminJob): string {
  if (job.state === "pending" || job.state === "retry") return "queued";
  if (job.state === "active") {
    return (jobRuntimeMs(job) ?? 0) >= 300_000 ? "slow" : "running";
  }
  return "terminal";
}

function toWorkbenchRow(job: AdminJob): WorkbenchRow {
  const runtimeMs = jobRuntimeMs(job);
  return {
    completedAt: job.completedAt,
    context: job.context,
    createdAt: job.createdAt,
    errorFingerprint: job.errorDetail?.fingerprint,
    id: job.id,
    lastError: job.lastError,
    lineage: job.lineage,
    maxTries: job.maxTries,
    progress: job.progress,
    queueAgeMs: Date.now() - Date.parse(job.createdAt),
    queueKey: job.concurrency?.key,
    runtimeBand: runtimeBand(job),
    runtimeMs,
    service: job.service,
    startedAt: job.startedAt,
    state: job.state,
    tries: job.tries,
    trigger: job.trigger,
    type: job.type,
    updatedAt: job.updatedAt,
  };
}

function compareJobs(
  left: AdminJob,
  right: AdminJob,
  sort: JobsQueryInput["sort"],
): number {
  const direction = sort?.direction ?? "desc";
  const field = sort?.field ?? "updatedAt";
  const multiplier = direction === "asc" ? 1 : -1;
  const value = field === "queueAge"
    ? Date.parse(left.createdAt) - Date.parse(right.createdAt)
    : field === "runtime"
    ? (jobRuntimeMs(left) ?? 0) - (jobRuntimeMs(right) ?? 0)
    : field === "retries"
    ? left.tries - right.tries
    : Date.parse(left.updatedAt) - Date.parse(right.updatedAt);
  return value === 0 ? left.id.localeCompare(right.id) : multiplier * value;
}

function groupJobs(
  jobs: AdminJob[],
  groupBy: NonNullable<JobsQueryInput["groupBy"]>,
): JobsQueryOutput["groups"] {
  const groups = new Map<string, AdminJob[]>();
  for (const job of jobs) {
    const key = groupBy === "type"
      ? job.type
      : groupBy === "state"
      ? job.state
      : groupBy === "queueKey"
      ? job.concurrency?.key ?? "unkeyed"
      : groupBy === "trigger"
      ? job.trigger?.kind ?? "unknown"
      : groupBy === "runtimeBand"
      ? runtimeBand(job)
      : job.service;
    groups.set(key, [...groups.get(key) ?? [], job]);
  }
  return [...groups.entries()].map(([key, entries]) => {
    const failures = entries.filter((job) =>
      job.state === "failed" || job.state === "dead"
    ).length;
    return {
      count: entries.length,
      depth: entries.length,
      failureRate: entries.length === 0 ? 0 : failures / entries.length,
      key,
      label: key,
      latestUpdatedAt: entries.map((job) => job.updatedAt).sort().at(-1),
      oldestCreatedAt: entries.map((job) => job.createdAt).sort()[0],
      ...(groupBy === "state" ? { state: key as AdminJobState } : {}),
    };
  });
}

function queryJobs(
  jobs: Map<string, AdminJob>,
  input: JobsQueryInput,
): Result<JobsQueryOutput, never> {
  const states = input.state ? new Set<AdminJobState>(input.state) : undefined;
  const search = input.search?.trim().toLowerCase();
  const filtered = [...jobs.values()]
    .filter((job) =>
      input.service === undefined || job.service === input.service
    )
    .filter((job) => input.type === undefined || job.type === input.type)
    .filter((job) => states === undefined || states.has(job.state))
    .filter((job) =>
      input.runtimeBand === undefined || runtimeBand(job) === input.runtimeBand
    )
    .filter((job) =>
      input.queueKey === undefined || job.concurrency?.key === input.queueKey
    )
    .filter((job) =>
      input.trigger === undefined || job.trigger?.kind === input.trigger
    )
    .filter((job) =>
      search === undefined || [
        job.id,
        job.service,
        job.type,
        job.state,
        job.context.requestId,
        job.context.traceId,
        job.context.traceparent,
        job.concurrency?.key,
        job.lastError,
        job.errorDetail?.message,
      ].filter(Boolean).join(" ").toLowerCase().includes(search)
    )
    .sort((left, right) => compareJobs(left, right, input.sort));
  const offset = input.offset ?? 0;
  const entries = filtered.slice(offset, offset + input.limit).map(
    toWorkbenchRow,
  );
  const byState: Record<string, number> = {};
  for (const job of filtered) {
    byState[job.state] = (byState[job.state] ?? 0) + 1;
  }
  return Result.ok({
    count: filtered.length,
    entries,
    groups: groupJobs(filtered, input.groupBy ?? "service"),
    limit: input.limit,
    nextOffset: offset + entries.length < filtered.length
      ? offset + entries.length
      : undefined,
    offset,
    stats: {
      byState,
      dead: byState.dead ?? 0,
      failed: byState.failed ?? 0,
      queued: (byState.pending ?? 0) + (byState.retry ?? 0),
      running: byState.active ?? 0,
      slow: filtered.filter((job) => runtimeBand(job) === "slow").length,
      total: filtered.length,
    },
  });
}

function inspect(
  jobs: Map<string, AdminJob>,
  jobKeysById: Map<string, string>,
  id: string,
): Result<JobsInspectOutput, JobsInspectError> {
  const job = findJobById(jobs, jobKeysById, id);
  if (!job) return Result.err(notFound(id));
  const timeline: JobsInspectOutput["timeline"] = [{
    sequence: 1,
    state: job.state,
    timestamp: job.createdAt,
    tries: job.tries,
    type: "created",
  }];
  if (job.startedAt) {
    timeline.push({
      sequence: timeline.length + 1,
      state: job.state,
      timestamp: job.startedAt,
      tries: job.tries,
      type: "started",
    });
  }
  if (job.lastError || job.errorDetail) {
    timeline.push({
      error: job.lastError,
      errorDetail: job.errorDetail,
      sequence: timeline.length + 1,
      state: job.state,
      timestamp: job.updatedAt,
      tries: job.tries,
      type: "error",
    });
  }
  const related = [...jobs.values()]
    .filter((candidate) => candidate.id !== job.id)
    .filter((candidate) =>
      candidate.context.traceId === job.context.traceId ||
      (job.concurrency?.key !== undefined &&
        candidate.concurrency?.key === job.concurrency.key) ||
      (candidate.service === job.service && candidate.type === job.type)
    )
    .slice(0, 10)
    .map(toWorkbenchRow);
  return Result.ok({
    attempts: job.startedAt
      ? [{
        endedAt: job.completedAt,
        error: job.errorDetail,
        startedAt: job.startedAt,
        state: job.state,
        try: job.tries,
      }]
      : [],
    errors: job.errorDetail ? [job.errorDetail] : [],
    job,
    lineage: job.lineage,
    related,
    timeline,
    trigger: job.trigger,
  });
}

function freshWorkers(workers: Map<string, AdminWorker>): AdminWorker[] {
  const now = Date.now();
  return [...workers.values()].filter((worker) => {
    const timestamp = Date.parse(worker.timestamp);
    return !Number.isNaN(timestamp) && now - timestamp <= workerFreshnessMs;
  });
}

function listServices(
  workers: Map<string, AdminWorker>,
  input: JobsListServicesInput,
): JobsListServicesOutput {
  const byService = new Map<string, AdminWorker[]>();
  for (const worker of freshWorkers(workers)) {
    const serviceWorkers = byService.get(worker.service) ?? [];
    serviceWorkers.push(worker);
    byService.set(worker.service, serviceWorkers);
  }
  const offset = input.offset ?? 0;
  const allEntries = [...byService.entries()]
    .map(([name, serviceWorkers]) => ({
      name,
      workers: serviceWorkers.sort((left, right) =>
        left.jobType.localeCompare(right.jobType) ||
        left.instanceId.localeCompare(right.instanceId)
      ),
      healthy: serviceWorkers.length > 0,
    }))
    .sort((left, right) => left.name.localeCompare(right.name));
  const entries = allEntries.slice(offset, offset + input.limit);
  return {
    entries,
    count: allEntries.length,
    offset,
    limit: input.limit,
    ...(offset + entries.length < allEntries.length
      ? { nextOffset: offset + entries.length }
      : {}),
  };
}

function createProjection(nats: NatsConnection): JobsAdminProjection {
  const jobs = new Map<string, AdminJob>();
  const jobKeysById = new Map<string, string>();
  const cancelSubjects = new Map<string, string>();
  const workers = new Map<string, AdminWorker>();
  const subscription: Subscription = nats.subscribe("trellis.jobs.>");
  const advisorySubscription: Subscription = nats.subscribe(
    "$JS.EVENT.ADVISORY.CONSUMER.MAX_DELIVERIES.JOBS_WORK.>",
  );
  const direct = jetstreamManager(nats)
    .then((manager) => manager.direct as DirectMessageReader | undefined)
    .catch(() => undefined);
  void (async () => {
    for await (const msg of subscription) {
      const payload = parseJsonPayload(msg.data);
      if (msg.subject.startsWith("trellis.jobs.workers.")) {
        if (isWorkerHeartbeat(payload)) {
          workers.set(workerKey(payload), payload);
        }
        continue;
      }
      if (isJobEvent(payload)) {
        const cancelSubject = cancelSubjectFromLifecycleSubject(msg.subject);
        if (cancelSubject) {
          cancelSubjects.set(
            jobKey(payload.service, payload.jobType, payload.jobId),
            cancelSubject,
          );
        }
        applyJobEvent(jobs, jobKeysById, payload);
      }
    }
  })();
  void (async () => {
    for await (const msg of advisorySubscription) {
      const advisory = parseMaxDeliveriesAdvisory(parseJsonPayload(msg.data));
      if (!advisory) continue;
      const reader = await direct;
      if (!reader) continue;
      let workMessage: { data: Uint8Array } | null;
      try {
        workMessage = await reader.getMessage(advisory.stream, {
          seq: advisory.streamSeq,
        });
      } catch {
        continue;
      }
      if (!workMessage) continue;
      const workEvent = parseJsonPayload(workMessage.data);
      if (!isJobEvent(workEvent)) continue;
      const key = jobKey(workEvent.service, workEvent.jobType, workEvent.jobId);
      const event = mapDeadEventFromAdvisory(
        jobs.get(key),
        workEvent,
        advisory,
      );
      if (!event) continue;
      nats.publish(
        `trellis.jobs.${event.service}.${event.jobType}.${event.jobId}.dead`,
        textEncoder.encode(JSON.stringify(event)),
        { headers: headersFromJobContext(event.context) },
      );
      await nats.flush();
      applyJobEvent(jobs, jobKeysById, event);
    }
  })();
  return {
    jobs,
    jobKeysById,
    cancelSubjects,
    workers,
    stop: () => {
      subscription.unsubscribe();
      advisorySubscription.unsubscribe();
    },
  };
}

/** Creates generated Jobs admin RPC handlers backed by the live JS projection. */
export function createJobsAdminHandlers(nats: NatsConnection) {
  const projection = createProjection(nats);
  return {
    stop: projection.stop,
    health: () =>
      Result.ok<JobsHealthOutput, never>({
        status: "healthy",
        service: "trellis.jobs",
        timestamp: new Date().toISOString(),
        checks: [],
      }),
    query: ({ input }: { input: JobsQueryInput }) =>
      queryJobs(projection.jobs, input),
    inspect: ({ input }: { input: JobsInspectInput }) =>
      inspect(projection.jobs, projection.jobKeysById, input.id),
    cancel: async ({ input }: { input: JobsCancelInput }) => {
      const job = findJobById(
        projection.jobs,
        projection.jobKeysById,
        input.id,
      );
      if (!job) return Result.err(notFound(input.id));
      if (isTerminalState(job.state)) {
        return Result.ok<JobsCancelOutput, JobsCancelError>({ job });
      }
      const event: JobEvent = {
        jobId: job.id,
        service: job.service,
        jobType: job.type,
        eventType: "cancelled",
        state: "cancelled",
        previousState: job.state,
        context: job.context,
        tries: job.tries,
        maxTries: job.maxTries,
        error: "cancelled",
        timestamp: new Date().toISOString(),
      };
      const cancelSubject = projection.cancelSubjects.get(
        jobKey(job.service, job.type, job.id),
      ) ?? `trellis.jobs.${job.service}.${job.type}.${job.id}.cancelled`;
      nats.publish(
        cancelSubject,
        textEncoder.encode(JSON.stringify(event)),
        { headers: headersFromJobContext(event.context) },
      );
      await nats.flush();
      return Result.ok<JobsCancelOutput, JobsCancelError>({
        job: applyJobEvent(projection.jobs, projection.jobKeysById, event) ??
          job,
      });
    },
    listServices: ({ input }: { input: JobsListServicesInput }) =>
      Result.ok<JobsListServicesOutput, never>(
        listServices(projection.workers, input),
      ),
  };
}
