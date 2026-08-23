import {
  defineAppContract,
  defineDeviceContract,
  state,
  TrellisDevice,
} from "@qlever-llc/trellis";
import { deriveDeviceIdentity } from "@qlever-llc/trellis/auth";
import { checkDeviceActivation } from "@qlever-llc/trellis/device/deno";
import { ValidationError } from "@qlever-llc/trellis/errors";
import { AuthDeviceUserAuthoritiesResolve } from "@qlever-llc/trellis/sdk/auth";
import { assertOperationCompleted } from "@qlever-llc/trellis-test";
import { assert, assertEquals, assertInstanceOf } from "@std/assert";
import { Type } from "typebox";

import type { LiveTrellisRuntime } from "../_support/runtime.ts";

const adminContract = defineAppContract(() => ({
  id:
    "trellis.integration.state-device-admin.state-activated-devices-rust-owner@v1",
  displayName: "Trellis Integration State Device Admin",
  description: "Provisions activated devices and inspects their State.",
  uses: [AuthDeviceUserAuthoritiesResolve],
}));

const deviceContract = defineDeviceContract(
  {
    schemas: {
      Preferences: Type.Object({ theme: Type.String() }),
      Draft: Type.Object({ title: Type.String() }),
    },
  },
  (ref) => ({
    id:
      "trellis.integration.state-device.state-activated-devices-rust-owner@v1",
    displayName: "Trellis Integration State Device",
    description:
      "Exercises device-owned State through the public device client.",
    uses: [state({
      preferences: { kind: "value", schema: ref.schema("Preferences") },
      drafts: { kind: "map", schema: ref.schema("Draft") },
    })],
  }),
);

export async function exerciseDeviceState(runtime: LiveTrellisRuntime) {
  const admin = await runtime.connectClient({
    name: "state-device-admin-state-activated-devices-rust-owner",
    contract: adminContract,
  });
  const deploymentName =
    "state-device-deployment-state-activated-devices-rust-owner";
  await runtime.deployments.create({ id: deploymentName, kind: "device" });
  const approval = await runtime.contracts.approve({
    deployment: deploymentName,
    contract: deviceContract,
  });

  async function activateDevice() {
    const rootSecret = crypto.getRandomValues(new Uint8Array(32));
    const identity = await deriveDeviceIdentity(rootSecret);
    const provisioned = await runtime.devices.provision({
      deploymentId: approval.deploymentId,
      idempotencyKey: crypto.randomUUID(),
      identityPublicKey: null,
      instanceId: null,
      participantId: approval.participantId,
    });
    assert(provisioned.provisioningSecret !== null);
    const provisionedIdentity = {
      deploymentId: approval.deploymentId,
      instanceId: provisioned.device.instanceId,
      principalId: provisioned.device.principalId,
      participantId: approval.participantId,
      participantArtifactDigest: approval.participantDigest,
      participantNeedsDigest: approval.participantNeedsDigest,
      provisioningSecret: provisioned.provisioningSecret,
      expectedSecretVersion: 1,
    };
    const activation = await checkDeviceActivation({
      trellisUrl: runtime.trellisUrl,
      contract: deviceContract,
      rootSecret,
      identity: provisionedIdentity,
      stateDir: await Deno.makeTempDir({ prefix: "trellis-state-device-" }),
    });
    assertEquals(activation.status, "activation_required");
    if (activation.status !== "activation_required") {
      throw new Error(
        `expected activation_required, got ${activation.status}`,
      );
    }
    const flowId = new URL(activation.activationUrl, runtime.trellisUrl)
      .searchParams.get("flowId");
    assert(flowId !== null);
    const activationRef = await admin.authDeviceUserAuthoritiesResolve({
      confirmationCode: activation.confirmationCode,
      flowId,
    }).start().orThrow();
    await assertOperationCompleted(activationRef);
    await activation.waitForOnlineApproval();
    const device = await TrellisDevice.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: deviceContract,
      rootSecret,
      identity: provisionedIdentity,
      log: false,
    }).orThrow();
    return { device, identity, provisioned, provisionedIdentity, rootSecret };
  }

  const deviceA = await activateDevice();
  const valueA = await deviceA.device.state.preferences.put({ theme: "A" })
    .orThrow();
  assertEquals(valueA.applied, true);
  const readValueA = await deviceA.device.state.preferences.get().orThrow();
  assert(!("migrationRequired" in readValueA));
  assert(readValueA.found);
  assertEquals(readValueA.entry.value.theme, "A");
  await deviceA.device.state.drafts.put("shared", { title: "A" }).orThrow();
  const readMapA = await deviceA.device.state.drafts.get("shared").orThrow();
  assert(!("migrationRequired" in readMapA));
  assert(readMapA.found);
  assertEquals(readMapA.entry.value.title, "A");
  const listedA = await deviceA.device.state.drafts.list().orThrow();
  assertEquals(listedA.count, 1);

  const deviceB = await activateDevice();
  const missingB = await deviceB.device.state.preferences.get().orThrow();
  assert(!("migrationRequired" in missingB));
  assertEquals(missingB.found, false);
  const missingMapB = await deviceB.device.state.drafts.get("shared")
    .orThrow();
  assert(!("migrationRequired" in missingMapB));
  assertEquals(missingMapB.found, false);
  await deviceB.device.state.preferences.put({ theme: "B" }).orThrow();
  await deviceB.device.state.drafts.put("shared", { title: "B" }).orThrow();
  for (
    const [device, theme, title] of [
      [deviceA.device, "A", "A"],
      [deviceB.device, "B", "B"],
    ] as const
  ) {
    const value = await device.state.preferences.get().orThrow();
    assert(!("migrationRequired" in value));
    assert(value.found);
    assertEquals(value.entry.value.theme, theme);
    const map = await device.state.drafts.get("shared").orThrow();
    assert(!("migrationRequired" in map));
    assert(map.found);
    assertEquals(map.entry.value.title, title);
  }
  assertEquals(
    (await deviceB.device.state.preferences.delete().orThrow()).deleted,
    true,
  );
  assertEquals(
    (await deviceB.device.state.drafts.delete("shared").orThrow()).deleted,
    true,
  );
  const deletedValueB = await deviceB.device.state.preferences.get().orThrow();
  assert(!("migrationRequired" in deletedValueB));
  assertEquals(deletedValueB.found, false);
  const deletedMapB = await deviceB.device.state.drafts.get("shared").orThrow();
  assert(!("migrationRequired" in deletedMapB));
  assertEquals(deletedMapB.found, false);

  await deviceA.device.connection.close();
  const reconnectedA = await TrellisDevice.connect({
    authorizationContextEphemeral: true,
    trellisUrl: runtime.trellisUrl,
    contract: deviceContract,
    rootSecret: deviceA.rootSecret,
    identity: deviceA.provisionedIdentity,
    log: false,
  }).orThrow();
  const persistedA = await reconnectedA.state.preferences.get().orThrow();
  assert(!("migrationRequired" in persistedA));
  assert(persistedA.found);
  assertEquals(persistedA.entry.value.theme, "A");
  const persistedMapA = await reconnectedA.state.drafts.get("shared").orThrow();
  assert(!("migrationRequired" in persistedMapA));
  assert(persistedMapA.found);
  assertEquals(persistedMapA.entry.value.title, "A");

  const adminTarget = {
    contractDigest: deviceContract.CONTRACT_DIGEST,
    contractId: deviceContract.CONTRACT_ID,
    deviceId: deviceA.provisioned.device.principalId,
    scope: "deviceApp" as const,
  };
  const adminGet = await runtime.state.adminGet({
    ...adminTarget,
    store: "preferences",
  });
  assert(!("migrationRequired" in adminGet));
  assertEquals(adminGet.found, true);
  const adminList = await runtime.state.adminList({
    ...adminTarget,
    limit: 10,
    store: "drafts",
  });
  assertEquals(adminList.count, 1);
  let wrongDigest: unknown;
  try {
    await runtime.state.adminGet({
      ...adminTarget,
      contractDigest: `${deviceContract.CONTRACT_DIGEST}-wrong`,
      store: "preferences",
    });
  } catch (error) {
    wrongDigest = error;
  }
  assertInstanceOf(wrongDigest, ValidationError);
  assert(String(wrongDigest).includes("/contractDigest"));
  const deleted = await runtime.state.adminDelete({
    ...adminTarget,
    store: "preferences",
  });
  assertEquals(deleted.deleted, true);
  const missingAfterDelete = await reconnectedA.state.preferences.get()
    .orThrow();
  assert(!("migrationRequired" in missingAfterDelete));
  assertEquals(missingAfterDelete.found, false);

  await reconnectedA.connection.close();
  await deviceB.device.connection.close();
  await admin.connection.close();
}
