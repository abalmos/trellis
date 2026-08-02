import {
  assert,
  assertEquals,
  assertNotEquals,
  assertThrows,
} from "@std/assert";
import { Type } from "typebox";

import {
  type ContractUses,
  defineAppContract,
  defineDeviceContract,
  defineError,
  defineServiceContract,
  digestContractManifest,
  globalCapabilityName,
  jobs,
  kv,
  normalizeContractManifest,
  optional,
  parseContractManifest,
  state,
  store,
} from "./mod.ts";
import { unwrapSchema } from "./runtime.ts";
import { getContractRuntime } from "./contract_runtime.ts";
import { HealthQuery } from "../sdk/health.ts";

const EmptySchema = Type.Object({});
const StringSchema = Type.Object({ value: Type.String() });

const baseSchemas = {
  Empty: EmptySchema,
  StringValue: StringSchema,
} as const;

function requiredUses(
  uses: ContractUses | undefined,
): ContractUses["required"] | undefined {
  return uses?.required;
}

function schemaRef<
  TSchemas extends Record<string, unknown>,
  const TName extends keyof TSchemas & string,
>(
  schema: TName,
) {
  return { schema } as const;
}

if (false) {
  defineServiceContract(
    {
      // @ts-expect-error overload should reject registry-side capabilities
      schemas: baseSchemas,
      // @ts-expect-error capabilities belong in the contract body, not the registry
      capabilities: {},
    },
    () => ({
      id: "capability.registry-compile-error@v1",
      displayName: "Capability registry compile error",
      description:
        "This contract intentionally fails type checking without the directive.",
    }),
  );

  defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "capability.compile-error@v1",
    displayName: "Capability compile error",
    description:
      "This contract intentionally fails type checking without the directive.",
    rpc: {
      "Capability.Missing": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: {
          // @ts-expect-error local capability references must be declared first
          call: [ref.capability("missing.local")],
        },
      },
    },
  }));
}

Deno.test("kind-specific helpers preserve emitted manifest shape and digest", async () => {
  const auth = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.auth@v1",
      displayName: "Trellis Auth",
      description: "Expose auth RPCs and events for source emission tests.",
      capabilities: {
        "events.auth": {
          displayName: "Auth events",
          description: "Publish and subscribe to auth events.",
        },
      },
      exports: {
        schemas: ["StringValue"],
      },
      rpc: {
        "Auth.Sessions.Me": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          capabilities: { call: [] },
          errors: ["UnexpectedError"],
        },
      },
      events: {
        "Auth.Connections.Opened": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          capabilities: {
            publish: ["events.auth"],
            subscribe: ["events.auth"],
          },
        },
      },
    }),
  );

  const audit = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.audit@v1",
      displayName: "Audit",
      description: "Expose audit APIs while depending on auth in tests.",
      capabilities: {
        "read": {
          displayName: "Read audit",
          description: "Read audit entries.",
        },
        "events.audit": {
          displayName: "Audit events",
          description: "Publish and subscribe to audit events.",
        },
      },
      uses: [auth.AuthSessionsMe, auth.AuthConnectionsOpened.subscribe],
      rpc: {
        "Audit.List": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          capabilities: { call: ["read"] },
          errors: ["UnexpectedError"],
        },
      },
      events: {
        "Audit.Recorded": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          capabilities: {
            publish: ["events.audit"],
            subscribe: ["events.audit"],
          },
        },
      },
    }),
  );

  assertEquals(audit.CONTRACT, {
    format: "trellis.contract.v1",
    id: "trellis.audit@v1",
    displayName: "Audit",
    description: "Expose audit APIs while depending on auth in tests.",
    kind: "service",
    capabilities: {
      [globalCapabilityName("trellis.audit@v1", "read")]: {
        displayName: "Read audit",
        description: "Read audit entries.",
      },
      [globalCapabilityName("trellis.audit@v1", "events.audit")]: {
        displayName: "Audit events",
        description: "Publish and subscribe to audit events.",
      },
    },
    schemas: {
      Empty: {
        properties: {},
        type: "object",
      },
      StringValue: {
        properties: { value: { type: "string" } },
        required: ["value"],
        type: "object",
      },
    },
    uses: {
      required: {
        "trellis.auth@v1": {
          contract: "trellis.auth@v1",
          rpc: { call: ["Auth.Sessions.Me"] },
          events: { subscribe: ["Auth.Connections.Opened"] },
        },
      },
    },
    rpc: {
      "Audit.List": {
        version: "v1",
        subject: "rpc.v1.Audit.List",
        input: { schema: "Empty" },
        output: { schema: "StringValue" },
        capabilities: {
          call: [globalCapabilityName("trellis.audit@v1", "read")],
        },
        errors: [{ type: "UnexpectedError" }],
      },
    },
    events: {
      "Audit.Recorded": {
        version: "v1",
        subject: "events.v1.Audit.Recorded",
        event: { schema: "StringValue" },
        capabilities: {
          publish: [
            globalCapabilityName("trellis.audit@v1", "events.audit"),
          ],
          subscribe: [
            globalCapabilityName("trellis.audit@v1", "events.audit"),
          ],
        },
      },
    },
  });

  assertEquals(
    getContractRuntime(audit).ownedApi.rpc["Audit.List"].subject,
    "rpc.v1.Audit.List",
  );
  assertEquals(
    getContractRuntime(audit).usedApi.rpc["Auth.Sessions.Me"].subject,
    "rpc.v1.Auth.Sessions.Me",
  );
  assertEquals(
    getContractRuntime(audit).usedApi.events["Auth.Connections.Opened"].subject,
    "events.v1.Auth.Connections.Opened",
  );
  assertEquals(
    getContractRuntime(audit).api.rpc["Audit.List"].subject,
    "rpc.v1.Audit.List",
  );
  assertEquals(
    getContractRuntime(audit).api.rpc["Auth.Sessions.Me"].subject,
    "rpc.v1.Auth.Sessions.Me",
  );
  assertEquals(
    audit.CONTRACT_DIGEST,
    digestContractManifest(audit.CONTRACT),
  );
  assertEquals(auth.CONTRACT.exports, {
    schemas: ["StringValue"],
  });
});

Deno.test("runtime descriptors precompile exact permission identity", () => {
  const contract = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "permission.source@v1",
      displayName: "Permission source",
      description: "Checks descriptor permission identity.",
      capabilities: {
        "unrelated.capability": {
          displayName: "Unrelated capability",
          description: "Must not affect exact permission identity.",
        },
      },
      rpc: {
        "Demo.Get": {
          version: "v1",
          subject: "unrelated.subject",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
          capabilities: { call: ["unrelated.capability"] },
        },
      },
      operations: {
        "Demo.Run": {
          version: "v2",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
          signals: {
            continue: { input: ref.schema("Empty") },
          },
        },
      },
      events: {
        "Demo.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
          capabilities: { subscribe: ["unrelated.capability"] },
        },
      },
      feeds: {
        "Demo.Live": {
          version: "v1",
          input: ref.schema("Empty"),
          event: ref.schema("StringValue"),
        },
      },
    }),
  );
  const api = getContractRuntime(contract).ownedApi;

  assertEquals(api.rpc["Demo.Get"]?.permission, {
    apiId: "permission.source@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Demo.Get",
    action: "call",
  });
  assertEquals(api.operations["Demo.Run"]?.permissions, {
    invoke: {
      apiId: "permission.source@v1",
      apiVersion: "v2",
      surfaceKind: "operation",
      surfaceName: "Demo.Run",
      action: "invoke",
    },
    observe: {
      apiId: "permission.source@v1",
      apiVersion: "v2",
      surfaceKind: "operation",
      surfaceName: "Demo.Run",
      action: "observe",
    },
    cancel: {
      apiId: "permission.source@v1",
      apiVersion: "v2",
      surfaceKind: "operation",
      surfaceName: "Demo.Run",
      action: "cancel",
    },
    control: {
      continue: {
        apiId: "permission.source@v1",
        apiVersion: "v2",
        surfaceKind: "operation",
        surfaceName: "Demo.Run.continue",
        action: "control",
      },
    },
  });
  assertEquals(api.events["Demo.Changed"]?.publishPermission, {
    apiId: "permission.source@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Demo.Changed",
    action: "publish",
  });
  assertEquals(api.events["Demo.Changed"]?.subscribePermission, {
    apiId: "permission.source@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Demo.Changed",
    action: "subscribe",
  });
  assertEquals(api.feeds?.["Demo.Live"]?.permission, {
    apiId: "permission.source@v1",
    apiVersion: "v1",
    surfaceKind: "feed",
    surfaceName: "Demo.Live",
    action: "subscribe",
  });
  assert(Object.isFrozen(api.rpc["Demo.Get"]?.permission));
  assert(Object.isFrozen(api.operations["Demo.Run"]?.permissions));
});

Deno.test("service contracts do not model runtime heartbeats as contract uses", () => {
  const contract = defineServiceContract({}, () => ({
    id: "baseline-health.service@v1",
    displayName: "Baseline Health Service",
    description:
      "Verify service contracts publish runtime heartbeat by default.",
  }));

  assertEquals(contract.CONTRACT.uses, undefined);
});

Deno.test("health contract does not automatically use itself", () => {
  const contract = defineServiceContract({}, () => ({
    id: "trellis.health@v1",
    displayName: "Trellis Health",
    description: "Expose shared Trellis health projection APIs.",
  }));

  assertEquals(contract.CONTRACT.uses, undefined);
});

function typecheckHealthContractDoesNotInferSelfUse(): void {
  const contract = defineServiceContract({}, () => ({
    id: "trellis.health@v1",
    displayName: "Trellis Health",
    description: "Expose shared Trellis health projection APIs.",
  }));

  // @ts-expect-error health contract should not infer an implicit self-use.
  const invalid = getContractRuntime(contract).usedApi.events.Health;
  void invalid;
}

void typecheckHealthContractDoesNotInferSelfUse;

Deno.test("state feature emits state declarations and required runtime actions", () => {
  const contract = defineDeviceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "baseline-health.device@v1",
      displayName: "Baseline Health Device",
      description: "Verify state features select the State runtime.",
      uses: [state({
        preferences: {
          kind: "value",
          schema: ref.schema("StringValue"),
        },
      })],
    }),
  );

  assertEquals(
    requiredUses(contract.CONTRACT.uses)?.["trellis.auth@v1"],
    undefined,
  );
  assertEquals(
    requiredUses(contract.CONTRACT.uses)?.["trellis.state@v1"]?.contract,
    "trellis.state@v1",
  );
});

Deno.test("explicit health use preserves only selected public surfaces", () => {
  const contract = defineServiceContract({}, () => ({
    id: "explicit-health.service@v1",
    displayName: "Explicit Health Service",
    description: "Verify explicit health use remains explicit.",
    uses: [HealthQuery],
  }));

  assertEquals(requiredUses(contract.CONTRACT.uses)?.["trellis.health@v1"], {
    contract: "trellis.health@v1",
    rpc: { call: ["Health.Query"] },
  });
});

Deno.test("defineServiceContract emits explicit exported schema names without filtering local schemas", () => {
  const contract = defineServiceContract(
    {
      schemas: baseSchemas,
    },
    () => ({
      id: "exports.example@v1",
      displayName: "Exports Example",
      description: "Declare which schema registry entries are public.",
      exports: {
        schemas: ["StringValue"],
      },
      rpc: {
        "Exports.Read": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.exports, {
    schemas: ["StringValue"],
  });
  assertEquals(Object.keys(contract.CONTRACT.schemas ?? {}), [
    "Empty",
    "StringValue",
  ]);
});

Deno.test("defineServiceContract emits docs on owned contract surfaces", () => {
  const contract = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "docs.example@v1",
      displayName: "Docs Example",
      description: "Expose documented contract surfaces.",
      docs: {
        summary: "Documented service.",
        markdown: "# Docs Example\n\nService documentation.",
      },
      uses: [
        state({
          settings: {
            kind: "value",
            schema: ref.schema("StringValue"),
            docs: { markdown: "State settings docs." },
          },
        }),
        jobs({
          "docs.process": {
            payload: ref.schema("StringValue"),
            docs: { markdown: "Job docs." },
          },
        }),
        kv({
          docs: {
            purpose: "Store docs values.",
            schema: ref.schema("StringValue"),
            docs: { markdown: "KV docs." },
          },
        }),
        store({
          blobs: {
            purpose: "Store docs blobs.",
            docs: { markdown: "Store docs." },
          },
        }),
      ],
      rpc: {
        "Docs.Read": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
          docs: { summary: "Read docs.", markdown: "RPC docs." },
        },
      },
      operations: {
        "Docs.Import": {
          version: "v1",
          input: ref.schema("StringValue"),
          output: ref.schema("StringValue"),
          signals: {
            Pause: {
              input: ref.schema("Empty"),
              docs: { markdown: "Pause signal docs." },
            },
          },
          docs: { markdown: "Operation docs." },
        },
      },
      events: {
        "Docs.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
          docs: { markdown: "Event docs." },
        },
      },
      feeds: {
        "Docs.Stream": {
          version: "v1",
          input: ref.schema("Empty"),
          event: ref.schema("StringValue"),
          docs: { markdown: "Feed docs." },
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.docs, {
    summary: "Documented service.",
    markdown: "# Docs Example\n\nService documentation.",
  });
  assertEquals(contract.CONTRACT.rpc?.["Docs.Read"]?.docs, {
    summary: "Read docs.",
    markdown: "RPC docs.",
  });
  assertEquals(
    contract.CONTRACT.operations?.["Docs.Import"]?.signals?.Pause.docs,
    { markdown: "Pause signal docs." },
  );
  assertEquals(contract.CONTRACT.events?.["Docs.Changed"]?.docs, {
    markdown: "Event docs.",
  });
  assertEquals(contract.CONTRACT.feeds?.["Docs.Stream"]?.docs, {
    markdown: "Feed docs.",
  });
  assertEquals(contract.CONTRACT.state?.settings.docs, {
    markdown: "State settings docs.",
  });
  assertEquals(contract.CONTRACT.jobs?.["docs.process"]?.docs, {
    markdown: "Job docs.",
  });
  assertEquals(contract.CONTRACT.resources?.kv?.docs.docs, {
    markdown: "KV docs.",
  });
  assertEquals(contract.CONTRACT.resources?.store?.blobs.docs, {
    markdown: "Store docs.",
  });
  assertEquals(normalizeContractManifest(contract.CONTRACT), contract.CONTRACT);
});

Deno.test("defineServiceContract emits keyed job concurrency policy", () => {
  const contract = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "jobs.keyed@v1",
      displayName: "Keyed Jobs",
      description: "Verify keyed jobs manifest emission.",
      uses: [jobs({
        syncTickets: {
          payload: ref.schema("StringValue"),
          keyConcurrency: {
            key: ["zendesk", "/value", "tickets"],
            maxActive: 1,
            heartbeatIntervalMs: 30_000,
            heartbeatTtlMs: 120_000,
            stalePolicy: "fail-stale",
          },
          queue: {
            maxQueuedPerKey: 0,
            whenFull: "reject",
          },
        },
      })],
    }),
  );

  assertEquals(contract.CONTRACT.jobs?.syncTickets, {
    payload: { schema: "StringValue" },
    keyConcurrency: {
      key: ["zendesk", "/value", "tickets"],
      maxActive: 1,
      heartbeatIntervalMs: 30_000,
      heartbeatTtlMs: 120_000,
      stalePolicy: "fail-stale",
    },
    queue: {
      maxQueuedPerKey: 0,
      whenFull: "reject",
    },
  });
});

Deno.test("contract parsing rejects invalid keyed job JSON Pointer syntax", () => {
  assertThrows(
    () =>
      parseContractManifest({
        format: "trellis.contract.v1",
        id: "jobs.invalid-key@v1",
        displayName: "Invalid Keyed Jobs",
        description: "Invalid keyed jobs manifest.",
        kind: "service",
        schemas: { StringValue: JSON.parse(JSON.stringify(StringSchema)) },
        jobs: {
          syncTickets: {
            payload: { schema: "StringValue" },
            keyConcurrency: {
              key: ["zendesk", "/origin~bad"],
            },
          },
        },
      }),
    Error,
    "invalid JSON Pointer escape",
  );
});

Deno.test("contract digest ignores docs-only differences", () => {
  const defineDocumented = (markdown: string) =>
    defineServiceContract({ schemas: baseSchemas }, (ref) => ({
      id: "digest.docs@v1",
      displayName: "Digest Docs",
      description: "Verify docs are not part of contract identity.",
      docs: { summary: markdown, markdown },
      uses: [
        state({
          settings: {
            kind: "value",
            schema: ref.schema("StringValue"),
            docs: { markdown },
          },
        }),
        jobs({
          "docs.process": {
            payload: ref.schema("StringValue"),
            docs: { markdown },
          },
        }),
        kv({
          docs: {
            purpose: "Store docs values.",
            schema: ref.schema("StringValue"),
            docs: { markdown },
          },
        }),
        store({
          blobs: {
            purpose: "Store docs blobs.",
            docs: { markdown },
          },
        }),
      ],
      rpc: {
        "Docs.Read": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
          docs: { markdown },
        },
      },
      operations: {
        "Docs.Import": {
          version: "v1",
          input: ref.schema("StringValue"),
          output: ref.schema("StringValue"),
          signals: {
            Pause: { input: ref.schema("Empty"), docs: { markdown } },
          },
          docs: { markdown },
        },
      },
      events: {
        "Docs.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
          docs: { markdown },
        },
      },
      feeds: {
        "Docs.Stream": {
          version: "v1",
          input: ref.schema("Empty"),
          event: ref.schema("StringValue"),
          docs: { markdown },
        },
      },
    }));

  const first = defineDocumented("First docs.");
  const second = defineDocumented("Second docs.");

  assertNotEquals(first.CONTRACT.docs, second.CONTRACT.docs);
  assertNotEquals(
    first.CONTRACT.operations?.["Docs.Import"]?.signals?.Pause.docs,
    second.CONTRACT.operations?.["Docs.Import"]?.signals?.Pause.docs,
  );
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("contract helpers reject registry-side exports at runtime", () => {
  assertThrows(
    () =>
      defineServiceContract(
        JSON.parse('{"exports":{"schemas":["StringValue"]}}'),
        () => ({
          id: "exports.registry-service@v1",
          displayName: "Registry Exports Service",
          description: "Should reject registry-side exports.",
        }),
      ),
    Error,
    "contract exports must be declared in the callback body",
  );

  assertThrows(
    () =>
      defineAppContract(
        JSON.parse('{"exports":{"schemas":["StringValue"]}}'),
        () => ({
          id: "exports.registry-app@v1",
          displayName: "Registry Exports App",
          description: "Should reject registry-side exports.",
        }),
      ),
    Error,
    "contract exports must be declared in the callback body",
  );
});

Deno.test("contract helpers reject registry-side capabilities at runtime", () => {
  assertThrows(
    () =>
      defineServiceContract(
        JSON.parse(
          '{"capabilities":{"read":{"displayName":"Read","description":"Read."}}}',
        ),
        () => ({
          id: "capabilities.registry-service@v1",
          displayName: "Registry Capabilities Service",
          description: "Should reject registry-side capabilities.",
        }),
      ),
    Error,
    "contract capabilities must be declared in the callback body",
  );

  assertThrows(
    () =>
      defineAppContract(
        JSON.parse(
          '{"capabilities":{"read":{"displayName":"Read","description":"Read."}}}',
        ),
        () => ({
          id: "capabilities.registry-app@v1",
          displayName: "Registry Capabilities App",
          description: "Should reject registry-side capabilities.",
        }),
      ),
    Error,
    "contract capabilities must be declared in the callback body",
  );
});

Deno.test("defineServiceContract emits RPC error refs using declared wire types", () => {
  const NotFoundError = defineError({
    type: "NotFoundError",
    fields: {},
    message: "Not found",
  });

  const contract = defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors: {
        NotFoundError,
      },
    },
    (ref) => ({
      id: "local-errors.example@v1",
      displayName: "Local Errors Example",
      description: "Verify RPC error refs emit declared error types.",
      rpc: {
        "Workspace.Get": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [ref.error("NotFoundError"), ref.error("UnexpectedError")],
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.rpc?.["Workspace.Get"]?.errors, [
    { type: "NotFoundError" },
    { type: "UnexpectedError" },
  ]);
});
Deno.test("defineServiceContract emits operation error refs using declared wire types", () => {
  const NotFoundError = defineError({
    type: "NotFoundError",
    fields: {},
    message: "Not found",
  });

  const contract = defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors: {
        NotFoundError,
      },
    },
    (ref) => ({
      id: "local-errors.example@v1",
      displayName: "Local Errors Example",
      description: "Verify operation error refs emit declared error types.",
      operations: {
        "Example.Process": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [ref.error("NotFoundError"), ref.error("UnexpectedError")],
          capabilities: { call: [] },
        },
      },
    }),
  );

  // Assert emitted manifest
  const emitted = contract.CONTRACT.operations?.["Example.Process"];
  assertEquals(emitted?.errors, [
    { type: "NotFoundError" },
    { type: "UnexpectedError" },
  ]);

  // Assert runtime API metadata
  const api =
    getContractRuntime(contract).ownedApi.operations["Example.Process"];
  assertEquals(api.declaredErrorTypes, ["NotFoundError", "UnexpectedError"]);
  assert(api.runtimeErrors !== undefined);
  assertEquals(api.runtimeErrors.length, 1);
  assertEquals(api.runtimeErrors[0].type, "NotFoundError");
  assertEquals(api.errors, ["NotFoundError", "UnexpectedError"]);
});

Deno.test("state feature emits named state declarations", async () => {
  const dashboardSchemas = {
    Preferences: Type.Object({ theme: Type.String() }),
    Draft: Type.Object({ title: Type.String() }),
  } as const;

  const dashboard = defineAppContract(
    { schemas: dashboardSchemas },
    (ref) => ({
      id: "trellis.dashboard@v1",
      displayName: "Dashboard",
      description: "Persist dashboard preferences and drafts.",
      uses: [state({
        preferences: {
          kind: "value",
          schema: ref.schema("Preferences"),
        },
        drafts: {
          kind: "map",
          schema: ref.schema("Draft"),
        },
      })],
    }),
  );

  assertEquals(dashboard.CONTRACT, {
    format: "trellis.contract.v1",
    id: "trellis.dashboard@v1",
    displayName: "Dashboard",
    description: "Persist dashboard preferences and drafts.",
    kind: "app",
    schemas: {
      Preferences: {
        properties: { theme: { type: "string" } },
        required: ["theme"],
        type: "object",
      },
      Draft: {
        properties: { title: { type: "string" } },
        required: ["title"],
        type: "object",
      },
    },
    state: {
      preferences: {
        kind: "value",
        schema: { schema: "Preferences" },
      },
      drafts: {
        kind: "map",
        schema: { schema: "Draft" },
      },
    },
    uses: {
      required: {
        "trellis.state@v1": {
          contract: "trellis.state@v1",
          rpc: {
            call: ["State.Delete", "State.Get", "State.List", "State.Put"],
          },
        },
      },
    },
  });

  assertEquals(
    getContractRuntime(dashboard).usedApi.rpc["State.Get"].subject,
    "rpc.v1.State.Get",
  );
  assertEquals(
    getContractRuntime(dashboard).api.rpc["State.Put"].subject,
    "rpc.v1.State.Put",
  );

  assertEquals(
    dashboard.CONTRACT_DIGEST,
    digestContractManifest(dashboard.CONTRACT),
  );
});

Deno.test("contract digest ignores display metadata, exports, and unused schemas", () => {
  const schemas = {
    Used: Type.Object({ value: Type.String() }),
    Unused: Type.Object({ label: Type.String() }),
  } as const;

  const first = defineServiceContract({ schemas }, (ref) => ({
    id: "digest.projection@v1",
    displayName: "Digest Projection",
    description: "First description.",
    exports: { schemas: ["Unused"] },
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Used"),
        output: ref.schema("Used"),
      },
    },
  }));

  const second = defineServiceContract({
    schemas: {
      Used: schemas.Used,
      Unused: Type.Object({ changed: Type.Boolean() }),
    },
  }, (ref) => ({
    id: "digest.projection@v1",
    displayName: "Renamed Digest Projection",
    description: "Second description.",
    exports: { schemas: ["Used", "Unused"] },
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Used"),
        output: ref.schema("Used"),
      },
    },
  }));

  assertNotEquals(first.CONTRACT.displayName, second.CONTRACT.displayName);
  assertNotEquals(first.CONTRACT.description, second.CONTRACT.description);
  assertNotEquals(first.CONTRACT.exports, second.CONTRACT.exports);
  assertNotEquals(
    first.CONTRACT.schemas?.Unused,
    second.CONTRACT.schemas?.Unused,
  );
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("contract digest normalizes capability order and duplicates", () => {
  const digestCapabilities = {
    a: { displayName: "A", description: "A capability." },
    b: { displayName: "B", description: "B capability." },
    "events.admin": {
      displayName: "Admin events",
      description: "Administer events.",
    },
    "events.audit": {
      displayName: "Audit events",
      description: "Audit events.",
    },
    "events.read": {
      displayName: "Read events",
      description: "Read events.",
    },
    "events.write": {
      displayName: "Write events",
      description: "Write events.",
    },
    "operations.control": {
      displayName: "Control operations",
      description: "Control operations.",
    },
  } as const;
  const first = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.capabilities@v1",
    displayName: "Digest Capabilities",
    description: "Verify capability normalization.",
    capabilities: {
      a: { displayName: "A", description: "A capability." },
      b: { displayName: "B", description: "B capability." },
      "events.admin": {
        displayName: "Admin events",
        description: "Administer events.",
      },
      "events.audit": {
        displayName: "Audit events",
        description: "Audit events.",
      },
      "events.read": {
        displayName: "Read events",
        description: "Read events.",
      },
      "events.write": {
        displayName: "Write events",
        description: "Write events.",
      },
      "operations.control": {
        displayName: "Control operations",
        description: "Control operations.",
      },
    },
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["b", "a", "a"] },
      },
    },
    events: {
      "Digest.Changed": {
        version: "v1",
        event: ref.schema("StringValue"),
        capabilities: {
          publish: ["events.write", "events.admin", "events.write"],
          subscribe: ["events.read", "events.audit", "events.read"],
        },
      },
    },
    operations: {
      "Digest.Import": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: {
          control: [
            "operations.control",
            "b",
            "operations.control",
          ],
        },
      },
    },
  }));

  const second = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.capabilities@v1",
    displayName: "Digest Capabilities",
    description: "Verify capability normalization.",
    capabilities: {
      b: { displayName: "B", description: "B capability." },
      a: { displayName: "A", description: "A capability." },
      "events.write": {
        displayName: "Write events",
        description: "Write events.",
      },
      "events.read": {
        displayName: "Read events",
        description: "Read events.",
      },
      "events.admin": {
        displayName: "Admin events",
        description: "Administer events.",
      },
      "operations.control": {
        displayName: "Control operations",
        description: "Control operations.",
      },
      "events.audit": {
        displayName: "Audit events",
        description: "Audit events.",
      },
    },
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["a", "b"] },
      },
    },
    events: {
      "Digest.Changed": {
        version: "v1",
        event: ref.schema("StringValue"),
        capabilities: {
          publish: ["events.admin", "events.write"],
          subscribe: ["events.audit", "events.read"],
        },
      },
    },
    operations: {
      "Digest.Import": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { control: ["b", "operations.control"] },
      },
    },
  }));

  assertEquals(first.CONTRACT.rpc?.["Digest.Read"]?.capabilities?.call, [
    globalCapabilityName("digest.capabilities@v1", "a"),
    globalCapabilityName("digest.capabilities@v1", "b"),
  ]);
  assertEquals(
    first.CONTRACT.operations?.["Digest.Import"]?.capabilities?.control,
    [
      globalCapabilityName("digest.capabilities@v1", "b"),
      globalCapabilityName("digest.capabilities@v1", "operations.control"),
    ],
  );
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("contract digest includes operation signal input schemas", () => {
  const schemas = {
    Empty: EmptySchema,
    Result: StringSchema,
    FirstSignal: Type.Object({ value: Type.String() }),
    SecondSignal: Type.Object({ value: Type.Number() }),
  } as const;

  const first = defineServiceContract({ schemas }, (ref) => ({
    id: "digest.operation-signals@v1",
    displayName: "Operation Signals",
    description: "Digest operation signals.",
    operations: {
      "Signals.Run": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Result"),
        signals: {
          continue: { input: ref.schema("FirstSignal") },
        },
      },
    },
  }));

  const second = defineServiceContract({ schemas }, (ref) => ({
    id: "digest.operation-signals@v1",
    displayName: "Operation Signals",
    description: "Digest operation signals.",
    operations: {
      "Signals.Run": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Result"),
        signals: {
          continue: { input: ref.schema("SecondSignal") },
        },
      },
    },
  }));

  assertEquals(first.CONTRACT.schemas?.FirstSignal, {
    properties: { value: { type: "string" } },
    required: ["value"],
    type: "object",
  });
  assertNotEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("defineServiceContract emits top-level capabilities with global names", () => {
  const authCapabilities = {
    "users.read": {
      displayName: "Read users",
      description: "Allows reading user profiles.",
      consequence: "Exposes user profile data.",
    },
  } as const;
  const contract = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "trellis.auth@v1",
    displayName: "Auth Capabilities",
    description: "Verify capability declarations emit globally.",
    capabilities: authCapabilities,
    rpc: {
      "Auth.Users.List": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["platform::audit", "users.read"] },
      },
    },
    operations: {
      "Auth.Users.Export": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: {
          call: ["users.read"],
          observe: ["users.read"],
          cancel: ["platform::cancel"],
        },
      },
    },
    events: {
      "Auth.UserChanged": {
        version: "v1",
        event: ref.schema("StringValue"),
        capabilities: {
          publish: ["users.read"],
          subscribe: ["external::subscribe", "users.read"],
        },
      },
    },
  }));

  const globalUsersRead = globalCapabilityName(
    "trellis.auth@v1",
    "users.read",
  );

  assertEquals(contract.CONTRACT.capabilities, {
    [globalUsersRead]: {
      displayName: "Read users",
      description: "Allows reading user profiles.",
      consequence: "Exposes user profile data.",
    },
  });
  assertEquals(contract.CONTRACT.rpc?.["Auth.Users.List"]?.capabilities?.call, [
    "platform::audit",
    globalUsersRead,
  ]);
  assertEquals(
    contract.CONTRACT.operations?.["Auth.Users.Export"]?.capabilities,
    {
      call: [globalUsersRead],
      observe: [globalUsersRead],
      cancel: ["platform::cancel"],
    },
  );
  assertEquals(contract.CONTRACT.events?.["Auth.UserChanged"]?.capabilities, {
    publish: [globalUsersRead],
    subscribe: ["external::subscribe", globalUsersRead],
  });
  assertEquals(
    getContractRuntime(contract).ownedApi.rpc["Auth.Users.List"]
      .callerCapabilities,
    ["platform::audit", globalUsersRead],
  );
  assertEquals(
    getContractRuntime(contract).ownedApi.events["Auth.UserChanged"]
      .subscribeCapabilities,
    ["external::subscribe", globalUsersRead],
  );
});

Deno.test("defineServiceContract rejects local capabilities with contract namespace prefixes", () => {
  assertThrows(
    () =>
      defineServiceContract({ schemas: baseSchemas }, (ref) => ({
        id: "trellis.core@v1",
        displayName: "Core Capabilities",
        description:
          "Verify namespace-prefixed local capabilities are rejected.",
        capabilities: {
          "trellis.core.catalog.read": {
            displayName: "Read catalog",
            description: "Read catalog entries.",
          },
        },
        rpc: {
          "Trellis.Catalog": {
            version: "v1",
            input: ref.schema("Empty"),
            output: ref.schema("StringValue"),
            capabilities: { call: ["trellis.core.catalog.read"] },
          },
        },
      })),
    Error,
    "must not start with contract namespace prefix 'trellis.core.'",
  );

  assertThrows(
    () =>
      defineServiceContract({ schemas: baseSchemas }, (ref) => ({
        id: "trellis.core@v1",
        displayName: "Core Capabilities",
        description:
          "Verify namespace-leaf-prefixed local capabilities are rejected.",
        capabilities: {
          "core.catalog.read": {
            displayName: "Read catalog",
            description: "Read catalog entries.",
          },
        },
        rpc: {
          "Trellis.Catalog": {
            version: "v1",
            input: ref.schema("Empty"),
            output: ref.schema("StringValue"),
            capabilities: { call: ["core.catalog.read"] },
          },
        },
      })),
    Error,
    "must not start with contract namespace prefix 'core.'",
  );
});

Deno.test("contract digest changes when capability metadata changes", () => {
  const firstCapabilities = {
    read: {
      displayName: "Read",
      description: "Read records.",
    },
  } as const;
  const secondCapabilities = {
    read: {
      displayName: "Read",
      description: "Read records with changed metadata.",
    },
  } as const;
  const first = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.capability-metadata@v1",
    displayName: "Capability Metadata",
    description: "First capability metadata.",
    capabilities: firstCapabilities,
    rpc: {
      "Capability.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["read"] },
      },
    },
  }));

  const second = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.capability-metadata@v1",
    displayName: "Capability Metadata",
    description: "First capability metadata.",
    capabilities: secondCapabilities,
    rpc: {
      "Capability.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["read"] },
      },
    },
  }));

  assertNotEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("shared manifest normalization preserves digest-bearing capabilities", () => {
  const capability = globalCapabilityName(
    "digest.normalization@v1",
    "read",
  );
  const contract = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.normalization@v1",
    displayName: "Digest Normalization",
    description: "Verify shared contract manifest normalization.",
    capabilities: {
      read: {
        displayName: "Read",
        description: "Read records.",
      },
    },
    rpc: {
      "Normalization.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["read"] },
      },
    },
  }));

  const raw = {
    ...contract.CONTRACT,
    unknownFutureField: true,
  } as typeof contract.CONTRACT;
  const normalized = normalizeContractManifest(raw);

  assertEquals(Object.hasOwn(normalized, "unknownFutureField"), false);
  assertEquals(normalized.capabilities, {
    [capability]: {
      displayName: "Read",
      description: "Read records.",
    },
  });
  assertEquals(
    digestContractManifest(raw),
    digestContractManifest(normalized),
  );
});

Deno.test("contract digest normalizes uses logical-name order and duplicates", () => {
  const dependency = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "digest.dependency@v1",
      displayName: "Digest Dependency",
      description: "Expose dependency surfaces for use normalization.",
      rpc: {
        "Dependency.A": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
        },
        "Dependency.B": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
        },
      },
    }),
  );

  const first = defineServiceContract({}, () => ({
    id: "digest.uses@v1",
    displayName: "Digest Uses",
    description: "Verify uses normalization.",
    uses: [
      dependency.DependencyB,
      dependency.DependencyA,
      dependency.DependencyA,
    ],
  }));

  const second = defineServiceContract({}, () => ({
    id: "digest.uses@v1",
    displayName: "Digest Uses",
    description: "Verify uses normalization.",
    uses: [dependency.DependencyA, dependency.DependencyB],
  }));

  assertEquals(
    requiredUses(first.CONTRACT.uses)?.["digest.dependency@v1"].rpc?.call,
    [
      "Dependency.A",
      "Dependency.B",
    ],
  );
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("descriptor selections emit contract-ID-keyed manifest uses", () => {
  const dependency = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "grouped.dependency@v1",
      displayName: "Grouped Dependency",
      description: "Expose dependency surfaces for grouped required uses.",
      rpc: {
        "Dependency.Read": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
        },
      },
      events: {
        "Dependency.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
    }),
  );

  const grouped = defineServiceContract({}, () => ({
    id: "grouped.required@v1",
    displayName: "Grouped Required",
    description: "Declare grouped required uses.",
    uses: [dependency.DependencyChanged.subscribe, dependency.DependencyRead],
  }));

  assertEquals(grouped.CONTRACT.uses, {
    required: {
      "grouped.dependency@v1": {
        contract: "grouped.dependency@v1",
        rpc: { call: ["Dependency.Read"] },
        events: { subscribe: ["Dependency.Changed"] },
      },
    },
  });
  assertEquals(
    getContractRuntime(grouped).usedApi.rpc["Dependency.Read"].subject,
    "rpc.v1.Dependency.Read",
  );
  assertEquals(
    getContractRuntime(grouped).usedApi.events["Dependency.Changed"].subject,
    "events.v1.Dependency.Changed",
  );
  assertEquals(
    grouped.CONTRACT_DIGEST,
    digestContractManifest(grouped.CONTRACT),
  );
});

Deno.test("grouped optional uses normalize selectors and affect digest", () => {
  const dependency = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "grouped.optional-dependency@v1",
      displayName: "Grouped Optional Dependency",
      description: "Expose dependency surfaces for grouped optional uses.",
      rpc: {
        "Dependency.Read": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
        },
      },
      events: {
        "Dependency.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
      feeds: {
        "Dependency.Changes": {
          version: "v1",
          input: ref.schema("Empty"),
          event: ref.schema("StringValue"),
        },
      },
    }),
  );

  const requiredOnly = defineServiceContract({}, () => ({
    id: "grouped.optional@v1",
    displayName: "Grouped Optional",
    description: "Declare grouped uses without optional dependencies.",
    uses: [],
  }));

  const withOptional = defineServiceContract({}, () => ({
    id: "grouped.optional@v1",
    displayName: "Grouped Optional",
    description: "Declare grouped optional dependencies.",
    uses: [
      optional(
        dependency.DependencyChanged.subscribe,
        dependency.DependencyChanges,
      ),
    ],
  }));

  assertEquals(withOptional.CONTRACT.uses, {
    optional: {
      "grouped.optional-dependency@v1": {
        contract: "grouped.optional-dependency@v1",
        events: { subscribe: ["Dependency.Changed"] },
        feeds: { subscribe: ["Dependency.Changes"] },
      },
    },
  });
  assertEquals(
    getContractRuntime(withOptional).usedApi.feeds["Dependency.Changes"]
      .subject,
    "feed.v1.Dependency.Changes",
  );
  assertNotEquals(requiredOnly.CONTRACT_DIGEST, withOptional.CONTRACT_DIGEST);
});

Deno.test("one dependency cannot mix required and optional selections", () => {
  const dependency = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "grouped.duplicate-dependency@v1",
      displayName: "Grouped Duplicate Dependency",
      description: "Expose dependency surfaces for duplicate grouped uses.",
      rpc: {
        "Dependency.Read": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("StringValue"),
        },
      },
      events: {
        "Dependency.Changed": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
    }),
  );

  const requiredOnly = defineServiceContract({}, () => ({
    id: "grouped.duplicate@v1",
    displayName: "Grouped Duplicate",
    description: "Declare required uses.",
    uses: [dependency.DependencyRead],
  }));

  assertEquals(
    requiredUses(requiredOnly.CONTRACT.uses)
      ?.["grouped.duplicate-dependency@v1"].rpc?.call,
    ["Dependency.Read"],
  );
  assertThrows(
    () =>
      defineServiceContract({}, () => ({
        id: "grouped.duplicate@v1",
        displayName: "Grouped Duplicate",
        description: "Reject mixed dependency optionality.",
        uses: [
          dependency.DependencyRead,
          optional(dependency.DependencyChanged.subscribe),
        ],
      })),
    Error,
    "cannot mix required and optional actions",
  );
});

Deno.test("contract digest normalizes RPC error order and duplicates", () => {
  const NotFoundError = defineError({
    type: "NotFoundError",
    fields: {},
    message: "Not found",
  });

  const registry = {
    schemas: { Empty: EmptySchema },
    errors: { NotFoundError },
  } as const;

  const first = defineServiceContract(registry, (ref) => ({
    id: "digest.errors@v1",
    displayName: "Digest Errors",
    description: "Verify RPC error normalization.",
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Empty"),
        errors: [
          ref.error("UnexpectedError"),
          ref.error("NotFoundError"),
          ref.error("UnexpectedError"),
        ],
      },
    },
  }));

  const second = defineServiceContract(registry, (ref) => ({
    id: "digest.errors@v1",
    displayName: "Digest Errors",
    description: "Verify RPC error normalization.",
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("Empty"),
        errors: [ref.error("NotFoundError"), ref.error("UnexpectedError")],
      },
    },
  }));

  assertEquals(first.CONTRACT.rpc?.["Digest.Read"]?.errors, [
    { type: "NotFoundError" },
    { type: "UnexpectedError" },
  ]);
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("contract digest changes for meaningful interface changes", () => {
  const digestReadCapability = {
    "digest.read": {
      displayName: "Read digest",
      description: "Read digest records.",
    },
  } as const;
  const first = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.meaningful@v1",
    displayName: "Digest Meaningful",
    description: "First interface.",
    capabilities: digestReadCapability,
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
      },
    },
  }));

  const second = defineServiceContract({ schemas: baseSchemas }, (ref) => ({
    id: "digest.meaningful@v1",
    displayName: "Digest Meaningful",
    description: "First interface.",
    capabilities: digestReadCapability,
    rpc: {
      "Digest.Read": {
        version: "v1",
        input: ref.schema("Empty"),
        output: ref.schema("StringValue"),
        capabilities: { call: ["digest.read"] },
      },
    },
  }));

  assertNotEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("defineServiceContract derives local error schemas from defineError runtime metadata", () => {
  const NotFoundError = defineError({
    type: "NotFoundError",
    fields: {},
    message: "Not found",
  });

  const contract = defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors: {
        NotFoundError,
      },
    },
    (ref) => ({
      id: "derived-local-errors.example@v1",
      displayName: "Derived Local Errors Example",
      description:
        "Verify local error schemas can be derived from defineError metadata.",
      rpc: {
        "Workspace.Get": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [ref.error("NotFoundError"), ref.error("UnexpectedError")],
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.errors?.NotFoundError?.schema, {
    schema: "NotFoundErrorData",
  });
  assertEquals(
    contract.CONTRACT.schemas?.NotFoundErrorData,
    JSON.parse(JSON.stringify(NotFoundError.schema)),
  );
  assertEquals(
    unwrapSchema(
      getContractRuntime(contract).ownedApi.rpc["Workspace.Get"].runtimeErrors
        ?.[0]?.schema ?? {},
    ),
    JSON.parse(JSON.stringify(NotFoundError.schema)),
  );
});

Deno.test("defineServiceContract ignores non-error exports in a mixed errors barrel", () => {
  const NotFoundError = defineError({
    type: "NotFoundError",
    fields: {},
    message: "Not found",
  });

  const errors = {
    NotFoundError,
    NotFoundErrorDataSchema: NotFoundError.schema,
  } as const;

  const contract = defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors,
    },
    (ref) => ({
      id: "mixed-local-errors.example@v1",
      displayName: "Mixed Local Errors Example",
      description:
        "Verify mixed error barrels can be passed directly to defineServiceContract.",
      rpc: {
        "Workspace.Get": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [ref.error("NotFoundError"), ref.error("UnexpectedError")],
        },
      },
    }),
  );

  assertEquals(Object.keys(contract.CONTRACT.errors ?? {}), ["NotFoundError"]);
  assertEquals(contract.CONTRACT.errors?.NotFoundError?.type, "NotFoundError");
  assertEquals(contract.CONTRACT.errors?.NotFoundError?.schema, {
    schema: "NotFoundErrorData",
  });
});

Deno.test("defineServiceContract emits generated defineError classes", () => {
  const WorkspaceMissingError = defineError({
    type: "WorkspaceMissingError",
    fields: {
      resourceId: Type.String(),
    },
    message: ({ resourceId }) => `Workspace ${resourceId} not found`,
  });

  const errors = {
    WorkspaceMissingError,
    WorkspaceMissingErrorDataSchema: WorkspaceMissingError.schema,
  } as const;

  const contract = defineServiceContract(
    {
      schemas: {
        Empty: EmptySchema,
      },
      errors,
    },
    (ref) => ({
      id: "generated-local-errors.example@v1",
      displayName: "Generated Local Errors Example",
      description:
        "Verify generated defineError classes can be passed directly to defineServiceContract.",
      rpc: {
        "Workspace.Get": {
          version: "v1",
          input: ref.schema("Empty"),
          output: ref.schema("Empty"),
          errors: [
            ref.error("WorkspaceMissingError"),
            ref.error("UnexpectedError"),
          ],
        },
      },
    }),
  );

  assertEquals(
    contract.CONTRACT.errors?.WorkspaceMissingError?.type,
    "WorkspaceMissingError",
  );
  assertEquals(contract.CONTRACT.errors?.WorkspaceMissingError?.schema, {
    schema: "WorkspaceMissingErrorData",
  });
  assertEquals(
    unwrapSchema(
      getContractRuntime(contract).ownedApi.rpc["Workspace.Get"].runtimeErrors
        ?.[0]?.schema ?? {},
    ),
    JSON.parse(JSON.stringify(WorkspaceMissingError.schema)),
  );
});

Deno.test("defineServiceContract rejects duplicate logical keys across used and owned APIs", () => {
  const auth = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.auth@v1",
      displayName: "Trellis Auth",
      description: "Expose auth RPCs in duplicate-key tests.",
      rpc: {
        "Auth.Sessions.Me": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        () => ({
          id: "duplicate@v1",
          displayName: "Duplicate",
          description: "Trigger duplicate logical RPC key validation.",
          uses: [auth.AuthSessionsMe],
          rpc: {
            "Auth.Sessions.Me": {
              version: "v1",
              input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
              output: schemaRef<typeof baseSchemas, "StringValue">(
                "StringValue",
              ),
            },
          },
        }),
      ),
    Error,
    "Duplicate rpc key 'Auth.Sessions.Me'",
  );
});

Deno.test("defineServiceContract emits KV resources with schema-backed defaults", () => {
  const kvSchemas = {
    Item: Type.Object({ value: Type.String() }),
  } as const;

  const contract = defineServiceContract(
    { schemas: kvSchemas },
    (ref) => ({
      id: "kv.example@v1",
      displayName: "KV Example",
      description: "Expose schema-backed KV resource declarations.",
      uses: [kv({
        items: {
          purpose: "Persist typed items",
          schema: ref.schema("Item"),
        },
      })],
    }),
  );

  assertEquals(contract.CONTRACT.resources?.kv?.items, {
    purpose: "Persist typed items",
    schema: { schema: "Item" },
    required: true,
    history: 1,
    ttlMs: 0,
  });
});

Deno.test("defineServiceContract emits top-level jobs with defaults", () => {
  const contract = defineServiceContract({ schemas: baseSchemas }, () => ({
    id: "jobs.example@v1",
    displayName: "Jobs Example",
    description: "Expose top-level jobs declarations in emitted manifests.",
    uses: [jobs({
      refresh: {
        payload: { schema: "Empty" },
        update: { schema: "StringValue" },
        result: { schema: "StringValue" },
      },
    })],
  }));

  assertEquals(contract.CONTRACT.jobs?.refresh, {
    payload: { schema: "Empty" },
    update: { schema: "StringValue" },
    result: { schema: "StringValue" },
  });
  assertEquals("jobs" in (contract.CONTRACT.resources ?? {}), false);
});

Deno.test("defineServiceContract emits dependency event consumer groups", () => {
  const source = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "events.source@v1",
      displayName: "Event Source",
      description: "Expose events for event consumer tests.",
      events: {
        "Source.Created": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
        "Source.Updated": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  const contract = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "events.consumer@v1",
      displayName: "Event Consumer",
      description: "Declare durable event consumer groups.",
      uses: [source.SourceCreated.subscribe, source.SourceUpdated.subscribe],
      eventConsumers: {
        ingest: {
          uses: {
            "events.source@v1": ["Source.Updated", "Source.Created"],
          },
          ordering: "parallel",
          ackWaitMs: 1_000,
          maxDeliver: 3,
          backoffMs: [0, 100],
          docs: { markdown: "Process source events." },
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.eventConsumers?.ingest, {
    uses: {
      "events.source@v1": ["Source.Created", "Source.Updated"],
    },
    replay: "new",
    ordering: "parallel",
    ackWaitMs: 1_000,
    maxDeliver: 3,
    backoffMs: [0, 100],
    docs: { markdown: "Process source events." },
  });
  const ingest = contract.CONTRACT.eventConsumers?.ingest;
  if (!ingest) {
    throw new Error("expected emitted ingest event consumer group");
  }

  const withoutDocs = normalizeContractManifest({
    ...contract.CONTRACT,
    eventConsumers: {
      ingest: {
        ...ingest,
        docs: undefined,
      },
    },
  });
  const changedReplay = normalizeContractManifest({
    ...contract.CONTRACT,
    eventConsumers: {
      ingest: {
        ...ingest,
        replay: "all",
      },
    },
  });
  assertEquals(
    digestContractManifest(contract.CONTRACT),
    digestContractManifest(withoutDocs),
  );
  assertNotEquals(
    digestContractManifest(contract.CONTRACT),
    digestContractManifest(changedReplay),
  );
});

Deno.test("defineServiceContract emits self-owned event consumer groups", () => {
  const first = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.self-consumer@v1",
      displayName: "Self Consumer",
      description: "Consume owned events without a dependency use alias.",
      events: {
        "Self.Created": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
        "Self.Updated": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
      eventConsumers: {
        ingest: {
          self: ["Self.Updated", "Self.Created"],
        },
      },
    }),
  );

  const second = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.self-consumer@v1",
      displayName: "Self Consumer",
      description: "Consume owned events without a dependency use alias.",
      events: {
        "Self.Updated": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
        "Self.Created": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
      eventConsumers: {
        ingest: {
          self: ["Self.Created", "Self.Updated"],
        },
      },
    }),
  );

  assertEquals(first.CONTRACT.eventConsumers?.ingest.self, [
    "Self.Created",
    "Self.Updated",
  ]);
  assertEquals(first.CONTRACT_DIGEST, digestContractManifest(first.CONTRACT));
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("defineServiceContract emits mixed dependency and self event consumer groups", () => {
  const source = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.mixed-source@v1",
      displayName: "Mixed Source",
      description: "Expose dependency events for mixed consumer tests.",
      events: {
        "Source.Created": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
    }),
  );

  const contract = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.mixed-consumer@v1",
      displayName: "Mixed Consumer",
      description: "Consume dependency and owned events in one group.",
      uses: [source.SourceCreated.subscribe],
      events: {
        "Self.Created": {
          version: "v1",
          event: ref.schema("StringValue"),
        },
      },
      eventConsumers: {
        ingest: {
          uses: { "events.mixed-source@v1": ["Source.Created"] },
          self: ["Self.Created"],
        },
      },
    }),
  );

  assertEquals(contract.CONTRACT.eventConsumers?.ingest, {
    uses: { "events.mixed-source@v1": ["Source.Created"] },
    self: ["Self.Created"],
    replay: "new",
    ordering: "strict",
  });
});

Deno.test("defineServiceContract sorts event consumer aliases and arrays stably", () => {
  const sourceA = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.sorted-a@v1",
      displayName: "Sorted A",
      description: "Expose A events for sorting tests.",
      events: {
        "A.Created": { version: "v1", event: ref.schema("StringValue") },
        "A.Updated": { version: "v1", event: ref.schema("StringValue") },
      },
    }),
  );
  const sourceB = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.sorted-b@v1",
      displayName: "Sorted B",
      description: "Expose B events for sorting tests.",
      events: {
        "B.Created": { version: "v1", event: ref.schema("StringValue") },
        "B.Updated": { version: "v1", event: ref.schema("StringValue") },
      },
    }),
  );

  const first = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.sorted-consumer@v1",
      displayName: "Sorted Consumer",
      description: "Normalize grouped event consumer order.",
      uses: [
        sourceB.BUpdated.subscribe,
        sourceB.BCreated.subscribe,
        sourceA.AUpdated.subscribe,
        sourceA.ACreated.subscribe,
      ],
      events: {
        "Self.Z": { version: "v1", event: ref.schema("StringValue") },
        "Self.A": { version: "v1", event: ref.schema("StringValue") },
      },
      eventConsumers: {
        ingest: {
          uses: {
            "events.sorted-b@v1": ["B.Updated", "B.Created", "B.Updated"],
            "events.sorted-a@v1": ["A.Updated", "A.Created"],
          },
          self: ["Self.Z", "Self.A", "Self.Z"],
        },
      },
    }),
  );

  const second = defineServiceContract(
    { schemas: baseSchemas },
    (ref) => ({
      id: "events.sorted-consumer@v1",
      displayName: "Sorted Consumer",
      description: "Normalize grouped event consumer order.",
      uses: [
        sourceA.ACreated.subscribe,
        sourceA.AUpdated.subscribe,
        sourceB.BCreated.subscribe,
        sourceB.BUpdated.subscribe,
      ],
      events: {
        "Self.A": { version: "v1", event: ref.schema("StringValue") },
        "Self.Z": { version: "v1", event: ref.schema("StringValue") },
      },
      eventConsumers: {
        ingest: {
          uses: {
            "events.sorted-a@v1": ["A.Created", "A.Updated"],
            "events.sorted-b@v1": ["B.Created", "B.Updated"],
          },
          self: ["Self.A", "Self.Z"],
        },
      },
    }),
  );

  assertEquals(first.CONTRACT.eventConsumers?.ingest.uses, {
    "events.sorted-a@v1": ["A.Created", "A.Updated"],
    "events.sorted-b@v1": ["B.Created", "B.Updated"],
  });
  assertEquals(first.CONTRACT.eventConsumers?.ingest.self, [
    "Self.A",
    "Self.Z",
  ]);
  assertEquals(first.CONTRACT_DIGEST, second.CONTRACT_DIGEST);
});

Deno.test("defineServiceContract validates event consumer group uses", () => {
  const source = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "events.validation-source@v1",
      displayName: "Event Validation Source",
      description: "Expose events for event consumer validation tests.",
      events: {
        "Source.Created": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        () => ({
          id: "events.empty-group@v1",
          displayName: "Empty Group",
          description: "Reject event consumers without uses or self.",
          eventConsumers: {
            ingest: {},
          },
        }),
      ),
    Error,
    "must declare at least one dependency or self event",
  );

  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        () => ({
          id: "events.unknown-use@v1",
          displayName: "Unknown Use",
          description: "Reject event consumers with unknown uses.",
          eventConsumers: {
            ingest: { uses: { source: ["Source.Created"] } },
          },
        }),
      ),
    Error,
    "references unknown use 'source'",
  );

  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        () => ({
          id: "events.not-subscribed@v1",
          displayName: "Not Subscribed",
          description: "Reject event consumers outside subscribed events.",
          uses: [source.SourceCreated.subscribe],
          eventConsumers: {
            ingest: {
              uses: { "events.validation-source@v1": ["Source.Updated"] },
            },
          },
        }),
      ),
    Error,
    "does not subscribe to",
  );
});

Deno.test("defineServiceContract rejects unknown owned event consumer refs", () => {
  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        (ref) => ({
          id: "events.unknown-owned@v1",
          displayName: "Unknown Owned Event",
          description:
            "Reject event consumers that reference unknown owned events.",
          events: {
            "Self.Created": {
              version: "v1",
              event: ref.schema("StringValue"),
            },
          },
          eventConsumers: {
            ingest: { self: ["Self.Missing"] },
          },
        }),
      ),
    Error,
    "references unknown owned event 'Self.Missing'",
  );
});

Deno.test("defineServiceContract emits store resources with defaults", () => {
  const contract = defineServiceContract({}, () => ({
    id: "store.example@v1",
    displayName: "Store Example",
    description: "Expose store resource declarations in emitted manifests.",
    uses: [store({
      uploads: {
        purpose: "Temporary uploaded files awaiting processing",
        maxObjectBytes: 100 * 1024 * 1024,
      },
    })],
  }));

  assertEquals(contract.CONTRACT.resources?.store?.uploads, {
    purpose: "Temporary uploaded files awaiting processing",
    required: true,
    ttlMs: 0,
    maxObjectBytes: 100 * 1024 * 1024,
  });
});

Deno.test("locally defined contracts can be reused as dependencies", () => {
  const audit = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.audit@v1",
      displayName: "Audit",
      description: "Expose audit events for dependency reuse tests.",
      events: {
        "Audit.Recorded": {
          version: "v1",
          event: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  const dashboard = defineAppContract(() => ({
    id: "trellis.dashboard@v1",
    displayName: "Dashboard",
    description: "Reuse locally defined contracts as dependencies in tests.",
    uses: [audit.AuditRecorded.subscribe],
  }));

  assertEquals(
    requiredUses(dashboard.CONTRACT.uses)?.["trellis.audit@v1"].contract,
    "trellis.audit@v1",
  );
  assertEquals(
    getContractRuntime(dashboard).usedApi.events["Audit.Recorded"].subject,
    "events.v1.Audit.Recorded",
  );
  assertEquals(
    getContractRuntime(dashboard).api.events["Audit.Recorded"].subject,
    "events.v1.Audit.Recorded",
  );
});

Deno.test("defineServiceContract emits owned and used operations", () => {
  const billingCapabilities = {
    "refund": {
      displayName: "Refund billing",
      description: "Start billing refunds.",
    },
    "read": {
      displayName: "Read billing",
      description: "Read billing operation status.",
    },
    "cancel": {
      displayName: "Cancel billing",
      description: "Cancel billing operations.",
    },
    "control": {
      displayName: "Control billing",
      description: "Control billing operations.",
    },
  } as const;
  const billingSchemas = {
    ...baseSchemas,
    SelectReason: Type.Object({ reason: Type.String() }),
  } as const;
  const billing = defineServiceContract(
    { schemas: billingSchemas },
    () => ({
      id: "trellis.billing@v1",
      displayName: "Billing",
      description: "Expose billing operations for source emission tests.",
      capabilities: billingCapabilities,
      operations: {
        "Billing.Refund": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          progress: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          update: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
          capabilities: {
            call: ["refund"],
            observe: ["read"],
            cancel: ["cancel"],
            control: ["control"],
          },
          signals: {
            selectReason: {
              input: schemaRef<typeof billingSchemas, "SelectReason">(
                "SelectReason",
              ),
            },
          },
          cancel: true,
        },
      },
    }),
  );

  const payments = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.payments@v1",
      displayName: "Payments",
      description: "Use billing operations in source emission tests.",
      uses: [billing.BillingRefund],
      operations: {
        "Payments.Capture": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "StringValue">("StringValue"),
        },
      },
    }),
  );

  assertEquals(payments.CONTRACT.operations, {
    "Payments.Capture": {
      version: "v1",
      subject: "operations.v1.Payments.Capture",
      input: { schema: "Empty" },
      output: { schema: "StringValue" },
    },
  });
  assertEquals(requiredUses(payments.CONTRACT.uses)?.["trellis.billing@v1"], {
    contract: "trellis.billing@v1",
    operations: {
      call: ["Billing.Refund"],
      cancel: ["Billing.Refund"],
      control: ["Billing.Refund"],
    },
  });
  assertEquals(
    getContractRuntime(payments).ownedApi.operations["Payments.Capture"]
      .subject,
    "operations.v1.Payments.Capture",
  );
  assertEquals(
    getContractRuntime(payments).usedApi.operations["Billing.Refund"].subject,
    "operations.v1.Billing.Refund",
  );
  const refundOperation = billing.CONTRACT.operations?.["Billing.Refund"];
  assertEquals(refundOperation?.update, { schema: "StringValue" });
  assertEquals(refundOperation?.signals, {
    selectReason: { input: { schema: "SelectReason" } },
  });
  assertEquals(refundOperation?.capabilities?.control, [
    globalCapabilityName("trellis.billing@v1", "control"),
  ]);
  assertEquals(
    unwrapSchema(
      getContractRuntime(billing).ownedApi.operations["Billing.Refund"].signals
        ?.selectReason.input ?? {},
    ),
    {
      properties: { reason: { type: "string" } },
      required: ["reason"],
      type: "object",
    },
  );
  assertEquals(
    getContractRuntime(billing).ownedApi.operations["Billing.Refund"]
      .controlCapabilities,
    ["trellis.billing::control"] as const,
  );
});

Deno.test("defineServiceContract emits transfer-capable operations", () => {
  const fileSchemas = {
    UploadInput: Type.Object({
      key: Type.String(),
    }),
  } as const;
  const files = defineServiceContract(
    { schemas: fileSchemas },
    () => ({
      id: "trellis.files@v1",
      displayName: "Files",
      description:
        "Expose transfer-capable operations for source emission tests.",
      uses: [store({
        uploads: {
          purpose: "Temporary uploads",
          ttlMs: 60_000,
          maxObjectBytes: 1024,
        },
      })],
      operations: {
        "Demo.Files.Upload": {
          version: "v1",
          input: schemaRef<typeof fileSchemas, "UploadInput">("UploadInput"),
          output: schemaRef<typeof fileSchemas, "UploadInput">("UploadInput"),
          transfer: {
            direction: "send",
            store: "uploads",
            key: "/key",
            expiresInMs: 60_000,
          },
        },
      },
    }),
  );

  assertEquals(files.CONTRACT.operations?.["Demo.Files.Upload"], {
    version: "v1",
    subject: "operations.v1.Demo.Files.Upload",
    input: { schema: "UploadInput" },
    output: { schema: "UploadInput" },
    transfer: {
      direction: "send",
      store: "uploads",
      key: "/key",
      expiresInMs: 60_000,
    },
  });
  assertEquals(
    getContractRuntime(files).ownedApi.operations["Demo.Files.Upload"].transfer,
    {
      direction: "send",
      store: "uploads",
      key: "/key",
      expiresInMs: 60_000,
    },
  );
});

Deno.test("defineServiceContract rejects duplicate logical keys across used and owned operations", () => {
  const billing = defineServiceContract(
    { schemas: baseSchemas },
    () => ({
      id: "trellis.billing@v1",
      displayName: "Billing",
      description: "Expose billing operations in duplicate-key tests.",
      operations: {
        "Billing.Refund": {
          version: "v1",
          input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
          output: schemaRef<typeof baseSchemas, "Empty">("Empty"),
        },
      },
    }),
  );

  assertThrows(
    () =>
      defineServiceContract(
        { schemas: baseSchemas },
        () => ({
          id: "duplicate.operations@v1",
          displayName: "Duplicate Operations",
          description: "Trigger duplicate logical operation key validation.",
          uses: [billing.BillingRefund],
          operations: {
            "Billing.Refund": {
              version: "v1",
              input: schemaRef<typeof baseSchemas, "Empty">("Empty"),
              output: schemaRef<typeof baseSchemas, "Empty">("Empty"),
            },
          },
        }),
      ),
    Error,
    "Duplicate operations key 'Billing.Refund'",
  );
});
