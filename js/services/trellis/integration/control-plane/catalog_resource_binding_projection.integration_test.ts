import { assert, assertEquals } from "@std/assert";
import {
  defineAppContract,
  defineServiceContract,
  Result,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import {
  sdk as trellisCore,
  TrellisBindingsGetResponseSchema,
} from "@qlever-llc/trellis/sdk/core.ts";
import type { TrellisBindingsGetOutput } from "@qlever-llc/trellis/sdk/core.ts";
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

const CASE_ID = "control-plane.catalog-resource-binding-projection" as const;
const serviceContractId = caseScopedContractId(
  "trellis.integration.control-plane.binding-projection-service",
  CASE_ID,
);
const bindingSubject = caseScopedSubject(
  "rpc.v1.integration.control-plane.binding-projection",
  CASE_ID,
  "BindingProjection.Get",
);

const schemas = {
  Empty: Type.Object({}),
  Record: Type.Object({ message: Type.String() }),
  Bindings: TrellisBindingsGetResponseSchema,
} as const;

const resourceContract = defineServiceContract({ schemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Binding Projection Service",
  description:
    "Projects KV, store, and jobs bindings through Trellis.Bindings.Get.",
  uses: {
    required: {
      core: trellisCore.use({ rpc: { call: ["Trellis.Bindings.Get"] } }),
    },
  },
  resources: {
    kv: {
      records: {
        purpose: "Binding projection KV bucket.",
        schema: ref.schema("Record"),
        required: true,
        history: 1,
        ttlMs: 0,
      },
    },
    store: {
      blobs: {
        purpose: "Binding projection object store.",
        required: true,
        ttlMs: 0,
        maxObjectBytes: 1048576,
        maxTotalBytes: 4194304,
      },
    },
  },
  jobs: {
    syncRecords: {
      payload: ref.schema("Record"),
    },
  },
  rpc: {
    "BindingProjection.Get": {
      version: "v1",
      subject: bindingSubject,
      input: ref.schema("Empty"),
      output: ref.schema("Bindings"),
      errors: [],
    },
  },
}));

const removedResourceContract = defineServiceContract({ schemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Binding Projection Service",
  description: "Replacement contract with resources removed.",
  uses: {
    required: {
      core: trellisCore.use({ rpc: { call: ["Trellis.Bindings.Get"] } }),
    },
  },
  rpc: {
    "BindingProjection.Get": {
      version: "v1",
      subject: bindingSubject,
      input: ref.schema("Empty"),
      output: ref.schema("Bindings"),
      errors: [],
    },
  },
}));

const appContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.binding-projection-client",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Binding Projection Client",
  description: "Calls the binding projection service RPC.",
  uses: {
    required: {
      service: resourceContract.use({
        rpc: { call: ["BindingProjection.Get"] },
      }),
    },
  },
}));

const deployment = caseScopedName("binding-projection", CASE_ID);
const serviceName = caseScopedName("binding-projection-service", CASE_ID);
const clientName = caseScopedName("binding-projection-client", CASE_ID);

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
    const client = await runtime.connectClient({
      name: clientName,
      contract: appContract,
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
      await connectedResourceService.handle.rpc.bindingProjection.get(
        async ({ client }) => {
          const binding = await client.request("Trellis.Bindings.Get", {})
            .orThrow();
          return Result.ok(binding);
        },
      );
      const projected = await client.rpc.bindingProjection.get({}).orThrow();
      assertBindingProjection(
        projected,
        resourceContract.CONTRACT_DIGEST,
        true,
        deployment,
      );

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
      await removedService.handle.rpc.bindingProjection.get(
        async ({ client }) => {
          const binding = await client.request("Trellis.Bindings.Get", {})
            .orThrow();
          return Result.ok(binding);
        },
      );
      const removedProjection = await client.rpc.bindingProjection.get({})
        .orThrow();
      assertBindingProjection(
        removedProjection,
        removedResourceContract.CONTRACT_DIGEST,
        false,
      );
    } finally {
      await client.connection.close().catch(() => undefined);
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

function assertBindingProjection(
  output: TrellisBindingsGetOutput,
  digest: string | undefined,
  hasResources: boolean,
  expectedServiceName?: string,
): void {
  if (digest === undefined) throw new Error("contract digest missing");
  assert(output.binding);
  assertEquals(output.binding.contractId, serviceContractId);
  assertEquals(output.binding.digest, digest);
  if (!hasResources) {
    assertEquals(output.binding.resources, {});
    return;
  }
  const resources = output.binding.resources;
  assert(isRecord(resources.kv?.records));
  assertEquals(typeof resources.kv.records.bucket, "string");
  assert(isRecord(resources.store?.blobs));
  assertEquals(typeof resources.store.blobs.name, "string");
  assert(isRecord(resources.jobs));
  assertEquals(resources.jobs.serviceName, expectedServiceName);
  assertEquals(typeof resources.jobs.namespace, "string");
  assert(resources.jobs.namespace !== expectedServiceName);
  assert(isRecord(resources.jobs.queues?.syncRecords));
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
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
