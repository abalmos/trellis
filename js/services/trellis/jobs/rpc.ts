import { jetstreamManager } from "@nats-io/jetstream";
import {
  headers as natsHeaders,
  type NatsConnection,
  type Subscription,
} from "@nats-io/nats-core";
import { Result, ValidationError } from "@qlever-llc/trellis";
import type {
  JobsCancelInput,
  JobsCancelOutput,
  JobsGetInput,
  JobsGetOutput,
  JobsHealthOutput,
  JobsListInput,
  JobsListOutput,
  JobsListServicesInput,
  JobsListServicesOutput,
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

type AdminJob = JobsListOutput["entries"][number];
type AdminJobState = AdminJob["state"];
type AdminWorker = JobsListServicesOutput["entries"][number]["workers"][number];

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

type JobsGetError = NotFoundError;
type JobsCancelError = NotFoundError;

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

function parseMaxDeliveriesAdvisory(
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
    case "dead":
    case "dismissed":
      return state;
    case "skipped":
    case "stale":
      return undefined;
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

function mapDeadEventFromAdvisory(
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

function validateSince(
  since: string | undefined,
): Result<void, ValidationError> {
  if (since === undefined || !Number.isNaN(Date.parse(since))) {
    return Result.ok(undefined);
  }
  return Result.err(
    new ValidationError({
      errors: [{ path: "/since", message: "since must be a date-time string" }],
    }),
  );
}

function listJobs(
  jobs: Map<string, AdminJob>,
  input: JobsListInput,
): Result<JobsListOutput, ValidationError> {
  const since = validateSince(input.since);
  if (since.isErr()) return since;
  const sinceTimestamp = input.since === undefined
    ? undefined
    : Date.parse(input.since);
  const states = input.state ? new Set<AdminJobState>(input.state) : undefined;
  const offset = input.offset ?? 0;
  const filtered = [...jobs.values()]
    .filter((job) =>
      input.service === undefined || job.service === input.service
    )
    .filter((job) => input.type === undefined || job.type === input.type)
    .filter((job) => states === undefined || states.has(job.state))
    .filter((job) =>
      sinceTimestamp === undefined ||
      Date.parse(job.updatedAt) >= sinceTimestamp
    )
    .sort((left, right) =>
      Date.parse(right.updatedAt) - Date.parse(left.updatedAt)
    );
  const entries = filtered.slice(offset, offset + input.limit);
  return Result.ok({
    entries,
    count: filtered.length,
    offset,
    limit: input.limit,
    ...(offset + entries.length < filtered.length
      ? { nextOffset: offset + entries.length }
      : {}),
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
    list: ({ input }: { input: JobsListInput }) =>
      listJobs(projection.jobs, input),
    get: ({ input }: { input: JobsGetInput }) => {
      const job = findJobById(
        projection.jobs,
        projection.jobKeysById,
        input.id,
      );
      return job
        ? Result.ok<JobsGetOutput, JobsGetError>({ job })
        : Result.err(notFound(input.id));
    },
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
