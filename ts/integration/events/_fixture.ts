import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import { Type } from "typebox";
import { integrationSlug } from "../_support/names.ts";

export function createEventsFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const eventSchemas = {
    EntityChanged: Type.Object({
      id: Type.String(),
      value: Type.String(),
      header: Type.Optional(Type.String()),
    }),
  } as const;
  const entityChangedSubject =
    `events.v1.Integration.Events.${slug}.Entity.Changed`;

  const serviceContract = defineServiceContract(
    { schemas: eventSchemas },
    (ref) => ({
      id: `trellis.integration.events-service.${slug}@v1`,
      apiId: `trellis.integration.events-service.${slug}@v1`,
      apiVersion: "1.0.0",
      displayName: `Trellis Integration Events Service (${slug})`,
      description: "Exercises generated event publish and subscribe surfaces.",
      capabilities: {
        publishRecords: {
          displayName: "Publish records",
          description: "Publish entity change records in the events fixture.",
        },
        readRecords: {
          displayName: "Read records",
          description:
            "Subscribe to entity change records in the events fixture.",
        },
      },
      events: {
        "Entity.Changed": {
          version: "v1",
          subject: entityChangedSubject,
          event: ref.schema("EntityChanged"),
          capabilities: {
            publish: ["publishRecords"],
            subscribe: ["readRecords"],
          },
        },
      },
    }),
  );

  const pubSubClientContract = defineAppContract(() => ({
    id: `trellis.integration.events-pubsub-client.${slug}@v1`,
    apiId: `trellis.integration.events-pubsub-client.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Events PubSub Client (${slug})`,
    description:
      "App/client participant with event publish and subscribe authority.",
    uses: [
      serviceContract.EntityChanged.publish,
      serviceContract.EntityChanged.subscribe,
    ],
  }));

  const subscribeOnlyClientContract = defineAppContract(() => ({
    id: `trellis.integration.events-subscribe-only-client.${slug}@v1`,
    apiId: `trellis.integration.events-subscribe-only-client.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Events Subscribe-Only Client (${slug})`,
    description: "App/client participant without event publish authority.",
    uses: [serviceContract.EntityChanged.subscribe],
  }));

  const publishOnlyClientContract = defineAppContract(() => ({
    id: `trellis.integration.events-publish-only-client.${slug}@v1`,
    apiId: `trellis.integration.events-publish-only-client.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Events Publish-Only Client (${slug})`,
    description: "App/client participant without event subscribe authority.",
    uses: [serviceContract.EntityChanged.publish],
  }));

  return {
    slug,
    serviceContract,
    pubSubClientContract,
    subscribeOnlyClientContract,
    publishOnlyClientContract,
    captureName: `events-fixture-capture-${slug}`,
    publisherName: `events-fixture-publisher-${slug}`,
    authorizedPublisherName: `events-fixture-authorized-publisher-${slug}`,
    subscribeOnlyName: `events-fixture-subscribe-only-${slug}`,
    publishOnlyName: `events-fixture-publish-only-${slug}`,
    sourceSubject: entityChangedSubject,
    publishedEntityId: `entity-${slug}`,
    deniedPublishEntityId: `entity-denied-${slug}`,
    deniedSubscribeEntityId: `entity-no-subscribe-${slug}`,
  };
}
