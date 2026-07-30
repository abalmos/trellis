import { assertEquals, assertRejects } from "@std/assert";
import {
  adminMethods,
  TrellisTestAdminAutomation,
} from "../src/admin_client.ts";

const expectedAdminMethods = [
  "authDeploymentsCreate",
  "authDeploymentAuthorityPlan",
  "authDeploymentAuthorityAcceptUpdate",
  "authDeploymentAuthorityAcceptMigration",
  "authDeploymentAuthorityList",
  "authDeploymentAuthorityReconcile",
  "authDeploymentAuthorityGet",
  "authServiceInstancesProvision",
  "authDeploymentAuthorityPlansList",
  "authDeploymentAuthorityReject",
  "authSessionsRevoke",
] as const;

Deno.test("admin registry and local dispatch stay in parity", async () => {
  assertEquals(Object.keys(adminMethods), [...expectedAdminMethods]);

  for (const method of expectedAdminMethods) {
    let called: PropertyKey | undefined;
    const client = new Proxy({}, {
      get: (_target, property) => (_input: unknown) => ({
        orThrow: async () => {
          called = property;
          return {};
        },
      }),
    });

    await adminMethods[method].call(client as never, {});
    assertEquals(called, method);
  }
});

Deno.test("client auth completion is not an admin RPC", async () => {
  assertEquals(Object.hasOwn(adminMethods, "completeClientAuth"), false);

  const admin = new TrellisTestAdminAutomation({
    trellisUrl: "http://127.0.0.1",
    adminPassword: "test",
    defaultDeployment: "test",
    defaultMutableDev: true,
    reconciliationMs: 1,
    autoAccept: [],
    getBootstrapUrl: () => Promise.reject(new Error("not used")),
    bootstrapComplete: true,
    rpcProxy: { url: "http://127.0.0.1", token: "test" },
  });

  await assertRejects(
    () => admin.callAdminRpc("completeClientAuth", {}),
    Error,
    "unsupported Trellis test admin RPC completeClientAuth",
  );
});
