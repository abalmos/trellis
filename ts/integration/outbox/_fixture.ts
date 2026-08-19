import {
  defineAppContract,
  defineServiceContract,
  jobs,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import {
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
  integrationSlug,
} from "../_support/names.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";

export function createOutboxFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const serviceContract = defineServiceContract(
    {
      schemas: {
        RecordChanged: Type.Object({
          id: Type.String(),
          value: Type.String(),
        }),
        ProcessRecord: Type.Object({ id: Type.String() }),
        ProcessedRecord: Type.Object({ id: Type.String() }),
      },
    },
    (ref) => ({
      id: caseScopedContractId("trellis.integration.outbox-service", caseId),
      displayName: `Trellis Integration Outbox Service (${slug})`,
      description: "Exercises live caller-owned SQL outbox dispatch.",
      capabilities: {
        publishRecords: {
          displayName: "Publish records",
          description: "Publish committed record events.",
        },
        readRecords: {
          displayName: "Read records",
          description: "Subscribe to committed record events.",
        },
      },
      events: {
        "Record.Changed": {
          version: "v1",
          subject: caseScopedSubject(
            "events.v1.Integration.Outbox",
            caseId,
            "Record.Changed",
          ),
          event: ref.schema("RecordChanged"),
          capabilities: {
            publish: ["publishRecords"],
            subscribe: ["readRecords"],
          },
        },
      },
      uses: [jobs({
        processRecord: {
          payload: { schema: "ProcessRecord" },
          result: { schema: "ProcessedRecord" },
        },
      })],
    }),
  );
  const captureContract = defineAppContract(() => ({
    id: caseScopedContractId("trellis.integration.outbox-capture", caseId),
    displayName: `Trellis Integration Outbox Capture (${slug})`,
    description: "Captures committed SQL outbox events.",
    uses: [serviceContract.RecordChanged.subscribe],
  }));
  const serviceName = caseScopedName("outbox-service", caseId);

  return {
    serviceContract,
    captureContract,
    serviceName,
    captureName: caseScopedName("outbox-capture", caseId),
    async registerService(runtime: LiveTrellisRuntime) {
      return await runtime.registerService({
        name: serviceName,
        contract: serviceContract,
      });
    },
    async connectService(
      runtime: LiveTrellisRuntime,
      identity: Awaited<ReturnType<LiveTrellisRuntime["registerService"]>>,
    ) {
      return await TrellisService.connect({
        authorizationContextEphemeral: true,
        trellisUrl: runtime.trellisUrl,
        contract: serviceContract,
        name: serviceName,
        identity,
        telemetry: false,
        runtime: {},
      }).orThrow();
    },
  };
}
