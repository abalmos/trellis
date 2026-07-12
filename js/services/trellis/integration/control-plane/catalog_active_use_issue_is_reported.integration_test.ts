import { assert, assertEquals } from "@std/assert";
import {
  defineAppContract,
  defineServiceContract,
  Result,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import * as trellisCore from "@qlever-llc/trellis/sdk/core.ts";
import type { TrellisCatalogOutput } from "@qlever-llc/trellis/sdk/core.ts";
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

const CASE_ID = "control-plane.catalog-active-use-issue-is-reported" as const;

const schemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String() }),
} as const;

const providerContractId = caseScopedContractId(
  "trellis.integration.control-plane.catalog-active-use-provider",
  CASE_ID,
);
const consumerContractId = caseScopedContractId(
  "trellis.integration.control-plane.catalog-active-use-consumer",
  CASE_ID,
);

const providerContract = defineServiceContract({ schemas }, (ref) => ({
  id: providerContractId,
  displayName: "Trellis Control-Plane Catalog Active Use Provider",
  description:
    "Provides the RPC required by the active-use issue integration consumer.",
  rpc: {
    "CatalogActiveUse.ProviderPing": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.catalog-active-use-provider",
        CASE_ID,
        "CatalogActiveUse.ProviderPing",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      errors: [],
    },
  },
}));

const consumerContract = defineServiceContract({ schemas }, (ref) => ({
  id: consumerContractId,
  displayName: "Trellis Control-Plane Catalog Active Use Consumer",
  description:
    "Requires the provider RPC so catalog active-use validation can report missing active dependencies.",
  uses: [providerContract.CatalogActiveUseProviderPing],
  rpc: {
    "CatalogActiveUse.ConsumerPing": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.catalog-active-use-consumer",
        CASE_ID,
        "CatalogActiveUse.ConsumerPing",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      errors: [],
    },
  },
}));

const catalogAdminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.catalog-active-use-admin",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Catalog Active Use Admin",
  description: "Reads the public catalog issue projection.",
  uses: [trellisCore.TrellisCatalog],
}));

const providerDeployment = caseScopedName(
  "catalog-active-use-provider-deployment",
  CASE_ID,
);
const consumerDeployment = caseScopedName(
  "catalog-active-use-consumer-deployment",
  CASE_ID,
);
const providerName = caseScopedName("catalog-active-use-provider", CASE_ID);
const consumerName = caseScopedName("catalog-active-use-consumer", CASE_ID);
const adminName = caseScopedName("catalog-active-use-admin", CASE_ID);

liveTrellisTest({
  name:
    "control-plane.catalog-active-use-issue-is-reported reports invalid active uses when a required provider becomes inactive",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = requireControlPlaneSqlite(runtime);
    const providerDigest = requireDigest(
      providerContract.CONTRACT_DIGEST,
      "provider",
    );
    const consumerDigest = requireDigest(
      consumerContract.CONTRACT_DIGEST,
      "consumer",
    );

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
    const catalogAdmin = await runtime.connectClient({
      name: adminName,
      contract: catalogAdminContract,
    });

    const connectedProviderService = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: providerContract,
      name: providerName,
      sessionKeySeed: providerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
    let providerService: { stop(): Promise<void> } | undefined =
      connectedProviderService;
    const consumerService = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: consumerContract,
      name: consumerName,
      sessionKeySeed: consumerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();

    try {
      connectedProviderService.handleCatalogActiveUseProviderPing((
        { input },
      ) => Result.ok({ message: input.message }));
      consumerService.handleCatalogActiveUseConsumerPing(({ input }) =>
        Result.ok({ message: input.message })
      );

      await runtime.waitFor(async () => {
        const catalog = await catalogAdmin.trellisCatalog({}).orThrow();
        return catalog.catalog.contracts.some((contract) =>
          contract.digest === providerDigest
        ) && catalog.catalog.contracts.some((contract) =>
          contract.digest === consumerDigest
        );
      }, { timeoutMs: 15_000, intervalMs: 100 });

      await connectedProviderService.stop();
      providerService = undefined;
      await setProviderOfferActive(sqlite, providerDigest, false);

      const issue = await runtime.waitFor(async () => {
        const catalog = await catalogAdmin.trellisCatalog({}).orThrow();
        return findActiveUseIssue(catalog, consumerDigest) ?? false;
      }, { timeoutMs: 15_000, intervalMs: 100 });

      assertEquals(issue.contractId, consumerContractId);
      assertEquals(issue.digest, consumerDigest);
      assert(issue.deploymentIds.includes(consumerDeployment));
      assert(
        issue.message.includes(providerContractId) &&
          issue.message.includes("inactive contract"),
        "expected catalog issue message to mention the inactive provider contract",
      );

      await setProviderOfferActive(sqlite, providerDigest, true);
      const restoredCatalog = await runtime.waitFor(async () => {
        const catalog = await catalogAdmin.trellisCatalog({}).orThrow();
        return findActiveUseIssue(catalog, consumerDigest) === undefined
          ? catalog
          : false;
      }, { timeoutMs: 15_000, intervalMs: 100 });
      assertEquals(
        findActiveUseIssue(restoredCatalog, consumerDigest),
        undefined,
      );
    } finally {
      await catalogAdmin.connection.close().catch(() => undefined);
      await consumerService.stop().catch(() => undefined);
      await providerService?.stop().catch(() => undefined);
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
    throw new Error(
      "control-plane catalog active-use issue test requires direct runtime SQLite access",
    );
  }
  return sqlite;
}

function requireDigest(digest: string | undefined, label: string): string {
  if (digest === undefined) throw new Error(`${label} contract digest missing`);
  return digest;
}

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

function findActiveUseIssue(
  catalog: TrellisCatalogOutput,
  consumerDigest: string,
) {
  return catalog.catalog.issues?.find((issue) =>
    issue.kind === "invalid-active-contract-uses" &&
    issue.contractId === consumerContractId &&
    issue.digest === consumerDigest
  );
}
