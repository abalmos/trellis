import { resolveParticipantV1WasmSync } from "../auth/protocol_wasm.ts";
import {
  canonicalizeJson,
  isJsonValue,
  type JsonValue,
  sha256Base64urlSync,
} from "./canonical.ts";
import { CONTRACT_RUNTIME, type ContractRuntime } from "./contract_runtime.ts";
import { type ActionSource, actionSource } from "./descriptors.ts";

type JsonObject = Record<string, JsonValue>;

/** Native protocol artifacts presented during client and service bootstrap. */
export type NativeProtocolPresentation = {
  api: JsonObject;
  participant: JsonObject;
  referencedApis: readonly JsonObject[];
};

/** Native artifacts and identity values built from one authoring source. */
export type NativeProtocolArtifacts = NativeProtocolPresentation & {
  apiDigest: string;
  participantDigest: string;
  participantNeedsDigest: string;
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

export type NativeProtocolContract = {
  readonly CONTRACT_ID: string;
  readonly CONTRACT_DIGEST: string;
  readonly API: Readonly<Record<string, unknown>>;
  readonly API_DIGEST: string;
  readonly PARTICIPANT: Readonly<Record<string, unknown>>;
  readonly PARTICIPANT_NEEDS_DIGEST: string;
  readonly [CONTRACT_RUNTIME]: ContractRuntime;
};

function nativeApi(contract: NativeProtocolContract): JsonObject {
  if (!contract.API) {
    throw new Error("Defined contract is missing native API artifact");
  }
  return checkedObject(contract.API);
}

function nativeParticipant(contract: NativeProtocolContract): JsonObject {
  if (!contract.PARTICIPANT) {
    throw new Error("Defined contract is missing native participant artifact");
  }
  return checkedObject(contract.PARTICIPANT);
}

function compileReferencedApi(
  source: ActionSource,
): JsonObject {
  const api = checkedObject(source.api);
  if (api.format !== "trellis.api.v1") {
    throw new Error("Action source must contain a trellis.api.v1 artifact");
  }
  if (source.apiDigest !== apiDigest(api)) {
    throw new Error("Action source API digest does not match its artifact");
  }
  return structuredClone(api);
}

/** Validate and coalesce exact action-source API evidence by API identity. */
export function collectActionSources(
  sources: Iterable<ActionSource>,
): ReadonlyMap<string, ActionSource> {
  const collected = new Map<string, ActionSource>();
  for (const source of sources) {
    const api = compileReferencedApi(source);
    if (typeof api.id !== "string") {
      throw new Error("Action source API artifact is missing an id");
    }
    const existing = collected.get(api.id);
    if (existing) {
      if (existing.apiDigest !== source.apiDigest) {
        throw new Error(
          `Conflicting action source revisions for API '${api.id}'`,
        );
      }
      if (
        canonicalizeJson(checkedObject(existing.api)) !== canonicalizeJson(api)
      ) {
        throw new Error(
          `Conflicting action source artifacts for API '${api.id}'`,
        );
      }
      continue;
    }
    collected.set(api.id, { api, apiDigest: source.apiDigest });
  }
  return collected;
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
  ) {
    target[key] = record && !Array.isArray(value)
      ? structuredClone({ ...record })
      : structuredClone(value);
  }
}

function compileApi(contract: JsonObject): JsonObject {
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
  const declaredCapabilities = object(contract.capabilities) ?? {};
  const normalizeCapability = (name: string) => {
    if (name in declaredCapabilities) {
      return `${apiId.replace(/@v\d+$/, "")}::${name}`;
    }
    if (name === "admin" || name === "service" || name.includes("::")) {
      return name;
    }
    throw new Error(`undeclared local capability '${name}'`);
  };
  const capabilityAllows: Record<string, JsonValue[]> = {};
  const apiId = String(contract.id);
  for (const name of Object.keys(declaredCapabilities)) {
    capabilityAllows[normalizeCapability(name)] = [];
  }
  const addCapability = (
    capability: JsonValue,
    action: string,
    target: JsonObject,
  ) => {
    if (typeof capability !== "string") return;
    const normalized = normalizeCapability(capability);
    const allows = capabilityAllows[normalized] ?? [];
    allows.push({ action, target });
    capabilityAllows[normalized] = allows;
  };
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
        .sort(([left], [right]) => compareProtocolStrings(left, right))
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
        compareProtocolStrings(left, right)
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
  if (Object.keys(declaredCapabilities).length > 0) {
    api.consent = Object.fromEntries(
      Object.entries(declaredCapabilities).sort(([left], [right]) =>
        compareProtocolStrings(left, right)
      ).map(([name, value]) => {
        const metadata = object(value) ?? {};
        return [normalizeCapability(name), {
          title: String(metadata.displayName ?? ""),
          description: String(metadata.description ?? ""),
          consequence: String(metadata.consequence ?? ""),
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

/** Computes the semantic digest for a normalized native API artifact. */
/** Return the semantic digest of a native API artifact. */
export function apiDigest(api: Readonly<Record<string, unknown>>): string {
  return apiDigestValue(checkedObject(api));
}

function apiDigestValue(api: JsonObject): string {
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
      delete definition.subject;
      if (section === "operations") {
        for (const signal of Object.values(object(definition.signals) ?? {})) {
          const signalDefinition = object(signal);
          if (signalDefinition) delete signalDefinition.docs;
        }
      }
    }
    projection[section] = lowered;
  }
  return sha256Base64urlSync(canonicalizeJson(projection));
}

function compileParticipant(
  source: JsonObject,
  api: JsonObject,
  apis: Readonly<Record<string, JsonObject>>,
  apiDigests: Readonly<Record<string, string>>,
): JsonObject {
  const apiId = api.id;
  if (typeof apiId !== "string") {
    throw new Error("Native API artifact is missing an id");
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
      "state",
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
        api: apiId,
        apiDigest: apiDigest(api),
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
      const referencedApiDigest = apiDigests[apiId];
      if (!referencedApiDigest) {
        throw new Error(
          `Referenced API artifact '${apiId}' is missing its digest`,
        );
      }
      const used: JsonObject = {
        api: apiId,
        apiDigest: referencedApiDigest,
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
  return participant;
}

function compareProtocolStrings(left: string, right: string): number {
  return left < right ? -1 : left > right ? 1 : 0;
}

/**
 * Returns native artifacts and exact dependency API evidence carried by a
 * defined contract without compiling or converting an authoring manifest.
 */
export function nativeProtocolPresentation(
  contract: NativeProtocolContract,
): NativeProtocolPresentation {
  const api = nativeApi(contract);
  const participant = nativeParticipant(contract);
  const discoveredApis = collectActionSources(
    (contract[CONTRACT_RUNTIME]?.actions ?? []).flatMap((selected) => {
      const source = actionSource(selected.action);
      return source ? [source] : [];
    }),
  );
  const ownedApiId = String(api.id);
  const apis = Object.fromEntries(
    [...discoveredApis].map(([id, source]) => [id, checkedObject(source.api)]),
  );
  if (
    apis[ownedApiId] &&
    canonicalizeJson(apis[ownedApiId]) !== canonicalizeJson(api)
  ) {
    throw new Error(`Conflicting API evidence for owned API '${ownedApiId}'`);
  }
  apis[ownedApiId] = api;
  const resolved = resolveParticipantV1WasmSync({ participant, apis });
  const resolvedApi = resolved.apiArtifacts[ownedApiId];
  if (!resolvedApi || canonicalizeJson(resolvedApi) !== canonicalizeJson(api)) {
    throw new Error(
      "Resolved owned API does not match the defined contract API",
    );
  }
  if (resolved.apiDigests[ownedApiId] !== contract.API_DIGEST) {
    throw new Error("Defined contract API digest does not match resolution");
  }
  if (resolved.participantDigest !== contract.CONTRACT_DIGEST) {
    throw new Error(
      "Defined contract participant digest does not match resolution",
    );
  }
  if (resolved.participantNeedsDigest !== contract.PARTICIPANT_NEEDS_DIGEST) {
    throw new Error(
      "Defined contract participant needs digest does not match resolution",
    );
  }
  return {
    api: resolvedApi,
    participant: resolved.participant,
    referencedApis: Object.entries(resolved.apiArtifacts)
      .filter(([id]) => id !== ownedApiId)
      .map(([, referencedApi]) => referencedApi),
  };
}

/** Build native API and participant artifacts directly from an authoring source. */
export function buildNativeProtocolArtifacts(
  source: Readonly<Record<string, unknown>>,
  referencedApis: Readonly<Record<string, ActionSource>> = {},
): NativeProtocolArtifacts {
  const contract = checkedObject(source);
  const api = compileApi(contract);
  const apiDigests: Record<string, string> = {};
  const collectedSources = collectActionSources(Object.values(referencedApis));
  const apis: Record<string, JsonObject> = Object.fromEntries(
    [...collectedSources].map(([id, source]) => {
      const artifact = compileReferencedApi(source);
      if (artifact.id !== id) {
        throw new Error(
          `Action source API map key '${id}' does not match artifact id`,
        );
      }
      apiDigests[id] = source.apiDigest;
      return [id, artifact];
    }),
  );
  Object.assign(apis, {
    [String(api.id)]: api,
  });
  const contractUses = object(contract.uses);
  for (
    const value of Object.values(object(contractUses?.required) ?? {})
      .concat(Object.values(object(contractUses?.optional) ?? {}))
  ) {
    const reference = object(value);
    const apiId = reference?.contract;
    if (typeof apiId !== "string" || !apis[apiId]) {
      throw new Error(`Referenced API artifact '${String(apiId)}' is required`);
    }
  }
  const participant = compileParticipant(contract, api, apis, apiDigests);
  const resolved = resolveParticipantV1WasmSync({
    participant,
    apis,
  });
  const apiId = String(api.id);
  const nativeApi = resolved.apiArtifacts[apiId];
  const nativeApiDigest = resolved.apiDigests[apiId];
  if (!nativeApi || !nativeApiDigest) {
    throw new Error(
      `Native API artifact '${apiId}' is missing from resolution`,
    );
  }
  return {
    api: nativeApi,
    participant: resolved.participant,
    referencedApis: Object.entries(resolved.apiArtifacts)
      .filter(([id]) => id !== apiId)
      .map(([, value]) => value),
    apiDigest: nativeApiDigest,
    participantDigest: resolved.participantDigest,
    participantNeedsDigest: resolved.participantNeedsDigest,
  };
}
