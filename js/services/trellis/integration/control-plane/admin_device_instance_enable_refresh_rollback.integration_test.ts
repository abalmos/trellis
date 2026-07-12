import { assertEquals } from "@std/assert";
import { defineAppContract } from "@qlever-llc/trellis";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import {
  caseScopedContractId,
  caseScopedName,
} from "@qlever-llc/trellis-test/integration";
import { liveTrellisTest, runtimeScopeIsolated } from "../_support/runtime.ts";
import {
  assertRefreshHookFailure,
  restartWithFailOnceHook,
} from "./_auth_admin_refresh_rollback.ts";

const CASE_ID =
  "control-plane.admin-device-instance-enable-refresh-rollback" as const;
const deploymentId = caseScopedName("admin-device-instance-enable", CASE_ID);
const refreshHook = "auth.admin.deviceInstances.refreshActiveContracts";

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.admin-device-instance-enable-refresh-admin",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Device Instance Enable Rollback Admin",
  description:
    "Exercises Auth device instance enable refresh rollback through live Trellis.",
  uses: [
    trellisAuth.AuthDeploymentsCreate,
    trellisAuth.AuthDevicesDisable,
    trellisAuth.AuthDevicesEnable,
    trellisAuth.AuthDevicesList,
    trellisAuth.AuthDevicesProvision,
  ],
}));

liveTrellisTest({
  name:
    "control-plane.admin-device-instance-enable-refresh-rollback rolls back failed device instance enable refresh",
  scope: runtimeScopeIsolated(),
  async fn(runtime) {
    let instanceId = "";
    const admin = await runtime.connectClient({
      name: caseScopedName("admin-device-instance-enable-client", CASE_ID),
      contract: adminContract,
    });
    try {
      await admin.authDeploymentsCreate({
        kind: "device",
        deploymentId,
        reviewMode: "none",
      }).orThrow();
      const provisioned = await admin.authDevicesProvision({
        deploymentId,
        publicIdentityKey: caseScopedName("enable-device-key", CASE_ID),
        activationKey: caseScopedName("enable-activation-key", CASE_ID),
        metadata: { name: caseScopedName("enable-device", CASE_ID) },
      }).orThrow();
      instanceId = provisioned.instance.instanceId;
      await admin.authDevicesDisable({ instanceId }).orThrow();
    } finally {
      await admin.connection.close().catch(() => undefined);
    }

    await restartWithFailOnceHook(runtime, refreshHook);

    const restartedAdmin = await runtime.connectClient({
      name: caseScopedName(
        "admin-device-instance-enable-retry-client",
        CASE_ID,
      ),
      contract: adminContract,
    });
    try {
      const failedEnable = await restartedAdmin.authDevicesEnable({
        instanceId,
      });
      assertRefreshHookFailure(failedEnable, refreshHook);

      const page = await restartedAdmin.authDevicesList({
        deploymentId,
        limit: 500,
      }).orThrow();
      assertEquals(
        page.entries.find((device) => device.instanceId === instanceId)?.state,
        "disabled",
      );
    } finally {
      await restartedAdmin.connection.close().catch(() => undefined);
    }
  },
});
