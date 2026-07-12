import { assertEquals } from "@std/assert";
import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
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
  "control-plane.authority-plan-migration-replaces-desired-state" as const;
const serviceContractId = caseScopedContractId(
  "trellis.integration.control-plane.authority-replace-service",
  CASE_ID,
);

const baseSchemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String() }),
} as const;
const replacementSchemas = {
  PingInput: Type.Object({ count: Type.Integer() }),
  PingOutput: Type.Object({ count: Type.Integer() }),
} as const;

const baseContract = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
  id: serviceContractId,
  displayName: "Trellis Control-Plane Authority Replace Service",
  description: "Base contract with legacy desired-state entries.",
  capabilities: {
    legacy: {
      displayName: "Legacy capability",
      description: "Capability that must disappear after migration.",
    },
  },
  rpc: {
    "AuthorityReplace.Legacy": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.authority-replace",
        CASE_ID,
        "AuthorityReplace.Legacy",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      capabilities: { call: ["legacy"] },
      errors: [],
    },
  },
}));

const replacementContract = defineServiceContract(
  { schemas: replacementSchemas },
  (ref) => ({
    id: serviceContractId,
    displayName: "Trellis Control-Plane Authority Replace Service",
    description: "Migration contract with replacement desired-state entries.",
    capabilities: {
      replacement: {
        displayName: "Replacement capability",
        description: "Capability that should replace legacy desired state.",
      },
    },
    rpc: {
      "AuthorityReplace.Next": {
        version: "v1",
        subject: caseScopedSubject(
          "rpc.v1.integration.control-plane.authority-replace",
          CASE_ID,
          "AuthorityReplace.Next",
        ),
        input: ref.schema("PingInput"),
        output: ref.schema("PingOutput"),
        capabilities: { call: ["replacement"] },
        errors: [],
      },
    },
  }),
);

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.authority-replace-admin",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Authority Replace Admin",
  description: "Reads deployment authority after migration.",
  uses: [trellisAuth.AuthDeploymentAuthorityGet],
}));

const deployment = caseScopedName("authority-replace", CASE_ID);
const baseServiceName = caseScopedName("authority-replace-base", CASE_ID);
const replacementServiceName = caseScopedName(
  "authority-replace-next",
  CASE_ID,
);
const adminName = caseScopedName("authority-replace-admin", CASE_ID);

type AuthorityPlanEntry = {
  readonly planId: string;
  readonly classification: "migration" | "update";
  readonly proposal: { readonly contractDigest: string };
};

liveTrellisTest({
  name:
    "control-plane.authority-plan-migration-replaces-desired-state replaces desired state on accepted migration",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.deployments.create({ id: deployment, mutableDev: false });
    const admin = await runtime.connectClient({
      name: adminName,
      contract: adminContract,
    });
    const baseKey = await runtime.services.createInstance({
      deployment,
      name: baseServiceName,
      contract: baseContract,
    });
    let baseService: { stop(): Promise<void> } | undefined =
      await TrellisService.connect({
        trellisUrl: runtime.trellisUrl,
        contract: baseContract,
        name: baseServiceName,
        sessionKeySeed: baseKey.seed,
        telemetry: false,
        server: { log: false },
      }).orThrow();
    let replacementService: { stop(): Promise<void> } | undefined;

    try {
      const before = await admin.authDeploymentAuthorityGet({
        deploymentId: deployment,
      }).orThrow();
      assertEquals(
        before.authority.desiredState.surfaces.some((surface) =>
          surface.name === "AuthorityReplace.Legacy"
        ),
        true,
      );
      await baseService.stop();
      baseService = undefined;

      const replacementKey = await runtime.services.provisionInstanceOnly({
        deployment,
      });
      const connectPromise = TrellisService.connect({
        trellisUrl: runtime.trellisUrl,
        contract: replacementContract,
        name: replacementServiceName,
        sessionKeySeed: replacementKey.seed,
        telemetry: false,
        server: { log: false },
      }).orThrow();
      connectPromise.catch(() => undefined);

      const plan = await waitForPendingMigrationPlan(runtime, {
        deployment,
        digest: replacementContract.CONTRACT_DIGEST,
      });
      await requireAuthority(runtime).acceptMigration({
        planId: plan.planId,
        acknowledgement: "Accepted by authority replacement integration test.",
      });
      await runtime.deployments.waitReady(deployment);
      replacementService = await connectPromise;

      const after = await admin.authDeploymentAuthorityGet({
        deploymentId: deployment,
      }).orThrow();
      assertEquals(
        after.authority.desiredState.surfaces.some((surface) =>
          surface.name === "AuthorityReplace.Legacy"
        ),
        false,
      );
      assertEquals(
        after.authority.desiredState.surfaces.some((surface) =>
          surface.name === "AuthorityReplace.Next"
        ),
        true,
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await replacementService?.stop().catch(() => undefined);
      await baseService?.stop().catch(() => undefined);
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
