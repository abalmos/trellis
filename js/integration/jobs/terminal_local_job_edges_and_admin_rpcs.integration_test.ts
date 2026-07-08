import { defineAppContract } from "@qlever-llc/trellis";
import { sdk as trellisJobs } from "@qlever-llc/trellis/sdk/jobs";
import { assert, assertEquals } from "@std/assert";

import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import {
  createJobsFixture,
  requireTerminalJobEdgesOutput,
} from "./_fixture.ts";

const CASE_ID = "jobs.terminal-local-job-edges-and-admin-rpcs" as const;
const fixture = createJobsFixture(CASE_ID);

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId("trellis.integration.jobs-admin-client", CASE_ID),
  displayName: "Trellis Integration Jobs Admin Client",
  description:
    "Observes and cancels jobs through the generated Jobs admin SDK.",
  uses: {
    required: {
      jobs: trellisJobs.use({
        rpc: { call: ["Jobs.Query", "Jobs.Inspect", "Jobs.Cancel"] },
      }),
    },
  },
}));

liveTrellisTest({
  name:
    "jobs.terminal-local-job-edges-and-admin-rpcs observes terminal local refs and admin RPCs",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    const admin = await runtime.connectClient({
      name: caseScopedName("jobs-admin-client", CASE_ID),
      contract: adminContract,
    });
    let serviceWait: Promise<void> | undefined;
    let client: Awaited<ReturnType<typeof runtime.connectClient>> | undefined;

    try {
      await fixture.mountTerminalLocalEdgesWorkflow(service);
      serviceWait = service.wait();
      client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });

      const result = requireTerminalJobEdgesOutput(
        await client.rpc.documents.terminalLocalEdges({
          documentId: fixture.documentId,
        }).orThrow(),
      );
      assertEquals(result.documentId, fixture.documentId);
      assertEquals(result.waitState, "completed");
      assertEquals(result.getState, "completed");
      assertEquals(result.cancelState, "completed");

      const adminJob = await runtime.waitFor(async () => {
        const current = await admin.rpc.jobs.inspect({ id: result.jobId })
          .orThrow();
        return current.job.state === "completed" ? current : false;
      }, { timeoutMs: 60_000, intervalMs: 100 });
      assertEquals(adminJob.job.id, result.jobId);
      assertEquals(adminJob.job.state, "completed");

      const listed = await admin.rpc.jobs.query({
        limit: 10,
        service: adminJob.job.service,
        type: adminJob.job.type,
        state: ["completed"],
      }).orThrow();
      assert(listed.entries.some((job) => job.id === result.jobId));

      const cancelled = await admin.rpc.jobs.cancel({ id: result.jobId })
        .orThrow();
      assertEquals(cancelled.job.id, result.jobId);
      assertEquals(cancelled.job.state, "completed");
    } finally {
      await client?.connection.close().catch(() => undefined);
      await admin.connection.close().catch(() => undefined);
      await service.stop().catch(() => undefined);
      await serviceWait?.catch(() => undefined);
    }
  },
});
