import {
  assertEquals,
  assertNotEquals,
  assertObjectMatch,
  assertThrows,
} from "@std/assert";
import { Type } from "typebox";

import { AuthCapabilitiesList } from "../sdk/_generated/auth/descriptors.ts";
import { API_DIGEST as AUTH_API_DIGEST } from "../sdk/_generated/auth/api.ts";
import {
  CONTRACT_EVENT_CONSUMERS_METADATA,
  defineAgentContract,
  defineAppContract,
  defineDeviceContract,
  defineServiceContract,
  jobs,
  kv,
  operationAccess,
  optional,
  state,
  store,
} from "./mod.ts";
import { actionRuntimeDescriptor, actionSource } from "./descriptors.ts";
import { canonicalizeJson, type JsonValue } from "./canonical.ts";
import { CONTRACT_RUNTIME } from "./contract_runtime.ts";
import {
  apiDigest,
  collectActionSources,
  nativeProtocolPresentation,
} from "./protocol_artifacts.ts";
import { resolveParticipantV1WasmSync } from "../auth/protocol_wasm.ts";

Deno.test("generated actions preserve canonical source artifact identity", () => {
  const compiled = nativeProtocolPresentation(defineAppContract(() => ({
    id: "trellis.test.auth-observer@v1",
    displayName: "Auth observer",
    description: "Checks generated dependency identity.",
    uses: [AuthCapabilitiesList],
  })));

  assertObjectMatch(compiled.participant, {
    uses: {
      required: {
        "trellis.auth@v1": {
          apiDigest: AUTH_API_DIGEST,
        },
      },
    },
  });
});

Deno.test("service contracts retain authoring event-consumer runtime metadata", () => {
  const contract = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.consumer@v1",
      displayName: "Consumer",
      description: "Checks event-consumer runtime metadata.",
      events: {
        Changed: { version: "v1", event: ref.schema("Empty") },
      },
      eventConsumers: {
        ingest: {
          self: ["Changed"],
          ackWaitMs: 1_000,
          maxDeliver: 2,
        },
      },
    }),
  );

  assertEquals(contract[CONTRACT_EVENT_CONSUMERS_METADATA], {
    ingest: { self: ["Changed"], ackWaitMs: 1_000, maxDeliver: 2 },
  });
});

Deno.test("operation access retains native observe, cancel, and control selections", () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.operation-provider@v1",
      displayName: "Operation provider",
      description: "Checks operation control selections.",
      capabilities: {
        operate: { displayName: "Operate", description: "Operate." },
      },
      operations: {
        Run: {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          cancel: true,
          signals: { update: { input: ref.schema("Empty") } },
          capabilities: {
            call: ["operate"],
            observe: ["operate"],
            cancel: ["operate"],
            control: ["operate"],
          },
          errors: [],
        },
      },
    }),
  );
  const consumer = defineAppContract(() => ({
    id: "trellis.test.operation-consumer@v1",
    displayName: "Operation consumer",
    description: "Uses operation control.",
    uses: [operationAccess(provider.Run, { cancel: true, control: true })],
  }));

  assertEquals(consumer.PARTICIPANT.uses, {
    required: {
      "trellis.test.operation-provider@v1": {
        api: provider.API.id,
        apiDigest: provider.API_DIGEST,
        operations: {
          invoke: ["Run"],
          observe: ["Run"],
          cancel: ["Run"],
          control: { Run: ["update"] },
        },
      },
    },
  });
});

Deno.test("inline actions preserve their provider API artifact", async () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.inline-provider@v1",
      displayName: "Inline provider",
      description: "Checks inline action source identity.",
      capabilities: {
        call: { displayName: "Call", description: "Call the provider." },
      },
      rpc: {
        "Inline.Call": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          capabilities: { call: ["call"] },
          errors: [],
        },
      },
    }),
  );
  const consumer = defineAppContract(() => ({
    id: "trellis.test.inline-consumer@v1",
    displayName: "Inline consumer",
    description: "Uses the inline provider.",
    uses: [provider.InlineCall],
  }));

  const provided = nativeProtocolPresentation(provider);
  const consumed = nativeProtocolPresentation(consumer);
  assertEquals(provider.CONTRACT_ID, "trellis.test.inline-provider@v1");
  assertEquals("CONTRACT" in provider, false);
  assertEquals(actionSource(provider.InlineCall), {
    api: provider.API,
    apiDigest: provider.API_DIGEST,
  });
  assertEquals(provider.API.format, "trellis.api.v1");
  assertEquals(provider.PARTICIPANT.format, "trellis.participant.v1");
  assertEquals(provider.PARTICIPANT.implements, {
    self: {
      api: provider.API.id,
      apiDigest: provider.API_DIGEST,
    },
  });
  const rpc = provider.API.rpc;
  if (rpc === null || typeof rpc !== "object" || Array.isArray(rpc)) {
    throw new Error("native API RPC section is missing");
  }
  const inlineCall = rpc["Inline.Call"];
  if (
    inlineCall === null || typeof inlineCall !== "object" ||
    Array.isArray(inlineCall)
  ) {
    throw new Error("native API RPC definition is missing");
  }
  assertEquals(inlineCall.subject, undefined);
  assertEquals(inlineCall.capabilities, undefined);
  assertEquals(provider.API.consent, {
    "trellis.test.inline-provider::call": {
      title: "Call",
      description: "Call the provider.",
      consequence: "",
    },
  });
  assertEquals(consumed.referencedApis, [provided.api]);
  assertEquals(
    actionRuntimeDescriptor(provider.InlineCall).callerCapabilities,
    [
      "trellis.test.inline-provider::call",
    ],
  );
});

Deno.test("capability permissions affect API identity, wording does not", () => {
  const createProvider = (displayName: string, capability: string) =>
    defineServiceContract(
      { schemas: { Empty: Type.Object({}) } },
      (ref) => ({
        id: "trellis.test.capability-provider@v1",
        displayName: "Capability provider",
        description: "Checks native capability identity.",
        capabilities: {
          [capability]: {
            displayName,
            description: "The capability wording.",
          },
          unused: {
            displayName: "Unused",
            description: "Unused capability wording.",
          },
        },
        rpc: {
          "Capability.Call": {
            version: "v1",
            input: ref.schema("Empty"),
            output: ref.schema("Empty"),
            capabilities: { call: [capability] },
            errors: [],
          },
        },
      }),
    );
  const original = createProvider("Call", "call");
  const renamed = createProvider("Renamed call", "call");
  const changed = createProvider("Call", "changed");

  assertEquals(original.API_DIGEST, renamed.API_DIGEST);
  assertNotEquals(original.API_DIGEST, changed.API_DIGEST);
  assertNotEquals(original.API.consent, renamed.API.consent);
  const consent = original.API.consent;
  if (
    consent === null || typeof consent !== "object" || Array.isArray(consent)
  ) {
    throw new Error("native API consent is missing");
  }
  const callConsent = consent["trellis.test.capability-provider::call"];
  if (
    callConsent === null || typeof callConsent !== "object" ||
    Array.isArray(callConsent)
  ) {
    throw new Error("native API capability consent is missing");
  }
  assertEquals(callConsent.description, "The capability wording.");
  const capabilities = original.API.capabilities;
  if (
    capabilities === null || typeof capabilities !== "object" ||
    Array.isArray(capabilities)
  ) {
    throw new Error("native API capabilities are missing");
  }
  assertEquals(
    capabilities["trellis.test.capability-provider::unused"],
    { allows: [] },
  );
});

Deno.test("capability references require declared local or platform/global names", () => {
  const define = (capability: string) =>
    defineServiceContract(
      { schemas: { Empty: Type.Object({}) } },
      (ref) => ({
        id: "trellis.test.capability-validation@v1",
        displayName: "Capability validation",
        description: "Checks capability reference validation.",
        capabilities: {
          local: { displayName: "Local", description: "Local capability." },
        },
        rpc: {
          Call: {
            version: "v1",
            input: ref.schema("Empty"),
            output: ref.schema("Empty"),
            capabilities: { call: [capability] },
            errors: [],
          },
        },
      }),
    );

  assertEquals(
    actionRuntimeDescriptor(define("local").Call).callerCapabilities,
    ["trellis.test.capability-validation::local"],
  );
  for (const capability of ["admin", "service", "external::qualified"]) {
    assertEquals(
      actionRuntimeDescriptor(define(capability).Call).callerCapabilities,
      [capability],
    );
  }
  assertThrows(() => define("typo.read"), Error, "undeclared local capability");
});

Deno.test("native presentation verifies all participant identities and evidence", () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.presentation-provider@v1",
      displayName: "Presentation provider",
      description: "Provides exact API evidence.",
      rpc: {
        Call: {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [],
        },
      },
    }),
  );
  const consumer = defineAppContract(() => ({
    id: "trellis.test.presentation-consumer@v1",
    displayName: "Presentation consumer",
    description: "Consumes exact API evidence.",
    uses: [provider.Call],
  }));
  const forged = (changes: Partial<typeof consumer>) => ({
    ...consumer,
    [CONTRACT_RUNTIME]: consumer[CONTRACT_RUNTIME],
    ...changes,
  });

  assertThrows(
    () => nativeProtocolPresentation(forged({ CONTRACT_DIGEST: "forged" })),
    Error,
    "participant digest",
  );
  assertThrows(
    () =>
      nativeProtocolPresentation(
        forged({ PARTICIPANT_NEEDS_DIGEST: "forged" }),
      ),
    Error,
    "needs digest",
  );
  const staleSelf = structuredClone(provider.PARTICIPANT);
  (staleSelf.implements as Record<string, Record<string, unknown>>).self
    .apiDigest = "forged";
  assertThrows(
    () => nativeProtocolPresentation({ ...provider, PARTICIPANT: staleSelf }),
    Error,
    "digest",
  );
  const runtime = consumer[CONTRACT_RUNTIME];
  assertThrows(
    () =>
      nativeProtocolPresentation(
        forged({ [CONTRACT_RUNTIME]: { ...runtime, actions: [] } }),
      ),
    Error,
    "required",
  );
});

Deno.test("action sources reject conflicting and forged API revisions", () => {
  const provider = defineServiceContract(
    { schemas: { Empty: Type.Object({}) } },
    (ref) => ({
      id: "trellis.test.revision-provider@v1",
      displayName: "Revision provider",
      description: "Checks exact action source evidence.",
      rpc: {
        Call: {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [],
        },
      },
    }),
  );
  const source = actionSource(provider.Call)!;
  const changedApi = {
    ...structuredClone(provider.API),
    schemas: {
      ...structuredClone(provider.API.schemas as Record<string, unknown>),
      Changed: { type: "string" },
    },
  };

  assertThrows(
    () =>
      collectActionSources([
        source,
        { api: changedApi, apiDigest: apiDigest(changedApi) },
      ]),
    Error,
    "Conflicting action source revisions",
  );
  assertThrows(
    () =>
      collectActionSources([
        { api: changedApi, apiDigest: source.apiDigest },
      ]),
    Error,
    "digest does not match",
  );
});

Deno.test("native authoring matches shared cross-language vectors", async () => {
  const fixture = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../conformance/native-authoring/vectors.json",
        import.meta.url,
      ),
    ),
  ) as {
    cases: Array<Record<string, JsonValue>>;
  };
  const dependency = defineServiceContract(
    { schemas: { Payload: Type.Object({ id: Type.String() }) } },
    (ref) => ({
      id: "conformance.dependency@v1",
      displayName: "Dependency",
      description: "Conformance dependency.",
      rpc: {
        "Dependency.Call": {
          version: "v1",
          input: ref.schema("Payload"),
          output: ref.schema("Payload"),
        },
      },
      events: {
        "Dependency.Changed": { version: "v1", event: ref.schema("Payload") },
      },
    }),
  );
  const optionalDependency = defineServiceContract(
    { schemas: { Payload: Type.Object({ id: Type.String() }) } },
    (ref) => ({
      id: "conformance.optional-dependency@v1",
      displayName: "Optional dependency",
      description: "Optional conformance dependency.",
      rpc: {
        "Optional.Call": {
          version: "v1",
          input: ref.schema("Payload"),
          output: ref.schema("Payload"),
        },
      },
    }),
  );
  const minimal = defineAppContract(() => ({
    id: "conformance.minimal-app@v1",
    displayName: "Minimal app",
    description: "Minimal native app.",
  }));
  const service = defineServiceContract(
    {
      schemas: {
        Payload: Type.Object({ id: Type.String() }),
        OldPayload: Type.Object({ id: Type.String() }),
      },
    },
    (ref) => ({
      id: "conformance.service@v1",
      displayName: "Conformance service",
      description: "Representative native service.",
      capabilities: {
        use: {
          displayName: "Use service",
          description: "Use representative surfaces.",
          consequence: "Runs work.",
        },
      },
      uses: [
        dependency.DependencyCall,
        dependency.DependencyChanged.subscribe,
        optional(optionalDependency.OptionalCall),
        state({
          settings: {
            kind: "value",
            schema: ref.schema("Payload"),
            stateVersion: "v2",
            acceptedVersions: { v1: ref.schema("OldPayload") },
          },
        }),
        jobs({
          work: {
            payload: ref.schema("Payload"),
            result: ref.schema("Payload"),
            keyConcurrency: {
              key: ["/id"],
              maxActive: 1,
              heartbeatIntervalMs: 1000,
              heartbeatTtlMs: 3000,
              stalePolicy: "fail-stale",
            },
            queue: { maxQueuedPerKey: 2, whenFull: "replace-oldest" },
          },
        }),
        kv({
          cache: {
            purpose: "Conformance cache",
            schema: ref.schema("Payload"),
          },
        }),
        store({ objects: { purpose: "Conformance objects" } }),
      ],
      rpc: {
        "Service.Call": {
          version: "v1",
          input: ref.schema("Payload"),
          output: ref.schema("Payload"),
          errors: [ref.error("UnexpectedError")],
          capabilities: { call: ["use"] },
        },
      },
      operations: {
        "Service.Run": {
          version: "v1",
          input: ref.schema("Payload"),
          progress: ref.schema("Payload"),
          update: ref.schema("Payload"),
          output: ref.schema("Payload"),
          cancel: true,
          signals: { continue: { input: ref.schema("Payload") } },
          capabilities: {
            call: ["use"],
            observe: ["use"],
            cancel: ["use"],
            control: ["use"],
          },
          transfer: {
            direction: "send",
            store: "objects",
            key: "/id",
            expiresInMs: 1000,
            maxBytes: 1024,
          },
        },
      },
      events: {
        "Service.Changed": {
          version: "v1",
          event: ref.schema("Payload"),
          params: ["/id"],
        },
      },
      feeds: {
        "Service.Live": {
          version: "v1",
          input: ref.schema("Payload"),
          event: ref.schema("Payload"),
        },
      },
      eventConsumers: {
        changes: {
          self: ["Service.Changed"],
          uses: { "conformance.dependency@v1": ["Dependency.Changed"] },
          replay: "all",
          ordering: "strict",
          ackWaitMs: 1000,
          maxDeliver: 2,
        },
      },
    }),
  );
  const device = defineDeviceContract(() => ({
    id: "conformance.device@v1",
    displayName: "Device",
    description: "Native device.",
    uses: [dependency.DependencyCall],
  }));
  const agent = defineAgentContract(() => ({
    id: "conformance.agent@v1",
    displayName: "Agent",
    description: "Native agent.",
    uses: [dependency.DependencyChanged.subscribe],
  }));
  const contracts = [minimal, ...Array(11).fill(service), device, agent];

  assertEquals(contracts.length, fixture.cases.length);
  for (const [index, contract] of contracts.entries()) {
    const presentation = nativeProtocolPresentation(contract);
    const apis = Object.fromEntries(
      [presentation.api, ...presentation.referencedApis].map((
        api,
      ) => [String(api.id), api]),
    );
    const resolved = resolveParticipantV1WasmSync({
      participant: presentation.participant,
      apis,
    });
    const actual = {
      name: fixture.cases[index]!.name,
      api: presentation.api,
      apiCanonicalJson: canonicalizeJson(presentation.api),
      apiDigest: resolved.apiDigests[String(presentation.api.id)],
      participant: presentation.participant,
      participantCanonicalJson: canonicalizeJson(presentation.participant),
      participantDigest: resolved.participantDigest,
      participantNeeds: resolved.participantNeeds,
      participantNeedsDigest: resolved.participantNeedsDigest,
      requiredGrants: resolved.requiredGrants,
      optionalGrants: resolved.optionalGrants,
    };
    assertEquals(actual, fixture.cases[index], String(actual.name));
  }
});

Deno.test("native participant resolution matches shared cross-language vectors", async () => {
  const fixture = JSON.parse(
    await Deno.readTextFile(
      new URL(
        "../../../../conformance/participant-resolution/vectors.json",
        import.meta.url,
      ),
    ),
  ) as {
    apis: Record<string, unknown>[];
    vectors: Array<{
      name: string;
      participant: Record<string, unknown>;
      participantErrorPath?: string;
      suppliedApis?: string[];
      valid: boolean;
      expectedNeeds?: {
        required: { grantSet: Record<string, JsonValue> };
        optional: { grantSet: Record<string, JsonValue> };
      } & Record<string, JsonValue>;
      expectedNeedsDigest?: string;
      expectedProposal?: Record<string, unknown>;
    }>;
  };
  const apis = Object.fromEntries(
    fixture.apis.map((api) => [String(api.id), api]),
  );
  const digests = resolveParticipantV1WasmSync({
    participant: {
      format: "trellis.participant.v1",
      id: "conformance-digest-reader",
      displayName: "Conformance digest reader",
      description: "Computes authoritative API digests for shared vectors.",
      kind: "app",
    },
    apis,
  }).apiDigests;
  const hydrateDigests = (value: unknown): unknown => {
    if (Array.isArray(value)) return value.map(hydrateDigests);
    if (!value || typeof value !== "object") return value;
    const hydrated = Object.fromEntries(
      Object.entries(value).map(([key, child]) => [key, hydrateDigests(child)]),
    );
    if (hydrated.apiDigest === "$actual" && typeof hydrated.api === "string") {
      hydrated.apiDigest = digests[hydrated.api];
    }
    return hydrated;
  };

  for (const vector of fixture.vectors) {
    if (!vector.valid || vector.participantErrorPath) continue;
    const participant = hydrateDigests(vector.participant);
    const suppliedApis = vector.suppliedApis
      ? Object.fromEntries(vector.suppliedApis.map((id) => [id, apis[id]]))
      : apis;
    const resolved = resolveParticipantV1WasmSync({
      participant,
      apis: suppliedApis,
    });

    if (vector.expectedNeeds) {
      assertEquals(
        resolved.participantNeeds,
        vector.expectedNeeds,
        `${vector.name} needs`,
      );
      assertEquals(
        resolved.requiredGrants,
        vector.expectedNeeds.required.grantSet,
        `${vector.name} required grants`,
      );
      assertEquals(
        resolved.optionalGrants,
        vector.expectedNeeds.optional.grantSet,
        `${vector.name} optional grants`,
      );
    }
    if (vector.expectedNeedsDigest) {
      assertEquals(
        resolved.participantNeedsDigest,
        vector.expectedNeedsDigest,
        `${vector.name} needs digest`,
      );
    }
    if (vector.expectedProposal) {
      assertEquals(
        resolved.authorityProposal,
        vector.expectedProposal,
        `${vector.name} authority proposal`,
      );
    }
  }
});
