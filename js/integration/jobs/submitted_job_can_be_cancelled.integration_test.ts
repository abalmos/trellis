import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture, requireJobsWorkflowOutput } from "./_fixture.ts";

const CASE_ID = "jobs.submitted-job-can-be-cancelled" as const;
const fixture = createJobsFixture(CASE_ID);

liveTrellisTest({
  name:
    "jobs.submitted-job-can-be-cancelled submits a long-running job and cancels it",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    let serviceWait: Promise<void> | undefined;

    try {
      await fixture.mountLongRunningWorkflow(service);
      serviceWait = service.wait();
      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });

      const result = requireJobsWorkflowOutput(
        await client.rpc.documents.submitLongProcess({
          documentId: fixture.documentId,
        }).orThrow(),
      );
      assertEquals(result.documentId, fixture.documentId);
      assertEquals(result.processedBy, "cancelled");
    } finally {
      await service.stop();
      await serviceWait;
    }
  },
});
