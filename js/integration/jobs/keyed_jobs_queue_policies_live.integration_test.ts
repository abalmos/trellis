import { assertEquals } from "@std/assert";
import { assertJobCompleted } from "@qlever-llc/trellis-test";
import { Result } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID = "jobs.keyed-jobs-queue-policies-live" as const;
const fixture = createJobsFixture(CASE_ID);

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

liveTrellisTest({
  name:
    "jobs.keyed-jobs-queue-policies-live coalesces, replaces, and removes queued keyed jobs",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    const holds = new Map<string, ReturnType<typeof deferred>>();
    const started: string[] = [];
    const completed: string[] = [];

    function hold(groupKey: string) {
      let current = holds.get(groupKey);
      if (!current) {
        current = deferred();
        holds.set(groupKey, current);
      }
      return current;
    }

    async function run(job: {
      payload: { documentId: string; groupKey: string; sequence: number };
      context: { requestId: string; traceId: string };
    }) {
      const marker = `${job.payload.groupKey}:${job.payload.sequence}`;
      started.push(marker);
      if (job.payload.sequence === 1) {
        await hold(job.payload.groupKey).promise;
      }
      completed.push(marker);
      return Result.ok({
        documentId: job.payload.documentId,
        groupKey: job.payload.groupKey,
        sequence: job.payload.sequence,
        processedBy: "ts-service-keyed-job",
        requestId: job.context.requestId,
        traceId: job.context.traceId,
      });
    }

    try {
      service.jobs.keyedProcessDocument.handle(({ job }) => run(job));
      service.jobs.keyedCoalesceProcessDocument.handle(({ job }) => run(job));
      service.jobs.keyedReplaceProcessDocument.handle(({ job }) => run(job));
      const serviceWait = service.wait();

      const coalesceKey = `${fixture.documentId}-coalesce`;
      const firstCoalesce = await service.jobs.keyedCoalesceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-coalesce-1`,
          groupKey: coalesceKey,
          sequence: 1,
        }).orThrow();
      if (firstCoalesce.kind !== "accepted") {
        throw new Error(
          `first coalesce was not accepted: ${firstCoalesce.kind}`,
        );
      }
      await runtime.waitFor(() => started.includes(`${coalesceKey}:1`));
      const secondCoalesce = await service.jobs.keyedCoalesceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-coalesce-2`,
          groupKey: coalesceKey,
          sequence: 2,
        }).orThrow();
      if (secondCoalesce.kind !== "accepted") {
        throw new Error(
          `second coalesce was not accepted: ${secondCoalesce.kind}`,
        );
      }
      const thirdCoalesce = await service.jobs.keyedCoalesceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-coalesce-3`,
          groupKey: coalesceKey,
          sequence: 3,
        }).orThrow();
      assertEquals(thirdCoalesce.kind, "coalesced");
      if (thirdCoalesce.kind === "coalesced") {
        assertEquals(thirdCoalesce.existing.id, secondCoalesce.ref.id);
        assertEquals(thirdCoalesce.reason, "queue-full");
      }
      hold(coalesceKey).resolve();
      await assertJobCompleted(firstCoalesce.ref, { sequence: 1 });
      await assertJobCompleted(secondCoalesce.ref, { sequence: 2 });

      const replaceKey = `${fixture.documentId}-replace`;
      const firstReplace = await service.jobs.keyedReplaceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-replace-1`,
          groupKey: replaceKey,
          sequence: 1,
        }).orThrow();
      if (firstReplace.kind !== "accepted") {
        throw new Error(`first replace was not accepted: ${firstReplace.kind}`);
      }
      await runtime.waitFor(() => started.includes(`${replaceKey}:1`));
      const secondReplace = await service.jobs.keyedReplaceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-replace-2`,
          groupKey: replaceKey,
          sequence: 2,
        }).orThrow();
      if (secondReplace.kind !== "accepted") {
        throw new Error(
          `second replace was not accepted: ${secondReplace.kind}`,
        );
      }
      const thirdReplace = await service.jobs.keyedReplaceProcessDocument
        .submit({
          documentId: `${fixture.documentId}-replace-3`,
          groupKey: replaceKey,
          sequence: 3,
        }).orThrow();
      assertEquals(thirdReplace.kind, "replaced");
      if (thirdReplace.kind !== "replaced") {
        throw new Error("expected replace-oldest outcome");
      }
      assertEquals(thirdReplace.replaced.id, secondReplace.ref.id);
      assertEquals((await secondReplace.ref.wait().orThrow()).state, "skipped");
      hold(replaceKey).resolve();
      await assertJobCompleted(firstReplace.ref, { sequence: 1 });
      await assertJobCompleted(thirdReplace.ref, { sequence: 3 });

      const removeKey = `${fixture.documentId}-remove`;
      const firstRemove = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-remove-1`,
        groupKey: removeKey,
        sequence: 1,
      }).orThrow();
      if (firstRemove.kind !== "accepted") {
        throw new Error(`first remove was not accepted: ${firstRemove.kind}`);
      }
      await runtime.waitFor(() => started.includes(`${removeKey}:1`));
      const secondRemove = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-remove-2`,
        groupKey: removeKey,
        sequence: 2,
      }).orThrow();
      if (secondRemove.kind !== "accepted") {
        throw new Error(`second remove was not accepted: ${secondRemove.kind}`);
      }
      assertEquals(
        (await secondRemove.ref.cancel().orThrow()).state,
        "cancelled",
      );
      const thirdRemove = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-remove-3`,
        groupKey: removeKey,
        sequence: 3,
      }).orThrow();
      if (thirdRemove.kind !== "rejected") {
        throw new Error(`third remove was not rejected: ${thirdRemove.kind}`);
      }
      assertEquals(thirdRemove.reason, "active-limit");
      assertEquals(thirdRemove.active, 1);
      assertEquals(thirdRemove.queued, 1);
      assertEquals(thirdRemove.limit, 1);
      hold(removeKey).resolve();
      await assertJobCompleted(firstRemove.ref, { sequence: 1 });
      assertEquals(completed.includes(`${removeKey}:2`), false);
      assertEquals(completed.includes(`${removeKey}:3`), false);

      await service.stop();
      await serviceWait;
    } finally {
      for (const gate of holds.values()) {
        gate.resolve();
      }
      await service.stop().catch(() => undefined);
    }
  },
});
