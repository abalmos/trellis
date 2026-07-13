import { assert, assertEquals } from "@std/assert";
import { defineAppContract, Result, RetryJobError } from "@qlever-llc/trellis";
import * as trellisJobs from "@qlever-llc/trellis/sdk/jobs";

import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID =
  "jobs.live-updates-are-typed-and-stop-at-terminal-state" as const;
const fixture = createJobsFixture(CASE_ID);
const marker = `transient-${fixture.slug}`;

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId("trellis.integration.jobs-admin-client", CASE_ID),
  displayName: `Trellis Integration Jobs Admin Client (${fixture.slug})`,
  description: "Inspects job lifecycle state without transient update bodies.",
  uses: [trellisJobs.JobsInspect],
}));

liveTrellisTest({
  name:
    "jobs.live-updates-are-typed-and-stop-at-terminal-state relays typed job updates through an operation",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    let serviceWait: Promise<void> | undefined;
    let releaseUpdates!: () => void;
    let emitAfterCompletion: (() => Promise<boolean>) | undefined;
    const updatesSubscribed = new Promise<void>((resolve) => {
      releaseUpdates = resolve;
    });
    let releaseRetryUpdates!: () => void;
    const retryUpdatesSubscribed = new Promise<void>((resolve) => {
      releaseRetryUpdates = resolve;
    });
    try {
      service.jobs.processDocument.handle(async ({ job }) => {
        if (job.payload.documentId.endsWith("-retry")) {
          await retryUpdatesSubscribed;
          const updateCount = job.redeliveryCount() === 0 ? 6 : 2;
          for (let processed = 1; processed <= updateCount; processed += 1) {
            await job.emitUpdate({ processed, marker }).orThrow();
          }
          if (job.redeliveryCount() === 0) {
            return Result.err(new RetryJobError({ message: "retry updates" }));
          }
        } else {
          await updatesSubscribed;
          await job.emitUpdate({ processed: 1, marker }).orThrow();
          await job.emitUpdate({ processed: 2, marker }).orThrow();
        }
        emitAfterCompletion = async () =>
          (await job.emitUpdate({ processed: 99, marker })).isErr();
        return Result.ok({
          documentId: job.payload.documentId,
          processedBy: "ts-service-job",
          requestId: job.context.requestId,
          traceId: job.context.traceId,
        });
      });

      await service.handleDocumentsProcessWithUpdates(async ({ op }) => {
        await op.started().orThrow();
        return op.defer();
      });
      serviceWait = service.wait();

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });
      const received: number[] = [];
      const observedByBuilder: number[] = [];
      const operation = await client.documentsProcessWithUpdates({
        documentId: fixture.documentId,
      })
        .onUpdate((event) => {
          observedByBuilder.push(event.update.processed);
        })
        .start().orThrow();
      const operationEvents = await operation.watch({ updates: true })
        .orThrow();
      const observer = (async () => {
        for await (const event of operationEvents) {
          if (event.type === "update") {
            received.push(event.update.processed);
          }
        }
      })();

      const controlled = await service.handleDocumentsProcessWithUpdates
        .control(operation.id).orThrow();
      const job = await service.jobs.processDocument.create({
        documentId: fixture.documentId,
      }).orThrow();
      const updates = await service.jobs.processDocument.updates(job.id)
        .orThrow();
      let relayComplete!: () => void;
      const relayed = new Promise<void>((resolve) => {
        relayComplete = resolve;
      });
      const relay = (async () => {
        for await (const update of updates) {
          await controlled.emitUpdate({ processed: update.processed })
            .orThrow();
          if (update.processed === 2) relayComplete();
        }
      })();
      releaseUpdates();
      await relayed;
      updates.unsubscribe();
      await relay;
      await job.wait().orThrow();
      assert(await emitAfterCompletion?.());
      await controlled.complete({ jobId: job.id, total: 3 }).orThrow();

      const terminal = await operation.wait().orThrow();
      await observer;

      assertEquals(received, [1, 2]);
      assertEquals(observedByBuilder, [1, 2]);
      assertEquals(terminal.state, "completed");
      assert(terminal.output);
      const output = terminal.output;
      assertEquals(output.total, 3);
      assert(output.jobId);

      const retryJob = await service.jobs.processDocument.create({
        documentId: `${fixture.documentId}-retry`,
      }).orThrow();
      const retryUpdates = await service.jobs.processDocument.updates(
        retryJob.id,
      )
        .orThrow();
      const retryValues: number[] = [];
      const collectRetryUpdates = (async () => {
        for await (const update of retryUpdates) {
          retryValues.push(update.processed);
        }
      })();
      releaseRetryUpdates();
      assertEquals((await retryJob.wait().orThrow()).state, "completed");
      await runtime.waitFor(
        () => retryValues.length === 8,
        { timeoutMs: 60_000, intervalMs: 25 },
      );
      retryUpdates.unsubscribe();
      await collectRetryUpdates;
      assertEquals(retryValues, [1, 2, 3, 4, 5, 6, 1, 2]);

      const lateEvents = await operation.watch().orThrow();
      for await (const event of lateEvents) {
        assert(event.type !== "update");
      }

      const admin = await runtime.connectClient({
        name: caseScopedName("jobs-admin-client", CASE_ID),
        contract: adminContract,
      });
      const inspected = await runtime.waitFor(async () => {
        const current = await admin.jobsInspect({
          id: output.jobId,
        }).orThrow();
        return current.job.state === "completed" ? current : false;
      }, { timeoutMs: 60_000, intervalMs: 100 });
      assert(!JSON.stringify(inspected).includes(marker));
    } finally {
      releaseUpdates();
      releaseRetryUpdates();
      await service.stop().catch(() => undefined);
      await serviceWait?.catch(() => undefined);
    }
  },
});
