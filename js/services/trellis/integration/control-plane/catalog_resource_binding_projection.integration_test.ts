import { assertEquals, assertExists } from "@std/assert";
import { defineServiceContract, jobs, kv, store } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import {
  caseScopedContractId,
  caseScopedName,
} from "@qlever-llc/trellis-test/integration";
import {
  type LiveTrellisRuntime,
  liveTrellisTest,
  runtimeScopeForCase,
} from "../_support/runtime.ts";

const CASE_ID = "control-plane.catalog-resource-binding-projection" as const;
const serviceContractId = caseScopedContractId(
  "trellis.integration.control-plane.binding-projection-service",
  CASE_ID,
);
const schemas = {
  Record: Type.Object({ message: Type.String() }),
} as const;

const resourceContract = defineServiceContract({ schemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Binding Projection Service",
  description:
    "Projects KV, store, and jobs declarations into typed runtime handles.",
  uses: [
    kv({
      records: {
        purpose: "Binding projection KV bucket.",
        schema: ref.schema("Record"),
        required: true,
        history: 1,
        ttlMs: 0,
      },
    }),
    store({
      blobs: {
        purpose: "Binding projection object store.",
        required: true,
        ttlMs: 0,
        maxObjectBytes: 1048576,
        maxTotalBytes: 4194304,
      },
    }),
    jobs({
      syncRecords: {
        payload: ref.schema("Record"),
      },
    }),
  ],
}));

const removedResourceContract = defineServiceContract({ schemas }, () => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Binding Projection Service",
  description: "Replacement contract with resources removed.",
}));

const deployment = caseScopedName("binding-projection", CASE_ID);
const serviceName = caseScopedName("binding-projection-service", CASE_ID);

type AuthorityPlanEntry = {
  readonly planId: string;
  readonly classification: "migration" | "update";
  readonly proposal: { readonly contractDigest: string };
};

liveTrellisTest({
  name:
    "control-plane.catalog-resource-binding-projection projects and removes service resource bindings",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.deployments.create({ id: deployment, mutableDev: false });
    const serviceKey = await runtime.services.createInstance({
      deployment,
      name: serviceName,
      contract: resourceContract,
    });
    const connectedResourceService = await connectResourceService(
      runtime.trellisUrl,
      serviceKey.seed,
    );
    let resourceService: { stop(): Promise<void> } | undefined =
      connectedResourceService;
    let removedService:
      | Awaited<ReturnType<typeof connectRemovedService>>
      | undefined;

    try {
      assertExists(connectedResourceService.kv.records);
      assertExists(connectedResourceService.store.blobs);
      assertExists(connectedResourceService.jobs.syncRecords);

      await resourceService.stop();
      resourceService = undefined;

      const removedKey = await runtime.services.provisionInstanceOnly({
        deployment,
      });
      const connectPromise = connectRemovedService(
        runtime.trellisUrl,
        removedKey.seed,
      );
      connectPromise.catch(() => undefined);

      const plan = await waitForPendingMigrationPlan(runtime, {
        deployment,
        digest: removedResourceContract.CONTRACT_DIGEST,
      });
      await requireAuthority(runtime).acceptMigration({
        planId: plan.planId,
        acknowledgement: "Accepted by binding projection integration test.",
      });
      await runtime.deployments.waitReady(deployment);

      removedService = await connectPromise;
      assertEquals(Object.keys(removedService.kv), []);
      assertEquals(Object.keys(removedService.store), []);
      assertEquals(Object.keys(removedService.jobs), []);
    } finally {
      await removedService?.stop().catch(() => undefined);
      await resourceService?.stop().catch(() => undefined);
    }
  },
});

async function connectResourceService(
  trellisUrl: string,
  sessionKeySeed: string,
) {
  return await TrellisService.connect({
    trellisUrl,
    contract: resourceContract,
    name: serviceName,
    sessionKeySeed,
    telemetry: false,
    server: { log: false },
  }).orThrow();
}

async function connectRemovedService(
  trellisUrl: string,
  sessionKeySeed: string,
) {
  return await TrellisService.connect({
    trellisUrl,
    contract: removedResourceContract,
    name: serviceName,
    sessionKeySeed,
    telemetry: false,
    server: { log: false },
  }).orThrow();
}

function requireAuthority(runtime: LiveTrellisRuntime) {
  if (runtime.authority === undefined) {
    throw new Error("authority helper is required");
  }
  return runtime.authority;
}

async function waitForPendingMigrationPlan(
  runtime: LiveTrellisRuntime,
  args: { readonly deployment: string; readonly digest: string | undefined },
): Promise<AuthorityPlanEntry> {
  if (args.digest === undefined) throw new Error("contract digest missing");
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
