import { assert, assertEquals } from "@std/assert";
import { defineAppContract } from "@qlever-llc/trellis";
import { sdk as trellisJobs } from "@qlever-llc/trellis/sdk/jobs";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID = "jobs.failed-job-retries-then-dead" as const;
const fixture = createJobsFixture(CASE_ID);

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId("trellis.integration.jobs-admin-client", CASE_ID),
  displayName: "Trellis Integration Jobs Admin Client",
  description: "Observes jobs through the generated Jobs admin SDK surface.",
  uses: {
    required: {
      jobs: trellisJobs.use({ rpc: { call: ["Jobs.Inspect"] } }),
    },
  },
}));

liveTrellisTest({
  name:
    "jobs.failed-job-retries-then-dead retries explicit retry requests until dead",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    const admin = await runtime.connectClient({
      name: caseScopedName("jobs-admin-client", CASE_ID),
      contract: adminContract,
    });
    let serviceWait: Promise<void> | undefined;

    try {
      const controls = await fixture.mountRetryThenDeadWorkflow(service);
      serviceWait = service.wait();

      const ref = await service.jobs.failingProcessDocument.create({
        documentId: fixture.documentId,
      }).orThrow();

      let lastAdminState = "missing";
      const adminDead = await runtime.waitFor(async () => {
        const current = await admin.rpc.jobs.inspect({ id: ref.id })
          .orThrow();
        lastAdminState = current.job.state;
        return current.job.state === "dead" ? current.job : false;
      }, { timeoutMs: 15_000, intervalMs: 100 }).catch((cause) => {
        throw new Error(
          `timed out waiting for admin dead state; last state ${lastAdminState}; attempts ${controls.attempts().length}`,
          { cause },
        );
      });

      assertEquals(adminDead.id, ref.id);
      assertEquals(adminDead.state, "dead");
      assertEquals(adminDead.maxTries, 2);
      assert(controls.attempts().length > 1, "handler should be retried");
    } finally {
      await admin.connection.close().catch(() => undefined);
      await service.stop().catch(() => undefined);
      await serviceWait?.catch(() => undefined);
    }
  },
});
