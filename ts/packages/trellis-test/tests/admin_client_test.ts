import { assertEquals, assertRejects } from "@std/assert";
import {
  type AdminDeploymentContext,
  createDeployment,
} from "../src/admin/deployment.ts";
import type {
  AdminRpc,
  TrellisTestAdminRpcMethod,
} from "../src/admin/methods.ts";
import {
  adminMethods,
  TrellisTestAdminAutomation,
} from "../src/admin_client.ts";

const expectedAdminMethods = [
  "authCapabilityGroupsPut",
  "authConnectionsList",
  "authPortalsGrantOverridesRemove",
  "authPortalsGrantOverridesPut",
  "authPortalsList",
  "authPortalsLoginSettingsUpdate",
  "authPortalsPut",
  "authPortalsRoutesPut",
  "authDevicesProvision",
  "stateAdminDelete",
  "stateAdminGet",
  "stateAdminList",
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
        orThrow: () => {
          called = property;
          return Promise.resolve({});
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

Deno.test("concurrent deployment creation shares failure and permits retry", async () => {
  const failure = new Error("deployment creation failed");
  let attempts = 0;
  const context: AdminDeploymentContext = {
    defaultDeployment: "test",
    reconciliationMs: 1,
    autoAccept: new Set(),
    createdDeployments: new Map(),
    deploymentIds: new Map(),
    authorityIds: new Map(),
    protocolApis: new Map(),
    rpc: <M extends TrellisTestAdminRpcMethod>(
      method: M,
      _input: AdminRpc[M]["input"],
    ): Promise<AdminRpc[M]["output"]> => {
      if (method !== "authDeploymentsCreate") {
        return Promise.reject(new Error(`unexpected admin RPC ${method}`));
      }
      attempts += 1;
      if (attempts === 1) return Promise.reject(failure);
      return Promise.resolve(
        {
          deployment: {
            kind: "service",
            deploymentId: "deployment-1",
            namespaces: [],
          },
        } as AdminRpc[M]["output"],
      );
    },
  };

  const first = createDeployment(context);
  const second = createDeployment(context);
  const firstError = await assertRejects(() => first, Error, failure.message);
  const secondError = await assertRejects(() => second, Error, failure.message);
  assertEquals(firstError, failure);
  assertEquals(secondError, failure);
  assertEquals(attempts, 1);
  await createDeployment(context);
  assertEquals(attempts, 2);
});
