import { assertJobCompleted } from "@qlever-llc/trellis-test";
import {
  defineAppContract,
  defineServiceContract,
  jobs,
  Result,
} from "@qlever-llc/trellis";
import { RetryJobError } from "@qlever-llc/trellis/jobs";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import {
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
  integrationSlug,
} from "../_support/names.ts";

export type JobsWorkflowOutput = {
  readonly documentId: string;
  readonly jobId: string;
  readonly processedBy: string;
  readonly requestId: string;
  readonly traceId: string;
};

export type KeyedJobsWorkflowOutput = JobsWorkflowOutput & {
  readonly groupKey: string;
  readonly sequence: number;
};

export type TerminalJobEdgesOutput = {
  readonly documentId: string;
  readonly jobId: string;
  readonly waitState: string;
  readonly getState: string;
  readonly cancelState: string;
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export function requireJobsWorkflowOutput(value: unknown): JobsWorkflowOutput {
  if (!isRecord(value)) {
    throw new Error("expected jobs workflow output");
  }
  if (
    typeof value.documentId !== "string" || typeof value.jobId !== "string" ||
    typeof value.processedBy !== "string" ||
    typeof value.requestId !== "string" || typeof value.traceId !== "string"
  ) {
    throw new Error("expected jobs workflow output fields");
  }
  return {
    documentId: value.documentId,
    jobId: value.jobId,
    processedBy: value.processedBy,
    requestId: value.requestId,
    traceId: value.traceId,
  };
}

export function requireKeyedJobsWorkflowOutput(
  value: unknown,
): KeyedJobsWorkflowOutput {
  const output = requireJobsWorkflowOutput(value);
  if (!isRecord(value)) {
    throw new Error("expected keyed jobs workflow output");
  }
  if (
    typeof value.groupKey !== "string" || typeof value.sequence !== "number"
  ) {
    throw new Error("expected keyed jobs workflow output fields");
  }
  return {
    ...output,
    groupKey: value.groupKey,
    sequence: value.sequence,
  };
}

export function requireTerminalJobEdgesOutput(
  value: unknown,
): TerminalJobEdgesOutput {
  if (!isRecord(value)) {
    throw new Error("expected terminal job edges output");
  }
  if (
    typeof value.documentId !== "string" || typeof value.jobId !== "string" ||
    typeof value.waitState !== "string" || typeof value.getState !== "string" ||
    typeof value.cancelState !== "string"
  ) {
    throw new Error("expected terminal job edges output fields");
  }
  return {
    documentId: value.documentId,
    jobId: value.jobId,
    waitState: value.waitState,
    getState: value.getState,
    cancelState: value.cancelState,
  };
}

export function createJobsFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const jobsSchemas = {
    WorkflowInput: Type.Object({ documentId: Type.String() }),
    WorkflowOutput: Type.Object({
      documentId: Type.String(),
      jobId: Type.String(),
      processedBy: Type.String(),
      requestId: Type.String(),
      traceId: Type.String(),
    }),
    UpdateOperationInput: Type.Object({ documentId: Type.String() }),
    UpdateOperationUpdate: Type.Object({ processed: Type.Number() }),
    UpdateOperationOutput: Type.Object({
      jobId: Type.String(),
      total: Type.Number(),
    }),
    KeyedWorkflowInput: Type.Object({
      documentId: Type.String(),
      groupKey: Type.String(),
      sequence: Type.Number(),
    }),
    KeyedWorkflowOutput: Type.Object({
      documentId: Type.String(),
      groupKey: Type.String(),
      sequence: Type.Number(),
      jobId: Type.String(),
      processedBy: Type.String(),
      requestId: Type.String(),
      traceId: Type.String(),
    }),
    TerminalJobEdgesOutput: Type.Object({
      documentId: Type.String(),
      jobId: Type.String(),
      waitState: Type.String(),
      getState: Type.String(),
      cancelState: Type.String(),
    }),
    JobPayload: Type.Object({ documentId: Type.String() }),
    JobUpdate: Type.Object({
      processed: Type.Number(),
      marker: Type.String(),
    }),
    LongJobPayload: Type.Object({ documentId: Type.String() }),
    FailingJobPayload: Type.Object({ documentId: Type.String() }),
    JobResult: Type.Object({
      documentId: Type.String(),
      processedBy: Type.String(),
      requestId: Type.String(),
      traceId: Type.String(),
    }),
    KeyedJobPayload: Type.Object({
      documentId: Type.String(),
      groupKey: Type.String(),
      sequence: Type.Number(),
    }),
    KeyedJobResult: Type.Object({
      documentId: Type.String(),
      groupKey: Type.String(),
      sequence: Type.Number(),
      processedBy: Type.String(),
      requestId: Type.String(),
      traceId: Type.String(),
    }),
  } as const;

  const serviceContract = defineServiceContract(
    { schemas: jobsSchemas },
    (ref) => ({
      id: caseScopedContractId("trellis.integration.jobs-service", caseId),
      displayName: `Trellis Integration Jobs Service (${slug})`,
      description: "Exercises service-local jobs behind a client-visible RPC.",
      uses: [jobs({
        processDocument: {
          payload: ref.schema("JobPayload"),
          update: ref.schema("JobUpdate"),
          result: ref.schema("JobResult"),
        },
        longProcessDocument: {
          payload: ref.schema("LongJobPayload"),
          result: ref.schema("JobResult"),
        },
        failingProcessDocument: {
          payload: ref.schema("FailingJobPayload"),
          result: ref.schema("JobResult"),
          maxDeliver: 2,
          backoffMs: [0],
        },
        keyedProcessDocument: {
          payload: ref.schema("KeyedJobPayload"),
          result: ref.schema("KeyedJobResult"),
          keyConcurrency: {
            key: ["document", "/groupKey"],
            maxActive: 1,
            heartbeatIntervalMs: 1_000,
            heartbeatTtlMs: 10_000,
            stalePolicy: "fail-stale",
          },
          queue: {
            maxQueuedPerKey: 1,
            whenFull: "reject",
          },
        },
        keyedCoalesceProcessDocument: {
          payload: ref.schema("KeyedJobPayload"),
          result: ref.schema("KeyedJobResult"),
          keyConcurrency: {
            key: ["document", "/groupKey"],
            maxActive: 1,
            heartbeatIntervalMs: 1_000,
            heartbeatTtlMs: 10_000,
            stalePolicy: "fail-stale",
          },
          queue: {
            maxQueuedPerKey: 1,
            whenFull: "coalesce",
          },
        },
        keyedReplaceProcessDocument: {
          payload: ref.schema("KeyedJobPayload"),
          result: ref.schema("KeyedJobResult"),
          keyConcurrency: {
            key: ["document", "/groupKey"],
            maxActive: 1,
            heartbeatIntervalMs: 1_000,
            heartbeatTtlMs: 10_000,
            stalePolicy: "fail-stale",
          },
          queue: {
            maxQueuedPerKey: 1,
            whenFull: "replace-oldest",
          },
        },
      })],
      operations: {
        "Documents.ProcessWithUpdates": {
          version: "v1",
          subject: caseScopedSubject(
            "operations.v1.Integration.Jobs",
            caseId,
            "Documents.ProcessWithUpdates",
          ),
          input: ref.schema("UpdateOperationInput"),
          update: ref.schema("UpdateOperationUpdate"),
          output: ref.schema("UpdateOperationOutput"),
          capabilities: { call: [], observe: [] },
          errors: [ref.error("UnexpectedError")],
        },
      },
      rpc: {
        "Documents.Process": {
          version: "v1",
          subject: caseScopedSubject(
            "rpc.v1.Integration.Jobs",
            caseId,
            "Documents.Process",
          ),
          input: ref.schema("WorkflowInput"),
          output: ref.schema("WorkflowOutput"),
          capabilities: { call: [] },
          errors: [],
        },
        "Documents.KeyedProcess": {
          version: "v1",
          subject: caseScopedSubject(
            "rpc.v1.Integration.Jobs",
            caseId,
            "Documents.KeyedProcess",
          ),
          input: ref.schema("KeyedWorkflowInput"),
          output: ref.schema("KeyedWorkflowOutput"),
          capabilities: { call: [] },
          errors: [],
        },
        "Documents.SubmitLongProcess": {
          version: "v1",
          subject: caseScopedSubject(
            "rpc.v1.Integration.Jobs",
            caseId,
            "Documents.SubmitLongProcess",
          ),
          input: ref.schema("WorkflowInput"),
          output: ref.schema("WorkflowOutput"),
          capabilities: { call: [] },
          errors: [],
        },
        "Documents.TerminalLocalEdges": {
          version: "v1",
          subject: caseScopedSubject(
            "rpc.v1.Integration.Jobs",
            caseId,
            "Documents.TerminalLocalEdges",
          ),
          input: ref.schema("WorkflowInput"),
          output: ref.schema("TerminalJobEdgesOutput"),
          capabilities: { call: [] },
          errors: [],
        },
      },
    }),
  );

  const clientContract = defineAppContract(() => ({
    id: caseScopedContractId("trellis.integration.jobs-client", caseId),
    displayName: `Trellis Integration Jobs Client (${slug})`,
    description: "App/client participant for the jobs integration fixture.",
    uses: [
      serviceContract.DocumentsProcessWithUpdates,
      serviceContract.DocumentsProcess,
      serviceContract.DocumentsKeyedProcess,
      serviceContract.DocumentsSubmitLongProcess,
      serviceContract.DocumentsTerminalLocalEdges,
    ],
  }));

  const serviceName = caseScopedName("jobs-fixture-service", caseId);

  async function connectService(runtime: LiveTrellisRuntime) {
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
    });
    return await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      sessionKeySeed: serviceKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
  }

  async function mountWorkflow(
    service: Awaited<ReturnType<typeof connectService>>,
  ) {
    service.jobs.processDocument.handle(async ({ job }) => {
      await job.progress({
        step: "process",
        current: 1,
        total: 1,
        message: `processed ${job.payload.documentId}`,
      }).orThrow();
      await job.log({
        timestamp: new Date().toISOString(),
        level: "info",
        message: `processed ${job.payload.documentId}`,
      }).orThrow();
      return Result.ok({
        documentId: job.payload.documentId,
        processedBy: "ts-service-job",
        requestId: job.context.requestId,
        traceId: job.context.traceId,
      });
    });

    await service.handleDocumentsProcess(async ({ input, client }) => {
      const ref = await client.jobs.processDocument.create({
        documentId: input.documentId,
      }).orThrow();
      const terminal = await assertJobCompleted(ref, {
        documentId: input.documentId,
        processedBy: "ts-service-job",
      });
      if (terminal.result === undefined) {
        throw new Error(`job ${ref.id} completed without a result`);
      }
      return Result.ok({
        documentId: terminal.result.documentId,
        jobId: ref.id,
        processedBy: terminal.result.processedBy,
        requestId: terminal.result.requestId,
        traceId: terminal.result.traceId,
      });
    });
  }

  async function mountKeyedSerializationWorkflow(
    service: Awaited<ReturnType<typeof connectService>>,
  ) {
    const started: number[] = [];
    const completed: number[] = [];
    let released = false;
    let secondStartedBeforeRelease = false;
    let resolveFirstStarted!: () => void;
    let resolveReleaseFirst!: () => void;
    const firstStarted = new Promise<void>((resolve) => {
      resolveFirstStarted = resolve;
    });
    const releaseFirstWait = new Promise<void>((resolve) => {
      resolveReleaseFirst = resolve;
    });

    service.jobs.keyedProcessDocument.handle(async ({ job }) => {
      started.push(job.payload.sequence);
      if (job.payload.sequence === 1) {
        resolveFirstStarted();
        await releaseFirstWait;
      } else if (!released) {
        secondStartedBeforeRelease = true;
      }
      completed.push(job.payload.sequence);
      return Result.ok({
        documentId: job.payload.documentId,
        groupKey: job.payload.groupKey,
        sequence: job.payload.sequence,
        processedBy: "ts-service-keyed-job",
        requestId: job.context.requestId,
        traceId: job.context.traceId,
      });
    }, { concurrency: 2 });

    await service.handleDocumentsKeyedProcess(async ({ input, client }) => {
      const ref = await client.jobs.keyedProcessDocument.create({
        documentId: input.documentId,
        groupKey: input.groupKey,
        sequence: input.sequence,
      }).orThrow();
      const terminal = await assertJobCompleted(ref, {
        documentId: input.documentId,
        groupKey: input.groupKey,
        sequence: input.sequence,
        processedBy: "ts-service-keyed-job",
      });
      if (terminal.result === undefined) {
        throw new Error(`job ${ref.id} completed without a result`);
      }
      return Result.ok({
        documentId: terminal.result.documentId,
        groupKey: terminal.result.groupKey,
        sequence: terminal.result.sequence,
        jobId: ref.id,
        processedBy: terminal.result.processedBy,
        requestId: terminal.result.requestId,
        traceId: terminal.result.traceId,
      });
    },);

    return {
      firstStarted,
      releaseFirst() {
        released = true;
        resolveReleaseFirst();
      },
      started: () => [...started],
      completed: () => [...completed],
      secondStartedBeforeRelease: () => secondStartedBeforeRelease,
    };
  }

  async function mountLongRunningWorkflow(
    service: Awaited<ReturnType<typeof connectService>>,
  ) {
    let resolveStarted!: () => void;
    const started = new Promise<void>((resolve) => {
      resolveStarted = resolve;
    });

    service.jobs.longProcessDocument.handle(async ({ job }) => {
      resolveStarted();
      while (!job.cancelled) {
        await job.heartbeat().orThrow();
        await new Promise((resolve) => setTimeout(resolve, 25));
      }
      return Result.ok({
        documentId: job.payload.documentId,
        processedBy: "ts-service-long-job",
        requestId: job.context.requestId,
        traceId: job.context.traceId,
      });
    });

    await service.handleDocumentsSubmitLongProcess(async ({ input, client }) => {
      const ref = await client.jobs.longProcessDocument.create({
        documentId: input.documentId,
      }).orThrow();
      await started;
      const cancelled = await ref.cancel().orThrow();
      const terminal = await ref.wait().orThrow();
      return Result.ok({
        documentId: input.documentId,
        jobId: ref.id,
        processedBy: terminal.state,
        requestId: cancelled.context.requestId,
        traceId: cancelled.context.traceId,
      });
    },);

    return { started };
  }

  async function mountRetryThenDeadWorkflow(
    service: Awaited<ReturnType<typeof connectService>>,
  ) {
    const attempts: number[] = [];

    service.jobs.failingProcessDocument.handle(async ({ job }) => {
      attempts.push(job.redeliveryCount());
      return Result.err(
        new RetryJobError({
          message: `retry ${job.payload.documentId}`,
          context: { documentId: job.payload.documentId },
        }),
      );
    });

    return { attempts: () => [...attempts] };
  }

  async function mountTerminalLocalEdgesWorkflow(
    service: Awaited<ReturnType<typeof connectService>>,
  ) {
    service.jobs.processDocument.handle(async ({ job }) => {
      return Result.ok({
        documentId: job.payload.documentId,
        processedBy: "ts-service-job",
        requestId: job.context.requestId,
        traceId: job.context.traceId,
      });
    });

    await service.handleDocumentsTerminalLocalEdges(async ({ input, client }) => {
      const ref = await client.jobs.processDocument.create({
        documentId: input.documentId,
      }).orThrow();
      const terminal = await ref.wait().orThrow();
      const snapshot = await ref.get().orThrow();
      const cancelAfterTerminal = await ref.cancel().orThrow();
      return Result.ok({
        documentId: input.documentId,
        jobId: ref.id,
        waitState: terminal.state,
        getState: snapshot.state,
        cancelState: cancelAfterTerminal.state,
      });
    },);
  }

  return {
    slug,
    serviceContract,
    clientContract,
    serviceName,
    clientName: caseScopedName("jobs-fixture-client", caseId),
    documentId: caseScopedName("doc", caseId),
    connectService,
    mountWorkflow,
    mountKeyedSerializationWorkflow,
    mountLongRunningWorkflow,
    mountRetryThenDeadWorkflow,
    mountTerminalLocalEdgesWorkflow,
  };
}
