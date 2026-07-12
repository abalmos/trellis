import { assertEquals } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createAuthorityPlanFixture } from "./_fixture.ts";

const CASE_ID =
  "authority-plan.incompatible-migration-approved-replaces-contract" as const;
const fixture = createAuthorityPlanFixture(CASE_ID);

liveTrellisTest({
  name:
    "authority-plan.incompatible-migration-approved-replaces-contract accepts migration and enables replacement",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.deployments.create({
      id: fixture.strictDeployment,
      mutableDev: false,
    });
    const baseKey = await runtime.services.createInstance({
      deployment: fixture.strictDeployment,
      name: fixture.baseServiceName,
      contract: fixture.baseContract,
    });
    let baseService:
      | Awaited<ReturnType<typeof fixture.connectService<typeof fixture.baseContract>>>
      | undefined = await fixture.connectService({
        runtime,
        contract: fixture.baseContract,
        name: fixture.baseServiceName,
        seed: baseKey.seed,
      });
    let replacementService:
      | Awaited<ReturnType<typeof fixture.connectService<typeof fixture.incompatibleSchemaContract>>>
      | undefined;

    try {
      await baseService.handlePlanPing(({ input }) =>
        Result.ok({ message: fixture.pingMessage(input), variant: "base" }));
      const baseResult = await fixture.connectClientAndPing(runtime, "before");
      assertEquals(baseResult, { message: "before", variant: "base" });
      await baseService.stop();
      baseService = undefined;

      const replacementKey = await runtime.services.provisionInstanceOnly({
        deployment: fixture.strictDeployment,
      });
      const connectPromise = fixture.connectServicePending({
        runtime,
        contract: fixture.incompatibleSchemaContract,
        name: fixture.replacementServiceName,
        seed: replacementKey.seed,
      }).then((connected) => {
        replacementService = connected;
        return connected;
      });
      connectPromise.catch(() => undefined);
      const plan = await fixture.waitForPendingPlan(runtime, {
        deploymentId: fixture.strictDeployment,
        classification: "migration",
        contractDigest: fixture.incompatibleSchemaContract.CONTRACT_DIGEST,
      });
      assertEquals(plan.classification, "migration");
      await fixture.acceptPlan(runtime, plan);
      await runtime.deployments.waitReady(fixture.strictDeployment);

      replacementService = await connectPromise;
      await replacementService.handlePlanPing(({ input }) =>
        Result.ok({ count: fixture.pingCount(input), variant: "incompatible" }));

      const client = await runtime.connectClient({
        name: fixture.incompatibleClientName,
        contract: fixture.incompatibleClientContract,
      });
      const result = await client.planPing({ count: 7 }).orThrow();
      assertEquals(result, { count: 7, variant: "incompatible" });
    } finally {
      await replacementService?.stop();
      await baseService?.stop();
    }
  },
});
