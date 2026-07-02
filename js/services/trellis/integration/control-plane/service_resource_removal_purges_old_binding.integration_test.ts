import { assertEquals } from "@std/assert";
import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { sdk as trellisAuth } from "@qlever-llc/trellis/sdk/auth";
import { Type } from "typebox";
import {
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
} from "@qlever-llc/trellis-test/integration";
import {
  type LiveTrellisRuntime,
  liveTrellisTest,
  runtimeScopeForCase,
} from "../_support/runtime.ts";

const CASE_ID =
  "control-plane.service-resource-removal-purges-old-binding" as const;
const serviceContractId = caseScopedContractId(
  "trellis.integration.control-plane.resource-removal-service",
  CASE_ID,
);
const resourceSubject = caseScopedSubject(
  "rpc.v1.integration.control-plane.resource-removal",
  CASE_ID,
  "ResourceRemoval.Ping",
);

const schemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String() }),
  Record: Type.Object({ message: Type.String() }),
} as const;

const resourceContract = defineServiceContract({ schemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Resource Removal Service",
  description: "Starts with a service KV resource binding.",
  resources: {
    kv: {
      records: {
        purpose: "Resource removed by the replacement contract.",
        schema: ref.schema("Record"),
        required: true,
        history: 1,
        ttlMs: 0,
      },
    },
  },
  rpc: {
    "ResourceRemoval.Ping": {
      version: "v1",
      subject: resourceSubject,
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      errors: [],
    },
  },
}));

const removedResourceContract = defineServiceContract({ schemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Resource Removal Service",
  description: "Replacement contract with the service KV resource removed.",
  rpc: {
    "ResourceRemoval.Ping": {
      version: "v1",
      subject: resourceSubject,
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      errors: [],
    },
  },
}));

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.resource-removal-admin",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Resource Removal Admin",
  description: "Reads deployment authority after resource removal.",
  uses: {
    required: {
      auth: trellisAuth.use({
        rpc: { call: ["Auth.DeploymentAuthority.Get"] },
      }),
    },
  },
}));

const deployment = caseScopedName("resource-removal", CASE_ID);
const serviceName = caseScopedName("resource-removal-service", CASE_ID);
const adminName = caseScopedName("resource-removal-admin", CASE_ID);

type AuthorityPlanEntry = {
  readonly planId: string;
  readonly classification: "migration" | "update";
  readonly proposal: { readonly contractDigest: string };
};

liveTrellisTest({
  name:
    "control-plane.service-resource-removal-purges-old-binding removes obsolete service resource binding",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.deployments.create({ id: deployment, mutableDev: false });
    const admin = await runtime.connectClient({
      name: adminName,
      contract: adminContract,
    });
    const resourceKey = await runtime.services.createInstance({
      deployment,
      name: serviceName,
      contract: resourceContract,
    });
    const connectedResourceService = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: resourceContract,
      name: serviceName,
      sessionKeySeed: resourceKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
    let resourceService: { stop(): Promise<void> } | undefined =
      connectedResourceService;
    let removedService: { stop(): Promise<void> } | undefined;

    try {
      assertEquals(typeof connectedResourceService.kv.records, "object");
      await resourceService.stop();
      resourceService = undefined;

      const removedKey = await runtime.services.provisionInstanceOnly({
        deployment,
      });
      const connectPromise = TrellisService.connect({
        trellisUrl: runtime.trellisUrl,
        contract: removedResourceContract,
        name: serviceName,
        sessionKeySeed: removedKey.seed,
        telemetry: false,
        server: { log: false },
      }).orThrow();
      connectPromise.catch(() => undefined);

      const plan = await waitForPendingMigrationPlan(runtime, {
        deployment,
        digest: removedResourceContract.CONTRACT_DIGEST,
      });
      await requireAuthority(runtime).acceptMigration({
        planId: plan.planId,
        acknowledgement: "Accepted by resource-removal integration test.",
      });
      await runtime.deployments.waitReady(deployment);

      const connectedRemovedService = await connectPromise;
      removedService = connectedRemovedService;
      assertEquals(Object.hasOwn(connectedRemovedService.kv, "records"), false);
      const authority = await admin.rpc.auth.deploymentAuthorityGet({
        deploymentId: deployment,
      }).orThrow();
      assertEquals(
        authority.authority.desiredState.resources.some((resource) =>
          resource.alias === "records"
        ),
        false,
      );
      assertEquals(
        authority.materializedAuthority?.resourceBindings.some((binding) =>
          binding.alias === "records"
        ),
        false,
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await removedService?.stop().catch(() => undefined);
      await resourceService?.stop().catch(() => undefined);
    }
  },
});

function requireAuthority(runtime: LiveTrellisRuntime) {
  if (runtime.authority === undefined) {
    throw new Error("authority helper is required");
  }
  return runtime.authority;
}

async function waitForPendingMigrationPlan(
  runtime: LiveTrellisRuntime,
  args: { readonly deployment: string; readonly digest: string },
): Promise<AuthorityPlanEntry> {
  return await runtime.waitFor(async () => {
    const plans = await requireAuthority(runtime).plans.list({
      deploymentId: args.deployment,
      state: "pending",
      classification: "migration",
      limit: 20,
    });
    return plans.entries.find((entry): entry is AuthorityPlanEntry =>
      isAuthorityPlanEntry(entry) &&
      entry.proposal.contractDigest === args.digest
    ) ?? false;
  }, { timeoutMs: 15_000, intervalMs: 100 });
}

function isAuthorityPlanEntry(value: unknown): value is AuthorityPlanEntry {
  return typeof value === "object" && value !== null &&
    "planId" in value && typeof value.planId === "string" &&
    "classification" in value &&
    (value.classification === "migration" ||
      value.classification === "update") &&
    "proposal" in value && typeof value.proposal === "object" &&
    value.proposal !== null && "contractDigest" in value.proposal &&
    typeof value.proposal.contractDigest === "string";
}
