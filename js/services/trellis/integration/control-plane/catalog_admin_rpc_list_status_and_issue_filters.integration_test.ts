import { assert, assertEquals } from "@std/assert";
import {
  type CallerRuntime,
  defineAppContract,
  defineServiceContract,
  Result,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import * as trellisCore from "@qlever-llc/trellis/sdk/core.ts";
import type {
  TrellisCatalogOutput,
  TrellisSurfaceStatusOutput,
} from "@qlever-llc/trellis/sdk/core.ts";
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
  "control-plane.catalog-admin-rpc-list-status-and-issue-filters" as const;

const schemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String(), servedBy: Type.String() }),
} as const;

const providerContractId = caseScopedContractId(
  "trellis.integration.control-plane.catalog-admin-provider",
  CASE_ID,
);
const consumerContractId = caseScopedContractId(
  "trellis.integration.control-plane.catalog-admin-consumer",
  CASE_ID,
);
const providerCapability = providerContractId.replace(/@v1$/, "") + "::ping";

const providerContract = defineServiceContract({ schemas }, (ref) => ({
  id: providerContractId,
  displayName: "Trellis Control-Plane Catalog Admin Provider",
  description:
    "Provides an RPC used by the catalog admin list/status/filter coverage.",
  capabilities: {
    ping: {
      displayName: "Call catalog admin ping",
      description: "Call the catalog admin provider RPC.",
    },
  },
  rpc: {
    "CatalogAdmin.ProviderPing": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.catalog-admin-provider",
        CASE_ID,
        "CatalogAdmin.ProviderPing",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      capabilities: { call: ["ping"] },
      errors: [],
    },
  },
}));

const consumerContract = defineServiceContract({ schemas }, (ref) => ({
  id: consumerContractId,
  displayName: "Trellis Control-Plane Catalog Admin Consumer",
  description:
    "Requires the provider RPC so catalog admin issue filters can inspect the relationship.",
  uses: [providerContract.CatalogAdminProviderPing],
  rpc: {
    "CatalogAdmin.ConsumerPing": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.catalog-admin-consumer",
        CASE_ID,
        "CatalogAdmin.ConsumerPing",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      errors: [],
    },
  },
}));

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.catalog-admin-client",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Catalog Admin Client",
  description: "Exercises generated catalog and admin list RPCs.",
  uses: [
    trellisCore.TrellisCatalog,
    trellisCore.TrellisContractGet,
    trellisCore.TrellisSurfaceStatus,
    trellisAuth.AuthDeploymentsList,
    providerContract.CatalogAdminProviderPing,
  ],
}));

const providerDeployment = caseScopedName(
  "catalog-admin-provider-deployment",
  CASE_ID,
);
const consumerDeployment = caseScopedName(
  "catalog-admin-consumer-deployment",
  CASE_ID,
);
const providerName = caseScopedName("catalog-admin-provider", CASE_ID);
const consumerName = caseScopedName("catalog-admin-consumer", CASE_ID);
const adminName = caseScopedName("catalog-admin-client", CASE_ID);

liveTrellisTest({
  name:
    "control-plane.catalog-admin-rpc-list-status-and-issue-filters lists catalog status and filtered issues",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = requireControlPlaneSqlite(runtime);
    const providerDigest = requireDigest(providerContract.CONTRACT_DIGEST);
    const consumerDigest = requireDigest(consumerContract.CONTRACT_DIGEST);

    await runtime.deployments.create({
      id: providerDeployment,
      mutableDev: false,
    });
    await runtime.deployments.create({
      id: consumerDeployment,
      mutableDev: false,
    });
    const providerKey = await runtime.services.createInstance({
      deployment: providerDeployment,
      name: providerName,
      contract: providerContract,
    });
    const consumerKey = await runtime.services.createInstance({
      deployment: consumerDeployment,
      name: consumerName,
      contract: consumerContract,
    });
    const admin = await runtime.connectClient({
      name: adminName,
      contract: adminContract,
    });
    const providerService = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: providerContract,
      name: providerName,
      sessionKeySeed: providerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
    const consumerService = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: consumerContract,
      name: consumerName,
      sessionKeySeed: consumerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();

    try {
      providerService.handleCatalogAdminProviderPing(({ input }) =>
        Result.ok({ message: input.message, servedBy: providerName })
      );

      const catalog = await runtime.waitFor(async () => {
        const current = await admin.trellisCatalog({}).orThrow();
        return hasContract(current, providerDigest) &&
            hasContract(current, consumerDigest)
          ? current
          : false;
      }, { timeoutMs: 15_000, intervalMs: 100 });
      assert(hasContract(catalog, providerDigest));
      assert(hasContract(catalog, consumerDigest));

      const serviceDeployments = await admin.authDeploymentsList({
        kind: "service",
        disabled: false,
        limit: 500,
      }).orThrow();
      assertEquals(
        serviceDeployments.entries.some((entry) =>
          entry.deploymentId === providerDeployment
        ),
        true,
      );
      assertEquals(
        serviceDeployments.entries.some((entry) =>
          entry.deploymentId === consumerDeployment
        ),
        true,
      );

      const status = await runtime.waitFor(async () => {
        const current = await providerStatus(admin);
        return current.state === "available" && current.runtime === "live"
          ? current
          : false;
      }, { timeoutMs: 15_000, intervalMs: 100 });
      assertEquals(status, {
        state: "available",
        liveImplementer: true,
        runtime: "live",
      });

      const consumerGet = await admin.trellisContractGet({
        digest: consumerDigest,
      }).orThrow();
      assertEquals(
        ((consumerGet.contract.uses as Record<string, unknown>)
          .required as Record<string, unknown>)[providerContractId],
        {
          contract: providerContractId,
          rpc: { call: ["CatalogAdmin.ProviderPing"] },
        },
      );

      await providerService.stop();
      await setProviderOfferActive(sqlite, providerDigest, false);
      const issue = await runtime.waitFor(async () => {
        const current = await admin.trellisCatalog({}).orThrow();
        return findIssue(current, consumerDigest) ?? false;
      }, { timeoutMs: 15_000, intervalMs: 100 });
      assertEquals(issue.kind, "invalid-active-contract-uses");
      assertEquals(issue.contractId, consumerContractId);
      assert(issue.deploymentIds.includes(consumerDeployment));
      assert(issue.message.includes(providerContractId));
    } finally {
      await admin.connection.close().catch(() => undefined);
      await consumerService.stop().catch(() => undefined);
      await providerService.stop().catch(() => undefined);
    }
  },
});

type ControlPlaneSqlite = NonNullable<
  LiveTrellisRuntime["controlPlane"]
>["sqlite"];

function requireControlPlaneSqlite(
  runtime: LiveTrellisRuntime,
): ControlPlaneSqlite {
  const sqlite = runtime.controlPlane?.sqlite;
  if (sqlite === undefined) {
    throw new Error("catalog admin RPC test requires control-plane SQLite");
  }
  return sqlite;
}

function requireDigest(digest: string | undefined): string {
  if (digest === undefined) throw new Error("contract digest missing");
  return digest;
}

function hasContract(catalog: TrellisCatalogOutput, digest: string): boolean {
  return catalog.catalog.contracts.some((contract) =>
    contract.digest === digest
  );
}

async function providerStatus(
  admin: CatalogAdminClient,
): Promise<TrellisSurfaceStatusOutput["status"]> {
  return (await admin.trellisSurfaceStatus({
    contractId: providerContractId,
    kind: "rpc",
    surface: "CatalogAdmin.ProviderPing",
    action: "call",
  }).orThrow()).status;
}

type CatalogAdminClient = CallerRuntime<typeof adminContract>;

async function setProviderOfferActive(
  sqlite: ControlPlaneSqlite,
  providerDigest: string,
  active: boolean,
): Promise<void> {
  const timestamp = active ? null : new Date(Date.now() - 60_000).toISOString();
  const result = await sqlite.execute(
    `UPDATE implementation_offers
      SET stale_at = ?, expires_at = ?
      WHERE deployment_kind = 'service'
        AND deployment_id = ?
        AND contract_digest = ?
        AND status = 'accepted'`,
    [timestamp, timestamp, providerDeployment, providerDigest],
  );
  assertEquals(result.rowsAffected, 1);
}

function findIssue(catalog: TrellisCatalogOutput, consumerDigest: string) {
  return catalog.catalog.issues?.find((issue) =>
    issue.kind === "invalid-active-contract-uses" &&
    issue.contractId === consumerContractId &&
    issue.digest === consumerDigest &&
    issue.deploymentIds.includes(consumerDeployment)
  );
}
