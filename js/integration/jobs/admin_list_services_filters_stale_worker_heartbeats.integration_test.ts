import { assert, assertEquals, assertExists } from "@std/assert";
import { join } from "@std/path";
import { connect, credsAuthenticator } from "@nats-io/transport-deno";
import { defineAppContract } from "@qlever-llc/trellis";
import { sdk as trellisJobs } from "@qlever-llc/trellis/sdk/jobs";
import {
  caseScopedContractId,
  caseScopedName,
} from "@qlever-llc/trellis-test/integration";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createJobsFixture } from "./_fixture.ts";

const CASE_ID =
  "jobs.admin-list-services-filters-stale-worker-heartbeats" as const;

const fixture = createJobsFixture(CASE_ID);
const adminClientContract = defineAppContract(() => ({
  id: caseScopedContractId("trellis.integration.jobs-admin-client", CASE_ID),
  displayName: `Trellis Integration Jobs Admin Client (${fixture.slug})`,
  description: "Uses Jobs admin ListServices for worker-presence coverage.",
  uses: {
    required: {
      jobs: trellisJobs.use({ rpc: { call: ["Jobs.ListServices"] } }),
    },
  },
}));

const freshService = fixture.serviceName;
const secondFreshService = caseScopedName("jobs-fixture-service-b", CASE_ID);
const jobType = "processDocument";
const freshInstance = caseScopedName("fresh-worker", CASE_ID);
const secondFreshInstance = caseScopedName("fresh-worker-b", CASE_ID);
const staleInstance = caseScopedName("stale-worker", CASE_ID);

liveTrellisTest({
  name:
    "jobs.admin-list-services-filters-stale-worker-heartbeats reports fresh workers only",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const adminClient = await runtime.connectClient({
      name: caseScopedName("jobs-admin-client", CASE_ID),
      contract: adminClientContract,
    });
    const nc = await connect({
      servers: runtime.natsUrl,
      authenticator: credsAuthenticator(
        await Deno.readFile(
          join(runtime.workdir, "nats", "creds", "trellis-auth.creds"),
        ),
      ),
    });

    try {
      await publishWorkerHeartbeat(nc, {
        service: freshService,
        jobType,
        instanceId: freshInstance,
        timestamp: new Date().toISOString(),
      });
      await publishWorkerHeartbeat(nc, {
        service: secondFreshService,
        jobType,
        instanceId: secondFreshInstance,
        timestamp: new Date().toISOString(),
      });
      await publishWorkerHeartbeat(nc, {
        service: freshService,
        jobType,
        instanceId: staleInstance,
        timestamp: new Date(Date.now() - 5 * 60_000).toISOString(),
      });
      await nc.flush();

      const page = await runtime.waitFor(async () => {
        const current = await adminClient.rpc.jobs.listServices({ limit: 20 })
          .orThrow();
        return current.entries.some((entry) =>
            entry.name === freshService &&
            entry.workers.some((worker) =>
              worker.jobType === jobType && worker.instanceId === freshInstance
            )
          )
          ? current
          : false;
      }, { timeoutMs: 60_000, intervalMs: 100 });

      const freshEntry = page.entries.find((entry) =>
        entry.name === freshService
      );
      assertExists(freshEntry);
      assertEquals(freshEntry?.workers.map((worker) => worker.instanceId), [
        freshInstance,
      ]);
      assertEquals(freshEntry.workers[0]?.service, freshService);
      assertEquals(freshEntry.workers[0]?.jobType, jobType);

      const firstPage = await adminClient.rpc.jobs.listServices({ limit: 1 })
        .orThrow();
      assert(firstPage.count >= 2);
      const secondPage = await adminClient.rpc.jobs.listServices({
        limit: 1,
        offset: firstPage.nextOffset,
      }).orThrow();
      assertEquals(firstPage.nextOffset, 1);
      assertEquals(secondPage.offset, 1);
      assert(firstPage.entries[0]?.name !== secondPage.entries[0]?.name);
    } finally {
      await nc.close().catch(() => undefined);
      await adminClient.connection.close().catch(() => undefined);
    }
  },
});

type WorkerHeartbeat = {
  readonly service: string;
  readonly jobType: string;
  readonly instanceId: string;
  readonly timestamp: string;
};

async function publishWorkerHeartbeat(
  nc: { publish(subject: string, payload: Uint8Array): void | Promise<void> },
  heartbeat: WorkerHeartbeat,
): Promise<void> {
  await nc.publish(
    `trellis.jobs.workers.${heartbeat.service}.${heartbeat.jobType}.${heartbeat.instanceId}.heartbeat`,
    new TextEncoder().encode(JSON.stringify(heartbeat)),
  );
}
