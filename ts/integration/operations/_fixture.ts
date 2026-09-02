import {
  defineAppContract,
  defineServiceContract,
  operationAccess,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { integrationSlug } from "../_support/names.ts";

export function createOperationsFixture(
  caseId: string,
  options: {
    readonly cancelable?: boolean;
    readonly signals?: boolean;
    readonly distinctControlCapabilities?: boolean;
    readonly clientControlsOperation?: boolean;
    readonly statusOperation?: boolean;
  } = {},
) {
  const slug = integrationSlug(caseId);
  const cancelCapability = options.distinctControlCapabilities
    ? "processCancel"
    : "process";
  const controlCapability = options.distinctControlCapabilities
    ? "processControl"
    : "process";
  const clientControlsOperation = options.clientControlsOperation ??
    options.signals === true;
  const operationSchemas = {
    OperationInput: Type.Object({ message: Type.String() }),
    OperationSignalInput: Type.Object({ suffix: Type.String() }),
    OperationProgress: Type.Object({
      message: Type.String(),
      step: Type.Number(),
    }),
    OperationUpdate: Type.Object({
      message: Type.String(),
      step: Type.Integer(),
    }),
    OperationOutput: Type.Object({
      message: Type.String(),
      done: Type.Boolean(),
    }),
    StatusInput: Type.Object({ message: Type.String() }),
    StatusProgress: Type.Object({ stage: Type.String() }),
    StatusOutput: Type.Object({ status: Type.String() }),
  } as const;

  const serviceContract = defineServiceContract(
    { schemas: operationSchemas },
    (ref) => ({
      id: `integration.operations-service.${slug}@v1`,
      apiId: `integration.operations-service.${slug}@v1`,
      apiVersion: "1.0.0",
      displayName: `Trellis Integration Operations Service (${slug})`,
      description: "Exercises generated operation start and watch surfaces.",
      capabilities: {
        process: {
          displayName: "Process entities",
          description: "Start and observe entity processing operations.",
        },
        ...(options.distinctControlCapabilities
          ? {
            processCancel: {
              displayName: "Cancel entity processing",
              description: "Cancel entity processing operations.",
            },
            processControl: {
              displayName: "Control entity processing",
              description: "Signal entity processing operations.",
            },
          }
          : {}),
      },
      operations: {
        "Entity.Process": {
          version: "v1",
          subject:
            `operations.v1.Integration.Operations.${slug}.Entity.Process`,
          input: ref.schema("OperationInput"),
          progress: ref.schema("OperationProgress"),
          update: ref.schema("OperationUpdate"),
          output: ref.schema("OperationOutput"),
          errors: [ref.error("UnexpectedError")],
          capabilities: {
            call: ["process"],
            observe: ["process"],
            ...(options.cancelable ? { cancel: [cancelCapability] } : {}),
            ...(options.signals ? { control: [controlCapability] } : {}),
          },
          cancel: options.cancelable === true,
          ...(options.signals
            ? {
              signals: {
                updateMessage: { input: ref.schema("OperationSignalInput") },
                appendMessage: { input: ref.schema("OperationSignalInput") },
              },
            }
            : {}),
        },
        ...(options.statusOperation === true
          ? {
            "Entity.Status": {
              version: "v1",
              subject:
                `operations.v1.Integration.Operations.${slug}.Entity.Status`,
              input: ref.schema("StatusInput"),
              progress: ref.schema("StatusProgress"),
              output: ref.schema("StatusOutput"),
              errors: [ref.error("UnexpectedError")],
              capabilities: {
                call: ["process"],
                observe: ["process"],
              },
            },
          }
          : {}),
      },
    }),
  );

  const clientContract = defineAppContract(() => ({
    id: `integration.operations-client.${slug}@v1`,
    apiId: `integration.operations-client.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Operations Client (${slug})`,
    description:
      "App/client participant for the operations integration fixture.",
    uses: [
      operationAccess(serviceContract.EntityProcess, {
        cancel: options.cancelable === true,
        control: clientControlsOperation,
      }),
      ...(options.statusOperation === true
        ? [serviceContract.EntityStatus]
        : []),
    ],
  }));

  const unauthorizedClientContract = defineAppContract(() => ({
    id: `integration.operations-unauthorized-client.${slug}@v1`,
    apiId: `integration.operations-unauthorized-client.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Unauthorized Operations Client (${slug})`,
    description:
      "App/client without operation call authority for Entity.Process.",
  }));

  const serviceName = `operations-fixture-service-${slug}`;

  async function connectService(runtime: LiveTrellisRuntime) {
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
    });
    return await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      identity: serviceKey,
      telemetry: false,
      runtime: {},
    }).orThrow();
  }

  return {
    slug,
    processOperationName: "Entity.Process",
    statusOperationName: "Entity.Status",
    serviceContract,
    clientContract,
    unauthorizedClientContract,
    serviceName,
    otherServiceName: `operations-fixture-other-service-${slug}`,
    clientName: `operations-fixture-client-${slug}`,
    unauthorizedClientName: `operations-fixture-unauthorized-client-${slug}`,
    message: "operation",
    connectService,
    aliasRuntime: <R extends object>(runtime: R) => runtime,
  };
}
