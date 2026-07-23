import { digestJson, isJsonValue, type JsonValue } from "./canonical.ts";
import { CONTRACT_RUNTIME, type ContractRuntime } from "./contract_runtime.ts";
import { actionSource } from "./descriptors.ts";

type JsonObject = Record<string, JsonValue>;

/** Canonical protocol artifacts compiled from a TypeScript contract manifest. */
export type CompiledProtocolArtifacts = {
  api: JsonObject;
  participant: JsonObject;
  referencedApis: readonly JsonObject[];
};

function object(value: JsonValue | undefined): JsonObject | undefined {
  return value !== null && typeof value === "object" && !Array.isArray(value)
    ? value as JsonObject
    : undefined;
}

function checkedObject(value: Readonly<Record<string, unknown>>): JsonObject {
  if (!Object.values(value).every(isJsonValue)) {
    throw new Error("Contract manifest must contain only JSON values");
  }
  return value as JsonObject;
}

type ContractInput = Readonly<Record<string, unknown>> & {
  readonly CONTRACT?: Readonly<Record<string, unknown>>;
  readonly CONTRACT_DIGEST?: string;
  readonly [CONTRACT_RUNTIME]?: ContractRuntime;
};

function sourceManifest(contract: ContractInput): JsonObject {
  return checkedObject(contract.CONTRACT ?? contract);
}

async function compileReferencedApi(
  source: Readonly<Record<string, unknown>> | {
    readonly artifact: Readonly<Record<string, unknown>>;
    readonly digest: string;
  },
): Promise<JsonObject> {
  const artifact = Reflect.get(source, "artifact");
  const digest = Reflect.get(source, "digest");
  const wrapped = artifact !== null && typeof artifact === "object" &&
    !Array.isArray(artifact) && typeof digest === "string";
  const manifest = checkedObject(
    wrapped ? artifact as Readonly<Record<string, unknown>> : source,
  );
  if (manifest.format === "trellis.api.v1") {
    return structuredClone(manifest);
  }
  return await compileApi(manifest, wrapped ? digest as string : undefined);
}

function normalizeSchema(value: JsonValue): void {
  if (Array.isArray(value)) {
    value.forEach(normalizeSchema);
    return;
  }
  const record = object(value);
  if (!record) return;
  if ("patternProperties" in record) {
    delete record.patternProperties;
    record.additionalProperties = true;
  }
  Object.values(record).forEach(normalizeSchema);
}

function copy(source: JsonObject, target: JsonObject, key: string): void {
  const value = source[key];
  const record = object(value);
  if (
    value !== undefined && value !== null &&
    !(record && Object.keys(record).length === 0)
  ) target[key] = structuredClone(value);
}

async function compileApi(
  contract: JsonObject,
  contractDigest?: string,
): Promise<JsonObject> {
  const api: JsonObject = { format: "trellis.api.v1" };
  for (
    const field of [
      "id",
      "displayName",
      "description",
      "docs",
      "schemas",
      "exports",
    ]
  ) copy(contract, api, field);
  Object.values(object(api.schemas) ?? {}).forEach(normalizeSchema);
  const schemas = object(api.schemas) ?? {};
  if ("TrellisContractArtifactIdentity" in schemas) {
    throw new Error("TrellisContractArtifactIdentity is reserved");
  }
  schemas.TrellisContractArtifactIdentity = {
    type: "string",
    const: contractDigest ?? (await digestJson(contract)).digest,
  };
  api.schemas = schemas;

  const capabilityAllows: Record<string, JsonValue[]> = {};
  const addCapability = (
    capability: JsonValue,
    action: string,
    target: JsonObject,
  ) => {
    if (typeof capability !== "string") return;
    (capabilityAllows[capability] ??= []).push({ action, target });
  };
  const apiId = String(contract.id);
  for (
    const [section, actionMap] of [
      ["rpc", { call: "call" }],
      ["operations", {
        call: "invoke",
        observe: "observe",
        cancel: "cancel",
      }],
      ["events", { publish: "publish", subscribe: "subscribe" }],
      ["feeds", { subscribe: "subscribe" }],
    ] as const
  ) {
    for (
      const [name, value] of Object.entries(object(contract[section]) ?? {})
        .sort(([left], [right]) => left.localeCompare(right))
    ) {
      const definition = object(value);
      const capabilities = object(definition?.capabilities);
      for (const [direction, action] of Object.entries(actionMap)) {
        for (
          const capability of capabilities?.[direction] as JsonValue[] ?? []
        ) {
          addCapability(capability, action, {
            kind: "apiSurface",
            api: apiId,
            surface: section === "operations"
              ? "operation"
              : section === "events"
              ? "event"
              : section === "feeds"
              ? "feed"
              : "rpc",
            name,
          });
        }
      }
      if (section === "operations") {
        for (const capability of capabilities?.control as JsonValue[] ?? []) {
          for (
            const signal of Object.keys(object(definition?.signals) ?? {})
              .sort()
          ) {
            addCapability(capability, "control", {
              kind: "operationSignal",
              api: apiId,
              operation: name,
              signal,
            });
          }
        }
      }
    }
  }
  if (Object.keys(capabilityAllows).length > 0) {
    api.capabilities = Object.fromEntries(
      Object.entries(capabilityAllows).sort(([left], [right]) =>
        left.localeCompare(right)
      ).map(([name, allows]) => {
        const keyed = new Map(
          allows.map((permission) => {
            const atom = object(permission)!;
            const target = object(atom.target)!;
            const kind = String(target.kind);
            const key = kind === "apiSurface"
              ? [
                kind,
                target.api,
                target.surface,
                target.name,
                "",
                atom.action,
              ]
              : [
                kind,
                target.api,
                "operation",
                target.operation,
                target.signal,
                atom.action,
              ];
            return [key.join("\0"), permission] as const;
          }),
        );
        return [name, {
          allows: [...keyed.entries()].sort().map(([, value]) => value),
        }];
      }),
    );
  }

  for (const section of ["rpc", "operations", "events", "feeds", "state"]) {
    const definitions = object(contract[section]);
    if (!definitions || Object.keys(definitions).length === 0) continue;
    const lowered = structuredClone(definitions);
    for (const value of Object.values(lowered)) {
      const definition = object(value);
      if (!definition) continue;
      delete definition.subject;
      delete definition.capabilities;
      if (Array.isArray(definition.errors)) {
        definition.errors = [
          ...new Set(
            definition.errors.map((error) => object(error)?.type ?? error),
          ),
        ].sort();
        if (definition.errors.length === 0) delete definition.errors;
      }
      const transfer = object(definition.transfer);
      if (transfer) definition.transfer = { direction: transfer.direction };
      if (definition.internal === false) delete definition.internal;
      if (definition.cancel === false) delete definition.cancel;
      if (Object.keys(object(definition.signals) ?? {}).length === 0) {
        delete definition.signals;
      }
      if (Array.isArray(definition.params) && definition.params.length === 0) {
        delete definition.params;
      }
      if (definition.class === "domain") delete definition.class;
      if (
        Object.keys(object(definition.acceptedVersions) ?? {}).length === 0
      ) delete definition.acceptedVersions;
    }
    api[section] = lowered;
  }
  const errors = structuredClone(object(contract.errors) ?? {});
  for (const value of Object.values(errors)) {
    const definition = object(value);
    if (definition) delete definition.type;
  }
  for (const section of ["rpc", "operations"]) {
    for (const value of Object.values(object(api[section]) ?? {})) {
      const definition = object(value);
      const referenced = Array.isArray(definition?.errors)
        ? definition.errors
        : [];
      for (const error of referenced) {
        if (typeof error === "string" && !(error in errors)) errors[error] = {};
      }
    }
  }
  if (Object.keys(errors).length > 0) api.errors = errors;
  return api;
}

async function apiDigest(api: JsonObject): Promise<string> {
  const projection: JsonObject = { format: api.format, id: api.id };
  for (const field of ["schemas", "exports", "capabilities"]) {
    copy(api, projection, field);
  }
  for (
    const section of [
      "errors",
      "rpc",
      "operations",
      "events",
      "feeds",
      "state",
    ]
  ) {
    const definitions = object(api[section]);
    if (!definitions || Object.keys(definitions).length === 0) continue;
    const lowered = structuredClone(definitions);
    for (const value of Object.values(lowered)) {
      const definition = object(value);
      if (!definition) continue;
      delete definition.docs;
      if (section === "operations") {
        for (const signal of Object.values(object(definition.signals) ?? {})) {
          const signalDefinition = object(signal);
          if (signalDefinition) delete signalDefinition.docs;
        }
      }
    }
    projection[section] = lowered;
  }
  return (await digestJson(projection)).digest;
}

/**
 * Compile a contract manifest into canonical API and participant artifacts.
 * Referenced APIs are keyed by canonical API ID.
 */
export async function compileProtocolArtifacts(
  contract: ContractInput,
  referencedApis: Readonly<Record<string, JsonObject>> = {},
): Promise<CompiledProtocolArtifacts> {
  const source = sourceManifest(contract);
  const api = await compileApi(source, contract.CONTRACT_DIGEST);
  const apis: Record<string, JsonObject> = {
    ...referencedApis,
    [String(api.id)]: api,
  };
  const discoveredApis: JsonObject[] = [];
  for (const selected of contract[CONTRACT_RUNTIME]?.actions ?? []) {
    const dependencySource = actionSource(selected.action);
    if (!dependencySource) continue;
    const dependencyApi = await compileReferencedApi(dependencySource);
    apis[String(dependencyApi.id)] = dependencyApi;
    discoveredApis.push(dependencyApi);
  }
  const participant: JsonObject = { format: "trellis.participant.v1" };
  for (
    const field of [
      "id",
      "displayName",
      "description",
      "docs",
      "kind",
      "schemas",
    ]
  ) copy(source, participant, field);
  Object.values(object(participant.schemas) ?? {}).forEach(normalizeSchema);
  if (
    ["rpc", "operations", "events", "feeds", "state"].some((section) =>
      Object.keys(object(source[section]) ?? {}).length > 0
    )
  ) {
    const operationTransfers: JsonObject = {};
    for (
      const [name, value] of Object.entries(object(source.operations) ?? {})
    ) {
      const transfer = object(object(value)?.transfer);
      if (transfer?.direction !== "send") continue;
      const mapping = structuredClone(transfer);
      delete mapping.direction;
      operationTransfers[name] = mapping;
    }
    participant.implements = {
      self: {
        api: api.id,
        apiDigest: await apiDigest(api),
        ...(Object.keys(operationTransfers).length > 0
          ? { operationTransfers }
          : {}),
      },
    };
  }

  const contractUses = object(source.uses);
  const uses: JsonObject = {};
  for (const group of ["required", "optional"]) {
    const references = object(contractUses?.[group]);
    if (!references) continue;
    const lowered: JsonObject = {};
    for (const [alias, value] of Object.entries(references)) {
      const reference = object(value);
      const apiId = reference?.contract;
      if (typeof apiId !== "string" || !apis[apiId]) {
        throw new Error(
          `Referenced API artifact '${String(apiId)}' is required`,
        );
      }
      const referencedApi = apis[apiId];
      const used: JsonObject = {
        api: apiId,
        apiDigest: await apiDigest(referencedApi),
      };
      copy(reference!, used, "rpc");
      const operations = object(reference!.operations);
      if (operations) {
        const calls = Array.isArray(operations.call) ? operations.call : [];
        const cancels = Array.isArray(operations.cancel)
          ? operations.cancel
          : [];
        const controls = Array.isArray(operations.control)
          ? operations.control
          : [];
        const definitions = object(referencedApi.operations) ?? {};
        used.operations = {
          invoke: calls,
          observe: calls,
          cancel: cancels.filter((name) =>
            typeof name === "string" &&
            object(definitions[name])?.cancel === true
          ),
          ...(controls.length > 0
            ? {
              control: Object.fromEntries(controls.map((name) => [
                name,
                Object.keys(object(definitions[String(name)])?.signals ?? {})
                  .sort(),
              ])),
            }
            : {}),
        };
      }
      copy(reference!, used, "events");
      copy(reference!, used, "feeds");
      lowered[alias] = used;
    }
    if (Object.keys(lowered).length > 0) uses[group] = lowered;
  }
  if (Object.keys(uses).length > 0) participant.uses = uses;
  copy(source, participant, "resources");
  const jobs = object(source.jobs);
  if (jobs) {
    participant.jobQueues = structuredClone(object(jobs.queues) ?? jobs);
  }
  const consumers = object(source.eventConsumers);
  if (consumers) {
    participant.eventConsumers = Object.fromEntries(
      Object.entries(consumers).map(([name, value]) => {
        const consumer = structuredClone(object(value) ?? {});
        const events = structuredClone(object(consumer.uses) ?? {});
        if (consumer.self !== undefined) events.self = consumer.self;
        delete consumer.uses;
        delete consumer.self;
        consumer.events = events;
        return [name, consumer];
      }),
    );
  }
  return { api, participant, referencedApis: discoveredApis };
}
