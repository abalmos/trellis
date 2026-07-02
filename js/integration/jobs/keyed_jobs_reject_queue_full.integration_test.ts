import { assertEquals } from "@std/assert";
import { assertJobCompleted } from "@qlever-llc/trellis-test";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID = "jobs.keyed-jobs-reject-queue-full" as const;
const fixture = createJobsFixture(CASE_ID);

liveTrellisTest({
  name:
    "jobs.keyed-jobs-reject-queue-full rejects same-key job when queue is full",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    let serviceWait: Promise<void> | undefined;
    let releaseFirst: (() => void) | undefined;
    let firstStarted = false;
    let firstReleased = false;

    try {
      const controls = await fixture.mountKeyedSerializationWorkflow(service);
      releaseFirst = () => controls.releaseFirst();
      serviceWait = service.wait();

      const groupKey = `${fixture.documentId}-same-key`;
      const first = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-1`,
        groupKey,
        sequence: 1,
      }).orThrow();
      if (first.kind !== "accepted") {
        throw new Error(`first keyed job was not accepted: ${first.kind}`);
      }
      await controls.firstStarted;
      firstStarted = true;

      const second = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-2`,
        groupKey,
        sequence: 2,
      }).orThrow();
      if (second.kind !== "accepted") {
        throw new Error(`second keyed job was not accepted: ${second.kind}`);
      }

      const third = await service.jobs.keyedProcessDocument.submit({
        documentId: `${fixture.documentId}-3`,
        groupKey,
        sequence: 3,
      }).orThrow();
      if (third.kind !== "rejected") {
        throw new Error(`third keyed job was not rejected: ${third.kind}`);
      }
      assertEquals(third.reason, "active-limit");
      assertEquals(third.key, `document:${groupKey}`);
      assertEquals(third.active, 1);
      assertEquals(third.queued, 1);
      assertEquals(third.limit, 1);

      const secondStartedBeforeRelease = controls.secondStartedBeforeRelease();
      const startedBeforeRelease = controls.started();

      controls.releaseFirst();
      firstReleased = true;
      assertEquals(secondStartedBeforeRelease, false);
      assertEquals(startedBeforeRelease, [1]);
      await assertJobCompleted(first.ref, {
        documentId: `${fixture.documentId}-1`,
        groupKey,
        sequence: 1,
        processedBy: "ts-service-keyed-job",
      });
      await assertJobCompleted(second.ref, {
        documentId: `${fixture.documentId}-2`,
        groupKey,
        sequence: 2,
        processedBy: "ts-service-keyed-job",
      });
      assertEquals(controls.completed(), [1, 2]);
    } finally {
      if (firstStarted && !firstReleased) {
        releaseFirst?.();
      }
      await service.stop();
      await serviceWait;
    }
  },
});
