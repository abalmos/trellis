import { assert, assertEquals, assertInstanceOf } from "@std/assert";
import {
  type CallerRuntime,
  defineAppContract,
  ValidationError,
} from "@qlever-llc/trellis";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import { isErr } from "@qlever-llc/result";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { caseScopedContractId, caseScopedName } from "../_support/names.ts";
import { createAuthorityPlanFixture } from "./_fixture.ts";

const CASE_ID =
  "authority-plan.acceptance-rejects-wrong-classification-expired-and-version-mismatch" as const;
const fixture = createAuthorityPlanFixture(CASE_ID);

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.authority-plan.acceptance-admin",
    CASE_ID,
  ),
  displayName: `Authority Plan Acceptance Admin (${fixture.slug})`,
  description: "Exercises invalid deployment authority plan acceptance paths.",
  uses: [
    trellisAuth.AuthDeploymentAuthorityAcceptMigration,
    trellisAuth.AuthDeploymentAuthorityAcceptUpdate,
    trellisAuth.AuthDeploymentAuthorityGet,
  ],
}));

liveTrellisTest({
  name:
    "authority-plan.acceptance-rejects-wrong-classification-expired-and-version-mismatch rejects invalid acceptances without mutation",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const sqlite = runtime.controlPlane?.sqlite;
    assert(sqlite, "live runtime must expose control-plane SQLite");
    await runtime.deployments.create({
      id: fixture.strictDeployment,
      mutableDev: false,
    });
    const baseKey = await runtime.services.createInstance({
      deployment: fixture.strictDeployment,
      name: fixture.baseServiceName,
      contract: fixture.baseContract,
    });
    const baseService = await fixture.connectService({
      runtime,
      contract: fixture.baseContract,
      name: fixture.baseServiceName,
      seed: baseKey.seed,
    });
    let replacementService:
      | Awaited<
        ReturnType<
          typeof fixture.connectService<
            typeof fixture.incompatibleSchemaContract
          >
        >
      >
      | undefined;
    let additiveService:
      | Awaited<
        ReturnType<
          typeof fixture.connectService<
            typeof fixture.compatibleAdditiveContract
          >
        >
      >
      | undefined;
    const admin = await runtime.connectClient({
      name: caseScopedName("authority-plan-acceptance-admin", CASE_ID),
      contract: adminContract,
    });

    try {
      const before = await authoritySnapshot(admin);
      const replacementKey = await runtime.services.provisionInstanceOnly({
        deployment: fixture.strictDeployment,
      });
      const replacementConnect = fixture.connectServicePending({
        runtime,
        contract: fixture.incompatibleSchemaContract,
        name: fixture.replacementServiceName,
        seed: replacementKey.seed,
      }).then((connected) => {
        replacementService = connected;
        return connected;
      });
      replacementConnect.catch(() => undefined);
      const migration = await fixture.waitForPendingPlan(runtime, {
        deploymentId: fixture.strictDeployment,
        classification: "migration",
        contractDigest: fixture.incompatibleSchemaContract.CONTRACT_DIGEST,
      });

      await assertValidationError(
        admin.authDeploymentAuthorityAcceptUpdate({
          planId: migration.planId,
        }),
      );
      await assertValidationError(
        // @ts-expect-error Intentionally exercise runtime validation of a malformed request.
        admin.authDeploymentAuthorityAcceptMigration({
          planId: migration.planId,
        }),
      );
      await assertValidationError(
        admin.authDeploymentAuthorityAcceptMigration({
          planId: migration.planId,
          acknowledgement: "Accepted by invalid acceptance test.",
          expectedDesiredVersion: "stale-version",
        }),
      );
      await assertAuthorityUnchanged(admin, before);
      assertEquals(
        (await fixture.waitForPendingPlan(runtime, {
          deploymentId: fixture.strictDeployment,
          classification: "migration",
          contractDigest: fixture.incompatibleSchemaContract.CONTRACT_DIGEST,
        })).planId,
        migration.planId,
      );

      const rejected = await fixture.rejectPlan(runtime, migration);
      await assertValidationError(
        admin.authDeploymentAuthorityAcceptMigration({
          planId: migration.planId,
          acknowledgement: "Accepted after rejection.",
        }),
      );
      await assertAuthorityUnchanged(admin, before);
      assertEquals(rejected.state, "rejected");
      await fixture.expectPromisePending(
        replacementConnect,
        "rejected migration replacement connected",
      );

      const additiveKey = await runtime.services.provisionInstanceOnly({
        deployment: fixture.strictDeployment,
      });
      const additiveConnect = fixture.connectServicePending({
        runtime,
        contract: fixture.compatibleAdditiveContract,
        name: fixture.additiveServiceName,
        seed: additiveKey.seed,
      }).then((connected) => {
        additiveService = connected;
        return connected;
      });
      additiveConnect.catch(() => undefined);
      const update = await fixture.waitForPendingPlan(runtime, {
        deploymentId: fixture.strictDeployment,
        classification: "update",
        contractDigest: fixture.compatibleAdditiveContract.CONTRACT_DIGEST,
      });
      await sqlite.execute(
        "UPDATE deployment_authority_plans SET expires_at = ? WHERE plan_id = ?",
        ["2020-01-01T00:00:00.000Z", update.planId],
      );
      await assertValidationError(
        admin.authDeploymentAuthorityAcceptUpdate({
          planId: update.planId,
        }),
      );
      await assertAuthorityUnchanged(admin, before);
      assertEquals(
        (await fixture.waitForPendingPlan(runtime, {
          deploymentId: fixture.strictDeployment,
          classification: "update",
          contractDigest: fixture.compatibleAdditiveContract.CONTRACT_DIGEST,
        })).planId,
        update.planId,
      );
      await fixture.expectPromisePending(
        additiveConnect,
        "expired update service connected",
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await additiveService?.stop();
      await replacementService?.stop();
      await baseService.stop();
    }
  },
});

type AuthorityAdmin = CallerRuntime<typeof adminContract>;

type AuthoritySnapshot = {
  readonly version: string;
  readonly desiredState: unknown;
};

async function authoritySnapshot(
  admin: AuthorityAdmin,
): Promise<AuthoritySnapshot> {
  const current = await admin.authDeploymentAuthorityGet({
    deploymentId: fixture.strictDeployment,
  }).orThrow();
  return {
    version: current.authority.version,
    desiredState: current.authority.desiredState,
  };
}

async function assertAuthorityUnchanged(
  admin: AuthorityAdmin,
  expected: AuthoritySnapshot,
): Promise<void> {
  assertEquals(await authoritySnapshot(admin), expected);
}

async function assertValidationError(
  result: { take(): Promise<unknown> },
): Promise<void> {
  const value = await result.take();
  assert(isErr(value));
  assertInstanceOf(value.error, ValidationError);
}
