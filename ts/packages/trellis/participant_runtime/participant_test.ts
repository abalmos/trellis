import { assertEquals, assertStrictEquals } from "@std/assert";
import { apiDescriptor, codecs, participantDescriptor } from "../generated.ts";
import { getParticipantRuntime, participantEvidence } from "./participant.ts";

Deno.test("generated participant descriptors project into the runtime", () => {
  const api = apiDescriptor({
    identity: "example.orders@v1",
    actions: {
      "rpc:Get": {
        kind: "rpc",
        descriptorName: "rpc:Get",
        input: codecs.string,
        output: codecs.u64,
        errors: [],
        download: false,
        pagination: undefined,
      },
      "event:Changed": {
        kind: "event",
        descriptorName: "event:Changed",
        payload: codecs.bytes,
        parameters: [["orderId"]],
      },
    },
  });
  const packageEvidence = {
    rootPackage: "example",
    rootDigest: "package-digest",
    packages: [{
      name: "example",
      version: "1.0.0",
      digest: "package-digest",
      source: "package example@1.0.0;",
    }],
  };
  const participant = participantDescriptor({
    kind: "app",
    identity: "example.Console",
    path: "Console",
    implements: [],
    uses: [{
      api,
      actions: [
        { descriptorName: "rpc:Get", direction: "call" },
        { descriptorName: "event:Changed", direction: "subscribe" },
      ],
      optionalCapabilities: [],
    }],
    resources: {},
    packageEvidence,
  });

  const runtime = getParticipantRuntime(participant);
  assertStrictEquals(runtime.usedApi.rpc.Get.input, codecs.string);
  assertEquals(runtime.usedApi.rpc.Get.subject, "rpc.v1.Get");
  assertEquals(runtime.usedApi.rpc.Get.permission, {
    apiId: "example.orders",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Get",
    action: "call",
  });
  assertEquals(
    runtime.usedApi.events.Changed.subject,
    "events.v1.Changed.{/orderId}",
  );
  assertEquals(runtime.actions.map((action) => action.connectedName), [
    "get",
    "onChanged",
  ]);

  const evidence = participantEvidence(participant);
  assertStrictEquals(evidence.packageEvidence, packageEvidence);
  assertEquals(evidence.participantPath, "Console");
  assertEquals(evidence.packageDigest, "package-digest");
});
