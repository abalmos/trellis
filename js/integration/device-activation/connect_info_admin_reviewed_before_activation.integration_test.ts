import { assertEquals } from "@std/assert";
import { TrellisDevice } from "@qlever-llc/trellis";
import { getDeviceConnectInfo } from "@qlever-llc/trellis/auth";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createDeviceActivationFixture } from "./_fixture.ts";

const CASE_ID =
  "device-activation.connect-info-admin-reviewed-before-activation" as const;
const fixture = createDeviceActivationFixture(CASE_ID);

liveTrellisTest({
  name:
    "device-activation.connect-info-admin-reviewed-before-activation returns admin-reviewed connect info before user activation",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const { admin, deploymentId } = await fixture.setupDeviceDeployment(
      runtime,
    );
    const { identity, rootSecret, provisioned } = await fixture
      .setupProvisionedDevice(admin, deploymentId);

    const connectInfo = await getDeviceConnectInfo({
      trellisUrl: runtime.trellisUrl,
      publicIdentityKey: identity.publicIdentityKey,
      identitySeed: identity.identitySeed,
      contractDigest: fixture.deviceContract.CONTRACT_DIGEST,
    });
    assertEquals(connectInfo.status, "ready");
    assertEquals(connectInfo.connectInfo.deploymentId, deploymentId);
    assertEquals(connectInfo.connectInfo.auth.authority, "admin_reviewed");

    const reviews = await admin.rpc.auth.deviceUserAuthoritiesReviewsList({
      deploymentId,
      instanceId: provisioned.instance.instanceId,
      limit: 20,
    }).orThrow();
    assertEquals(reviews.entries.length, 0);

    const device = await TrellisDevice.connect({
      trellisUrl: runtime.trellisUrl,
      contract: fixture.deviceContract,
      rootSecret,
      log: false,
    }).orThrow();
    try {
      const me = await device.request("Auth.Sessions.Me", {}).orThrow();
      assertEquals(me.participantKind, "device");
      assertEquals(me.device?.deploymentId, deploymentId);
      assertEquals(me.device?.runtimePublicKey, identity.publicIdentityKey);
    } finally {
      await device.connection.close();
    }
  },
});
