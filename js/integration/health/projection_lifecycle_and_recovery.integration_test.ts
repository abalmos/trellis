import { assert, assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createHealthFixture } from "./_fixture.ts";

const CASE_ID = "health.projection-lifecycle-and-recovery" as const;
const fixture = createHealthFixture(CASE_ID);

liveTrellisTest({
  name:
    "health.projection-lifecycle-and-recovery projects lifecycle and replays downtime samples",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const observer = await runtime.connectClient({
      name: "health-projection-observer",
      contract: fixture.observerContract,
    });
    const service = await fixture.setupService(runtime);
    const waitForHealth = async (status: string, afterRevision = -1) =>
      await runtime.waitFor(async () => {
        const response = await observer.healthQuery({
          participantKinds: ["service"],
          contractIds: [fixture.serviceContract.CONTRACT_ID],
          limit: 20,
          offset: 0,
        }).orThrow().catch(() => undefined);
        return response?.entries[0]?.effectiveStatus === status &&
            response.projection.revision > afterRevision
          ? response
          : undefined;
      }, { timeoutMs: 30_000, intervalMs: 100 });

    try {
      const healthy = await waitForHealth("healthy");
      const initialRevision = healthy.projection.revision;

      assert(runtime.restartControlPlane, "live runtime must support restart");
      await runtime.restartControlPlane();
      const recovered = await waitForHealth("healthy", initialRevision);
      assertEquals(recovered.projection.gapDetected, false);

      await service.stop();
      const offline = await waitForHealth("offline");
      assertEquals(offline.entries[0].offlineInstances, 1);
      await runtime.waitFor(async () => {
        const inspect = await observer.healthInspect({
          participantKind: "service",
          contractId: fixture.serviceContract.CONTRACT_ID,
          historyLimit: 20,
        }).orThrow();
        return inspect.history.some((interval) =>
            interval.effectiveStatus === "offline" &&
            interval.reason === "deadline-expired"
          )
          ? inspect
          : undefined;
      }, { timeoutMs: 5_000, intervalMs: 100 });
      const now = Date.now();
      await runtime.waitFor(async () => {
        const metrics = await observer.healthMetrics({
          participantKind: "service",
          contractId: fixture.serviceContract.CONTRACT_ID,
          start: new Date(now - 5 * 60_000).toISOString(),
          end: new Date(now + 1_000).toISOString(),
          stepMs: 300_000,
        }).orThrow();
        return metrics.summary.sampleCount >= 2 &&
            metrics.summary.transitions >= 1
          ? metrics
          : undefined;
      }, { timeoutMs: 10_000, intervalMs: 100 });
    } finally {
      await service.stop().catch(() => undefined);
    }
  },
});
