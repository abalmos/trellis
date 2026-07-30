import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import {
  aliasCaseScopedActions,
  aliasCaseScopedRuntime,
  caseScopedActions,
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
  integrationSlug,
} from "../_support/names.ts";

export type FeedFrame = {
  readonly topic: string;
  readonly message: string;
  readonly sequence: number;
};

type OptionalFeedClient = {
  readonly feed?: {
    readonly entity?: {
      readonly live?: (input: { readonly topic: string }) => {
        orThrow(): Promise<unknown>;
      };
    };
  };
};

export function createFeedsFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const feedSchemas = {
    FeedInput: Type.Object({ topic: Type.String() }),
    FeedFrame: Type.Object({
      topic: Type.String(),
      message: Type.String(),
      sequence: Type.Number(),
    }),
  } as const;

  const serviceContract = aliasCaseScopedActions(
    caseId,
    defineServiceContract(
      { schemas: feedSchemas },
      (ref) => ({
        id: caseScopedContractId("trellis.integration.feeds-service", caseId),
        displayName: `Trellis Integration Feeds Service (${slug})`,
        description: "Exercises generated feed subscribe and handler surfaces.",
        capabilities: {
          readFeeds: {
            displayName: "Read feeds",
            description: "Subscribe to entity feed updates.",
          },
        },
        feeds: caseScopedActions(caseId, {
          "Entity.Live": {
            version: "v1",
            subject: caseScopedSubject(
              "feeds.v1.Integration.Feeds",
              caseId,
              "Entity.Live",
            ),
            input: ref.schema("FeedInput"),
            event: ref.schema("FeedFrame"),
            capabilities: { subscribe: ["readFeeds"] },
          },
        }),
      }),
    ),
  );

  const clientContract = aliasCaseScopedActions(
    caseId,
    defineAppContract(() => ({
      id: caseScopedContractId("trellis.integration.feeds-client", caseId),
      displayName: `Trellis Integration Feeds Client (${slug})`,
      description: "App/client participant for the feeds integration fixture.",
      uses: [serviceContract.EntityLive],
    })),
  );

  const unauthorizedClientContract = defineAppContract(() => ({
    id: caseScopedContractId(
      "trellis.integration.feeds-unauthorized-client",
      caseId,
    ),
    displayName: `Trellis Integration Feeds Unauthorized Client (${slug})`,
    description: "App/client participant without feed subscribe authority.",
  }));

  async function connectService(runtime: LiveTrellisRuntime) {
    const serviceKey = await runtime.registerService({
      name: caseScopedName("feeds-fixture-service", caseId),
      contract: serviceContract,
    });
    return aliasCaseScopedRuntime(
      serviceContract,
      await TrellisService.connect({
        authorizationContextEphemeral: true,
        trellisUrl: runtime.trellisUrl,
        contract: serviceContract,
        name: caseScopedName("feeds-fixture-service", caseId),
        identity: serviceKey,
        telemetry: false,
        server: {},
      }).orThrow(),
    );
  }

  return {
    slug,
    serviceContract,
    clientContract,
    unauthorizedClientContract,
    serviceName: caseScopedName("feeds-fixture-service", caseId),
    clientName: caseScopedName("feeds-fixture-client", caseId),
    unauthorizedClientName: caseScopedName(
      "feeds-fixture-unauthorized",
      caseId,
    ),
    topic: caseScopedName("entity-feed", caseId),
    connectService,
  };
}

function isFeedFrame(value: unknown): value is FeedFrame {
  return typeof value === "object" && value !== null &&
    "topic" in value && typeof value.topic === "string" &&
    "message" in value && typeof value.message === "string" &&
    "sequence" in value && typeof value.sequence === "number";
}

export async function collectFeedFrames(
  stream: AsyncIterable<unknown>,
  count: number,
): Promise<FeedFrame[]> {
  const frames: FeedFrame[] = [];
  for await (const frame of stream) {
    if (!isFeedFrame(frame)) {
      throw new Error("feed emitted an unexpected frame shape");
    }
    frames.push(frame);
    if (frames.length === count) return frames;
  }
  throw new Error(`feed ended after ${frames.length} of ${count} frames`);
}

export async function withTimeout<T>(
  promise: Promise<T>,
  label: string,
): Promise<T> {
  let timeout: ReturnType<typeof setTimeout> | undefined;
  try {
    return await Promise.race([
      promise,
      new Promise<T>((_, reject) => {
        timeout = setTimeout(
          () => reject(new Error(`${label} timed out`)),
          10000,
        );
      }),
    ]);
  } finally {
    if (timeout !== undefined) clearTimeout(timeout);
  }
}

export function optionalFeedClient(
  client: OptionalFeedClient,
): OptionalFeedClient {
  return client;
}
