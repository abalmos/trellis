import { assert, assertEquals } from "@std/assert";
import {
  defineAppContract,
  defineServiceContract,
  Result,
} from "@qlever-llc/trellis";
import {
  ACTION_METADATA,
  actionSource,
} from "../../../../../trellis/contract_support/mod.ts";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import {
  integrationSlug,
  runtimeScopeForCase,
  trellisIntegrationTest,
} from "@qlever-llc/trellis-test/integration";
import { externalServiceRepoRuntime } from "../trellis.integration.ts";

const CASE_ID = "external.rpc-smoke" as const;
const slug = integrationSlug(CASE_ID);
const message = `hello-${slug}`;

const schemas = {
  PingInput: Type.Object({ message: Type.String() }),
  PingOutput: Type.Object({ message: Type.String(), reply: Type.String() }),
} as const;

const serviceContract = defineServiceContract(
  { schemas },
  (ref) => ({
    id: "external.fixture.rpc-service" + "@v1",
    apiId: "external.fixture.rpc-service" + "@v1",
    apiVersion: "1.0.0",
    displayName: `External Fixture RPC Service (${slug})`,
    description:
      "Out-of-tree style service contract used by trellis-test smoke coverage.",
    capabilities: {
      ping: {
        displayName: "Ping service",
        description: "Call the fixture service ping RPC.",
      },
    },
    rpc: {
      "Echo.Ping": {
        version: "v1",
        input: ref.schema("PingInput"),
        output: ref.schema("PingOutput"),
        capabilities: { call: ["ping"] },
        errors: [],
      },
    },
  }),
);

const clientContract = defineAppContract(() => ({
  id: "external.fixture.rpc-client" + "@v1",
  apiId: "external.fixture.rpc-client" + "@v1",
  apiVersion: "1.0.0",
  displayName: `External Fixture RPC Client (${slug})`,
  description:
    "Out-of-tree style app contract used by trellis-test smoke coverage.",
  uses: [serviceContract.EchoPing],
}));

const serviceName = "external-rpc-service";
const clientName = "external-rpc-client";

trellisIntegrationTest({
  caseId: CASE_ID,
  name: "external.rpc-smoke calls service RPC through generic runner",
  scope: runtimeScopeForCase(CASE_ID),
  runtime: externalServiceRepoRuntime,
  async fn(runtime) {
    const uses = clientContract.PARTICIPANT.uses as {
      required?: Record<string, { rpc?: { call?: string[] } }>;
    };
    assert(
      uses.required?.[serviceContract.CONTRACT_ID]?.rpc?.call?.includes(
        "Echo.Ping",
      ),
    );
    assertEquals(
      actionSource(serviceContract.EchoPing)?.api.id,
      serviceContract.API.id,
    );
    assertEquals(
      actionSource(serviceContract.EchoPing)?.apiDigest,
      serviceContract.API_DIGEST,
    );
    assertEquals(
      serviceContract.EchoPing[ACTION_METADATA].descriptor.permission,
      {
        apiId: serviceContract.API.id,
        apiVersion: "v1",
        surfaceKind: "rpc",
        surfaceName: "Echo.Ping",
        action: "call",
      },
    );
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
    });
    const service = await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      identity: serviceKey,
      telemetry: false,
      runtime: {},
    }).orThrow();

    try {
      await service.handleEchoPing(({ input }) =>
        Result.ok({ message: input.message, reply: `pong:${input.message}` })
      );

      const client = await runtime.connectClient({
        name: clientName,
        contract: clientContract,
      });
      const result = await client.echoPing({ message }).orThrow();

      assertEquals(result, { message, reply: `pong:${message}` });
    } finally {
      await service.stop();
    }
  },
});
