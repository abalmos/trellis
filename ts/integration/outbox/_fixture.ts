import {
  defineAppContract,
  defineServiceContract,
  jobs,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import { integrationSlug } from "../_support/names.ts";
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
      id: `integration.outbox-service.${slug}@v1`,
      apiId: `integration.outbox-service.${slug}@v1`,
      apiVersion: "1.0.0",
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
          subject: `events.v1.Integration.Outbox.${slug}.Record.Changed`,
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
    id: `integration.outbox-capture.${slug}@v1`,
    apiId: `integration.outbox-capture.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Outbox Capture (${slug})`,
    description: "Captures committed SQL outbox events.",
    uses: [serviceContract.RecordChanged.subscribe],
  }));
  const serviceName = `outbox-service-${slug}`;

  return {
    serviceContract,
    captureContract,
    serviceName,
    captureName: `outbox-capture-${slug}`,
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
