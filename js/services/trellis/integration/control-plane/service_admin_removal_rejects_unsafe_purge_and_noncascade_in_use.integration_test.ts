import { assert, assertEquals, assertInstanceOf } from "@std/assert";
import {
  defineAppContract,
  defineServiceContract,
  isErr,
  Result,
  TrellisClient,
  ValidationError,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { sdk as trellisAuth } from "@qlever-llc/trellis/sdk/auth";
import { Type } from "typebox";
import {
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
} from "@qlever-llc/trellis-test/integration";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";

const CASE_ID =
  "control-plane.service-admin-removal-rejects-unsafe-purge-and-noncascade-in-use" as const;
const deploymentId = caseScopedName("service-admin-removal", CASE_ID);
const serviceName = caseScopedName("service-admin-removal-service", CASE_ID);
const clientName = caseScopedName("service-admin-removal-client", CASE_ID);

const schemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String() }),
} as const;

const serviceContract = defineServiceContract({ schemas }, (ref) => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.service-admin-removal-service",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Service Admin Removal Probe",
  description:
    "Verifies rejected service deployment removal leaves a live service usable.",
  capabilities: {
    ping: {
      displayName: "Call removal probe ping",
      description: "Call the removal rejection probe RPC.",
    },
  },
  rpc: {
    "RemovalReject.Ping": {
      version: "v1",
      subject: caseScopedSubject(
        "rpc.v1.integration.control-plane.service-admin-removal",
        CASE_ID,
        "RemovalReject.Ping",
      ),
      input: ref.schema("PingInput"),
      output: ref.schema("PingOutput"),
      capabilities: { call: ["ping"] },
      errors: [],
    },
  },
}));

const clientContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.service-admin-removal-client",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Service Admin Removal Client",
  description: "Calls the service after rejected deployment removal attempts.",
  uses: {
    required: {
      removalService: serviceContract.use({
        rpc: { call: ["RemovalReject.Ping"] },
      }),
    },
  },
}));

const adminContract = defineAppContract(() => ({
  id: caseScopedContractId(
    "trellis.integration.control-plane.service-admin-removal-admin",
    CASE_ID,
  ),
  displayName: "Trellis Control-Plane Service Admin Removal Admin",
  description:
    "Exercises rejected Auth.Deployments.Remove paths through live Trellis.",
  uses: {
    required: {
      auth: trellisAuth.use({
        rpc: {
          call: [
            "Auth.DeploymentAuthority.Get",
            "Auth.Deployments.List",
            "Auth.Deployments.Remove",
            "Auth.ServiceInstances.List",
            "Auth.Sessions.List",
          ],
        },
      }),
    },
  },
}));

liveTrellisTest({
  name:
    "control-plane.service-admin-removal-rejects-unsafe-purge-and-noncascade-in-use rejects unsafe removes without changing live service records",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
      deployment: deploymentId,
    });
    const clientKey = await runtime.registerClient({
      name: clientName,
      contract: clientContract,
    });
    const clientAuth = runtime.clientAuth(clientKey);

    const service = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      sessionKeySeed: serviceKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
    service.handle.rpc.removalReject.ping(({ input }) =>
      Result.ok({ message: input.message })
    );
    const client = await TrellisClient.connect({
      trellisUrl: runtime.trellisUrl,
      name: clientName,
      contract: clientContract,
      auth: clientAuth.auth,
      onAuthRequired: clientAuth.onAuthRequired,
    }).orThrow();
    const admin = await runtime.connectClient({
      name: caseScopedName("service-admin-removal-admin", CASE_ID),
      contract: adminContract,
    });

    try {
      await assertRemoveValidationError(admin, {
        kind: "service",
        deploymentId,
        cascade: true,
        purgeResources: true,
      });
      await assertRemoveValidationError(admin, {
        kind: "service",
        deploymentId,
        purgeUnusedContracts: true,
      });
      await assertRemoveValidationError(admin, {
        kind: "service",
        deploymentId,
      });

      await assertRecordsStillListed(
        admin,
        serviceKey.sessionKey,
        clientKey.sessionKey,
      );
      assertEquals(
        await client.rpc.removalReject.ping({ message: "after" }).orThrow(),
        { message: "after" },
      );
    } finally {
      await admin.connection.close().catch(() => undefined);
      await client.connection.close().catch(() => undefined);
      await service.stop().catch(() => undefined);
    }
  },
});

async function assertRemoveValidationError(
  admin: RemovalAdmin,
  input: Record<string, unknown>,
): Promise<void> {
  const method: string = "Auth.Deployments.Remove";
  const value = await admin.request(method, input).take();
  assert(isErr(value));
  assertInstanceOf(value.error, ValidationError);
}

async function assertRecordsStillListed(
  admin: RemovalAdmin,
  serviceSessionKey: string,
  clientSessionKey: string,
): Promise<void> {
  const deployments = await admin.rpc.auth.deploymentsList({
    kind: "service",
    limit: 500,
  }).orThrow();
  assertEquals(
    deployments.entries.some((deployment) =>
      deployment.deploymentId === deploymentId && deployment.disabled === false
    ),
    true,
  );

  const instances = await admin.rpc.auth.serviceInstancesList({
    deploymentId,
    limit: 500,
  }).orThrow();
  assertEquals(
    instances.entries.some((instance) =>
      instance.instanceKey === serviceSessionKey && instance.disabled === false
    ),
    true,
  );

  const authority = await admin.rpc.auth.deploymentAuthorityGet({
    deploymentId,
  })
    .orThrow();
  assertEquals(authority.authority.deploymentId, deploymentId);
  assertEquals(authority.authority.disabled, false);

  const sessions = await admin.rpc.auth.sessionsList({ limit: 500 }).orThrow();
  assertSessionListed(sessions.entries, serviceSessionKey, "service");
  assertSessionListed(sessions.entries, clientSessionKey, "app");
}

function assertSessionListed(
  entries: Array<{ sessionKey: string; participantKind: string }>,
  sessionKey: string,
  participantKind: string,
): void {
  assertEquals(
    entries.some((entry) =>
      entry.sessionKey === sessionKey &&
      entry.participantKind === participantKind
    ),
    true,
  );
}

type RemovalAdmin = {
  request(method: string, input: unknown): { take(): Promise<unknown> };
  rpc: {
    auth: {
      deploymentsList(input: { kind: "service"; limit: number }): {
        orThrow(): Promise<{
          entries: Array<{ deploymentId: string; disabled: boolean }>;
        }>;
      };
      serviceInstancesList(input: { deploymentId: string; limit: number }): {
        orThrow(): Promise<{
          entries: Array<{ instanceKey: string; disabled: boolean }>;
        }>;
      };
      deploymentAuthorityGet(input: { deploymentId: string }): {
        orThrow(): Promise<{
          authority: { deploymentId: string; disabled: boolean };
        }>;
      };
      sessionsList(input: { limit: number }): {
        orThrow(): Promise<{
          entries: Array<{ sessionKey: string; participantKind: string }>;
        }>;
      };
    };
  };
};
