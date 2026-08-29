import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { Type } from "typebox";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { integrationSlug } from "../_support/names.ts";

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

  const serviceContract = defineServiceContract(
    { schemas: feedSchemas },
    (ref) => ({
      id: `trellis.integration.feeds-service.${slug}@v1`,
      apiId: `trellis.integration.feeds-service.${slug}@v1`,
      displayName: `Trellis Integration Feeds Service (${slug})`,
      description: "Exercises generated feed subscribe and handler surfaces.",
      capabilities: {
        readFeeds: {
          displayName: "Read feeds",
          description: "Subscribe to entity feed updates.",
        },
      },
      feeds: {
        "Entity.Live": {
          version: "v1",
          subject: `feeds.v1.Integration.Feeds.${slug}.Entity.Live`,
          input: ref.schema("FeedInput"),
          event: ref.schema("FeedFrame"),
          capabilities: { subscribe: ["readFeeds"] },
        },
      },
    }),
  );

  const clientContract = defineAppContract(() => ({
    id: `trellis.integration.feeds-client.${slug}@v1`,
    apiId: `trellis.integration.feeds-client.${slug}@v1`,
    displayName: `Trellis Integration Feeds Client (${slug})`,
    description: "App/client participant for the feeds integration fixture.",
    uses: [serviceContract.EntityLive],
  }));

  const unauthorizedClientContract = defineAppContract(() => ({
    id: `trellis.integration.feeds-unauthorized-client.${slug}@v1`,
    apiId: `trellis.integration.feeds-unauthorized-client.${slug}@v1`,
    displayName: `Trellis Integration Feeds Unauthorized Client (${slug})`,
    description: "App/client participant without feed subscribe authority.",
  }));

  async function connectService(runtime: LiveTrellisRuntime) {
    const serviceKey = await runtime.registerService({
      name: `feeds-fixture-service-${slug}`,
      contract: serviceContract,
    });
    return await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: `feeds-fixture-service-${slug}`,
      identity: serviceKey,
      telemetry: false,
      runtime: {},
    }).orThrow();
  }

  return {
    slug,
    serviceContract,
    clientContract,
    unauthorizedClientContract,
    serviceName: `feeds-fixture-service-${slug}`,
    clientName: `feeds-fixture-client-${slug}`,
    unauthorizedClientName: `feeds-fixture-unauthorized-${slug}`,
    topic: `entity-feed-${slug}`,
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
