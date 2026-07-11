import { assert, assertEquals, assertNotEquals } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createOutboxFixture } from "./_fixture.ts";

const CASE_ID = "outbox.dispatches-jobs-through-sql-outbox" as const;
const fixture = createOutboxFixture(CASE_ID);

liveTrellisTest({
  name:
    "outbox.dispatches-jobs-through-sql-outbox creates and submits durable jobs after commit",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const db = await fixture.createDb();
    const runtimeName = `@trellis/${fixture.slug}`;
    const service = await fixture.connectService(
      runtime,
      runtimeName,
    );
    let serviceWait: Promise<void> | undefined;
    const processed: string[] = [];
    const jobServices: string[] = [];
    const submissionIds: string[] = [];
    const keyedSubmissionIds: string[] = [];

    try {
      const sqlOutbox = fixture.createOutbox(service, db);
      service.jobs.syncCustomer.handle(({ job }) => {
        processed.push(job.payload.customerId);
        jobServices.push(job.ref.service);
        return Promise.resolve(Result.ok({ ok: true }));
      });
      await service.handle.rpc.documents.syncCustomer(async ({ input }) => {
        await sqlOutbox.transaction(async ({ job }) => {
          const created = await job.syncCustomer.create({
            customerId: `${input.customerId}-create`,
          }).orThrow();
          const submitted = await job.syncCustomer.submit({
            customerId: `${input.customerId}-submit`,
          }).orThrow();
          const keyedAccepted = await job.keyedCustomer.create({
            customerId: input.customerId,
          }).orThrow();
          const keyedRejected = await job.keyedCustomer.create({
            customerId: input.customerId,
          }).orThrow();
          submissionIds.push(created.submissionId, submitted.submissionId);
          keyedSubmissionIds.push(
            keyedAccepted.submissionId,
            keyedRejected.submissionId,
          );
        }).orThrow();
        return Result.ok({ ok: true });
      });
      serviceWait = service.wait();

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });
      assertEquals(
        await client.rpc.documents.syncCustomer({
          customerId: fixture.documentId,
        })
          .orThrow(),
        { ok: true },
      );

      await runtime.waitFor(() => processed.length === 2);
      assertEquals(processed.toSorted(), [
        `${fixture.documentId}-create`,
        `${fixture.documentId}-submit`,
      ]);
      assertEquals(jobServices.length, 2);
      for (const serviceName of jobServices) {
        assertNotEquals(serviceName, runtimeName);
      }
      for (const submissionId of submissionIds) {
        assertEquals(
          (await sqlOutbox.jobSubmissionOutcome(submissionId).orThrow())?.kind,
          "accepted",
        );
      }
      await runtime.waitFor(async () =>
        (await sqlOutbox.jobSubmissionOutcome(keyedSubmissionIds[1]!)
          .orThrow()) !==
          undefined
      );
      assertEquals(
        (await sqlOutbox.jobSubmissionOutcome(keyedSubmissionIds[0]!).orThrow())
          ?.kind,
        "accepted",
      );
      const rejected = await sqlOutbox.jobSubmissionOutcome(
        keyedSubmissionIds[1]!,
      ).orThrow();
      assertEquals(rejected?.kind, "rejected");
      assertEquals(rejected?.reason, "queue-depth");

      const rows = db.exec(
        "SELECT kind, state, outcome FROM trellis_outbox ORDER BY kind",
      )[0];
      assertEquals(
        rows?.values.map(([kind, state]) => ({ kind, state })),
        [
          { kind: "job.create", state: "dispatched" },
          { kind: "job.create", state: "dispatched" },
          { kind: "job.create", state: "dispatched" },
          { kind: "job.submit", state: "dispatched" },
        ],
      );
      const outcomes = (rows?.values ?? []).map((row) =>
        JSON.parse(String(row[2]))
      );
      assertEquals(outcomes.map((outcome) => outcome.kind).toSorted(), [
        "accepted",
        "accepted",
        "accepted",
        "rejected",
      ]);
      for (
        const outcome of outcomes.filter((outcome) =>
          outcome.kind === "accepted"
        )
      ) {
        assert(typeof outcome.jobId === "string");
      }
    } finally {
      await service.stop();
      await serviceWait;
      db.close();
    }
  },
});
