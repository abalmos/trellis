import { assert, assertEquals } from "@std/assert";
import { createAuth } from "@qlever-llc/trellis/auth";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { createServiceApprovalFixture } from "./_fixture.ts";

const CASE_ID =
  "service-approval.service-bootstrap-denies-missing-disabled-and-digest-drift" as const;
const fixture = createServiceApprovalFixture(CASE_ID);

liveTrellisTest({
  name:
    "service-approval.service-bootstrap-denies-missing-disabled-and-digest-drift denies invalid service bootstrap without mutating state",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = runtime.controlPlane?.sqlite;
    assert(sqlite, "live runtime must expose control-plane SQLite");

    await runtime.deployments.create({ id: fixture.deploymentId });
    const { seed, sessionKey } = await fixture.provisionServiceInstance(
      runtime,
    );
    const admin = await fixture.connectAdmin(runtime);
    await runtime.contracts.approve({
      deployment: fixture.deploymentId,
      contract: fixture.serviceContract,
      allowPlanClassifications: ["update", "migration"],
    });

    let state = await storedServiceState(admin, sessionKey);
    await admin.rpc.auth.deploymentsDisable({
      kind: "service",
      deploymentId: fixture.deploymentId,
    }).orThrow();
    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      reason: "service_deployment_disabled",
      status: 403,
      state,
    });
    await admin.rpc.auth.deploymentsEnable({
      kind: "service",
      deploymentId: fixture.deploymentId,
    }).orThrow();
    await assertServiceReconnects(runtime, seed);

    const instanceId = (await storedServiceState(admin, sessionKey)).instance
      .instanceId;
    await admin.rpc.auth.serviceInstancesDisable({ instanceId }).orThrow();
    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      reason: "service_disabled",
      status: 403,
      state,
    });
    await admin.rpc.auth.serviceInstancesEnable({ instanceId }).orThrow();
    await assertServiceReconnects(runtime, seed);

    await sqlite.execute("DELETE FROM contracts WHERE digest = ?", [
      fixture.serviceContract.CONTRACT_DIGEST,
    ]);
    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      reason: "manifest_required",
      status: 409,
      state,
    });
    await assertBootstrapReady(runtime, seed, fixture.serviceContract.CONTRACT);

    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      contract: driftedServiceContract(),
      reason: "presented_contract_digest_mismatch",
      status: 409,
      state,
    });
    await assertBootstrapReady(runtime, seed, fixture.serviceContract.CONTRACT);

    const materialized = await materializedAuthority(sqlite);
    await sqlite.execute(
      "UPDATE materialized_authority SET status = ?, reconciled_at = NULL WHERE deployment_id = ?",
      ["pending", fixture.deploymentId],
    );
    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      reason: "authority_reconciliation_pending",
      status: 202,
      state,
    });
    await restoreMaterializedAuthority(sqlite, materialized);
    await assertBootstrapReady(runtime, seed, fixture.serviceContract.CONTRACT);

    await sqlite.execute(
      "UPDATE materialized_authority SET status = ?, error = ? WHERE deployment_id = ?",
      ["failed", "forced test failure", fixture.deploymentId],
    );
    state = await storedServiceState(admin, sessionKey);
    await assertDeniedBootstrap(runtime, seed, {
      admin,
      reason: "authority_reconciliation_failed",
      status: 202,
      state,
    });
    await restoreMaterializedAuthority(sqlite, materialized);
    await assertServiceReconnects(runtime, seed);

    await admin.connection.close().catch(() => undefined);
  },
});

type AdminClient = Awaited<ReturnType<typeof fixture.connectAdmin>>;
type ControlPlaneSqlite = NonNullable<LiveTrellisRuntime["controlPlane"]>[
  "sqlite"
];

async function assertDeniedBootstrap(
  runtime: LiveTrellisRuntime,
  seed: string,
  args: {
    admin: AdminClient;
    reason: string;
    status: number;
    state: Awaited<ReturnType<typeof storedServiceState>>;
    contract?: unknown;
  },
): Promise<void> {
  const response = await fetchServiceBootstrap(runtime, seed, args.contract);
  const body = await response.json() as Record<string, unknown>;
  assertEquals(response.status, args.status);
  assertEquals(body.reason, args.reason);
  assertEquals(
    await storedServiceState(args.admin, args.state.instance.instanceKey),
    args.state,
  );
}

async function assertBootstrapReady(
  runtime: LiveTrellisRuntime,
  seed: string,
  contract: unknown,
): Promise<void> {
  const response = await fetchServiceBootstrap(runtime, seed, contract);
  const body = await response.json() as Record<string, unknown>;
  assertEquals(response.status, 200);
  assertEquals(body.status, "ready");
}

async function assertServiceReconnects(
  runtime: LiveTrellisRuntime,
  seed: string,
): Promise<void> {
  const service = await fixture.connectService(runtime, seed);
  await service.stop();
}

async function fetchServiceBootstrap(
  runtime: LiveTrellisRuntime,
  seed: string,
  contract?: unknown,
): Promise<Response> {
  const auth = await createAuth({ sessionKeySeed: seed });
  const iat = auth.currentIat();
  return await fetch(new URL("/bootstrap/service", runtime.trellisUrl), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({
      sessionKey: auth.sessionKey,
      contractId: fixture.serviceContract.CONTRACT.id,
      contractDigest: fixture.serviceContract.CONTRACT_DIGEST,
      ...(contract === undefined ? {} : { contract }),
      iat,
      sig: await auth.natsConnectSigForIat(
        iat,
        fixture.serviceContract.CONTRACT_DIGEST,
      ),
    }),
  });
}

async function storedServiceState(admin: AdminClient, instanceKey: string) {
  const deployments = await admin.rpc.auth.deploymentsList({
    kind: "service",
    limit: 500,
  }).orThrow();
  const deployment = deployments.entries.find((entry) =>
    entry.deploymentId === fixture.deploymentId
  );
  assert(deployment, "expected service deployment row");

  const instances = await admin.rpc.auth.serviceInstancesList({
    deploymentId: fixture.deploymentId,
    limit: 500,
  }).orThrow();
  const instance = instances.entries.find((entry) =>
    entry.instanceKey === instanceKey
  );
  assert(instance, "expected service instance row");
  return { deployment, instance };
}

function driftedServiceContract() {
  return {
    ...fixture.serviceContract.CONTRACT,
    capabilities: {
      ...fixture.serviceContract.CONTRACT.capabilities,
      drift: {
        displayName: "Digest drift",
        description: "Forces a different manifest digest.",
      },
    },
  };
}

async function materializedAuthority(sqlite: ControlPlaneSqlite) {
  const rows = await sqlite.query(
    "SELECT status, reconciled_at, error FROM materialized_authority WHERE deployment_id = ?",
    [fixture.deploymentId],
  );
  const row = rows[0];
  assert(row, "expected materialized authority row");
  assert(typeof row.status === "string", "expected materialized status");
  assert(
    typeof row.reconciled_at === "string" || row.reconciled_at === null,
    "expected materialized reconciled_at",
  );
  assert(
    typeof row.error === "string" || row.error === null,
    "expected materialized error",
  );
  return row as {
    status: string;
    reconciled_at: string | null;
    error: string | null;
  };
}

async function restoreMaterializedAuthority(
  sqlite: ControlPlaneSqlite,
  row: { status: string; reconciled_at: string | null; error: string | null },
): Promise<void> {
  await sqlite.execute(
    "UPDATE materialized_authority SET status = ?, reconciled_at = ?, error = ? WHERE deployment_id = ?",
    [row.status, row.reconciled_at, row.error, fixture.deploymentId],
  );
}
