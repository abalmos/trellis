import type { BaseError } from "@qlever-llc/result";
import type { Codec } from "../generated.ts";
import type {
  EventDesc,
  FeedDesc,
  OperationDesc,
  PermissionAtom,
  RPCDesc,
  RuntimeApi,
  RuntimeRpcErrorDesc,
} from "./api.ts";
import { lowerCamelSurfaceName, pascalSurfaceName } from "./surface_names.ts";
import type { TrellisAvailability } from "../connection.ts";

export type GeneratedActionDescriptor = Readonly<{
  kind: "rpc" | "operation" | "event" | "feed";
  descriptorName: `${"rpc" | "operation" | "event" | "feed"}:${string}`;
  input?: Codec<unknown>;
  output?: Codec<unknown>;
  payload?: Codec<unknown>;
  event?: Codec<unknown>;
  progress?: Codec<unknown>;
  errors?: readonly RuntimeRpcErrorClass[];
  signals?: Readonly<Record<string, Codec<unknown>>>;
  parameters?: readonly (readonly string[])[];
  upload?: boolean;
  download?: boolean;
}>;

export type GeneratedApiDescriptor = Readonly<{
  identity: string;
  actions: Readonly<Record<string, GeneratedActionDescriptor>>;
}>;

export type GeneratedActionSelection = Readonly<{
  api: GeneratedApiDescriptor;
  actions: readonly Readonly<{
    descriptorName: string;
    direction: "call" | "invoke" | "publish" | "subscribe";
  }>[];
  optionalCapabilities: readonly string[];
}>;

export type GeneratedResourceDescriptor =
  & Readonly<Record<string, unknown>>
  & Readonly<{
    kind: "state" | "kv" | "store" | "job" | "consumer";
    availability: "required" | "optional";
  }>;

/** Generated participant descriptor accepted by Trellis runtimes. */
export type GeneratedParticipant = Readonly<{
  kind: "service" | "device" | "app" | "agent";
  identity: string;
  path: string;
  implements: readonly GeneratedApiDescriptor[];
  uses: readonly GeneratedActionSelection[];
  resources: Readonly<Record<string, GeneratedResourceDescriptor>>;
  companion?: Readonly<{
    participant: GeneratedParticipant;
    availability: "required" | "optional";
  }>;
  packageEvidence: unknown;
}>;

type RuntimeRpcErrorClass = Readonly<{
  type: string;
  fromSerializable(data: unknown): BaseError;
}>;

export type RuntimeSelectedAction = Readonly<{
  api: GeneratedApiDescriptor;
  descriptor: GeneratedActionDescriptor;
  direction: "call" | "invoke" | "publish" | "subscribe";
  name: string;
  connectedName: string;
  optional: boolean;
  optionalCapabilities: readonly string[];
}>;

type RuntimeStateDescriptor = Readonly<{
  kind: "value";
  value: unknown;
  schema: unknown;
  stateVersion: string;
  acceptedVersions: Record<string, unknown>;
}>;

export type ParticipantRuntime = Readonly<{
  ownedApi: RuntimeApi;
  usedApi: RuntimeApi;
  api: RuntimeApi;
  actions: readonly RuntimeSelectedAction[];
  state: Readonly<Record<string, RuntimeStateDescriptor>>;
  kv: Readonly<Record<string, Readonly<Record<string, unknown>>>>;
  jobs: Readonly<Record<string, Readonly<Record<string, unknown>>>>;
  eventConsumers: Readonly<Record<string, Readonly<Record<string, unknown>>>>;
}>;

function actionName(descriptorName: string): string {
  return descriptorName.slice(descriptorName.indexOf(":") + 1);
}

function permission(
  apiId: string,
  descriptor: GeneratedActionDescriptor,
  action: PermissionAtom["action"],
): PermissionAtom {
  const match = /^(.+)@v([1-9][0-9]*)$/.exec(apiId);
  if (!match) throw new Error(`Invalid generated API identity '${apiId}'`);
  return {
    apiId: match[1],
    apiVersion: `v${match[2]}` as `v${number}`,
    surfaceKind: descriptor.kind,
    surfaceName: actionName(descriptor.descriptorName),
    action,
  };
}

function runtimeErrors(
  errors: readonly RuntimeRpcErrorClass[] | undefined,
): readonly RuntimeRpcErrorDesc[] | undefined {
  return errors?.map((error) => ({
    type: error.type,
    fromSerializable: error.fromSerializable,
  }));
}

function subject(descriptor: GeneratedActionDescriptor): string {
  const name = actionName(descriptor.descriptorName);
  const prefix = descriptor.kind === "event" ? "events" : descriptor.kind;
  const parameters =
    descriptor.parameters?.map((path) => `.{/${path.join("/")}}`).join("") ??
      "";
  return `${prefix}.v1.${name}${parameters}`;
}

function runtimeDescriptor(
  api: GeneratedApiDescriptor,
  descriptor: GeneratedActionDescriptor,
): RPCDesc | OperationDesc | EventDesc | FeedDesc {
  const transportSubject = subject(descriptor);
  const errors = descriptor.errors?.map((error) => error.type);
  const declaredErrors = runtimeErrors(descriptor.errors);
  switch (descriptor.kind) {
    case "rpc":
      return {
        subject: transportSubject,
        input: descriptor.input as Codec<unknown>,
        output: descriptor.output as Codec<unknown>,
        permission: permission(api.identity, descriptor, "call"),
        callerCapabilities: [],
        ...(descriptor.download ? { transfer: { direction: "receive" } } : {}),
        ...(errors ? { errors, declaredErrorTypes: errors } : {}),
        ...(declaredErrors ? { runtimeErrors: declaredErrors } : {}),
      };
    case "operation":
      return {
        subject: transportSubject,
        input: descriptor.input as Codec<unknown>,
        output: descriptor.output,
        progress: descriptor.progress,
        update: descriptor.progress,
        permissions: {
          invoke: permission(api.identity, descriptor, "invoke"),
          observe: permission(api.identity, descriptor, "observe"),
          cancel: permission(api.identity, descriptor, "cancel"),
          control: Object.fromEntries(
            Object.keys(descriptor.signals ?? {}).map((name) => [
              name,
              permission(api.identity, descriptor, "control"),
            ]),
          ),
        },
        signals: Object.fromEntries(
          Object.entries(descriptor.signals ?? {}).map(([name, input]) => [
            name,
            { input },
          ]),
        ),
        callerCapabilities: [],
        observeCapabilities: [],
        cancelCapabilities: [],
        controlCapabilities: [],
        ...(descriptor.upload ? { transfer: { direction: "send" } } : {}),
        ...(errors ? { errors, declaredErrorTypes: errors } : {}),
        ...(declaredErrors ? { runtimeErrors: declaredErrors } : {}),
      };
    case "event":
      return {
        subject: transportSubject,
        params: descriptor.parameters?.map((path) =>
          `/${path.join("/")}` as `/${string}`
        ),
        event: descriptor.payload as Codec<unknown>,
        publishPermission: permission(api.identity, descriptor, "publish"),
        subscribePermission: permission(api.identity, descriptor, "subscribe"),
        publishCapabilities: [],
        subscribeCapabilities: [],
      };
    case "feed":
      return {
        subject: transportSubject,
        input: descriptor.input as Codec<unknown>,
        event: descriptor.event as Codec<unknown>,
        permission: permission(api.identity, descriptor, "subscribe"),
        subscribeCapabilities: [],
      };
  }
}

function addAction(
  target: RuntimeApi,
  api: GeneratedApiDescriptor,
  descriptor: GeneratedActionDescriptor,
): void {
  const name = actionName(descriptor.descriptorName);
  const runtime = runtimeDescriptor(api, descriptor);
  if (descriptor.kind === "rpc") target.rpc[name] = runtime as RPCDesc;
  else if (descriptor.kind === "operation") {
    target.operations[name] = runtime as OperationDesc;
  } else if (descriptor.kind === "event") {
    target.events[name] = runtime as EventDesc;
  } else {
    const feeds = target.feeds ?? {};
    feeds[name] = runtime as FeedDesc;
    target.feeds = feeds;
  }
}

function emptyApi(): RuntimeApi {
  return { rpc: {}, operations: {}, events: {}, feeds: {}, subjects: {} };
}

/** Projects generated descriptors into the current pre-WO-05 transport runtime. */
export function getParticipantRuntime(
  participant: GeneratedParticipant,
): ParticipantRuntime {
  const ownedApi = emptyApi();
  const usedApi = emptyApi();
  const actions: RuntimeSelectedAction[] = [];
  const state: Record<string, RuntimeStateDescriptor> = {};
  const kv: Record<string, Readonly<Record<string, unknown>>> = {};
  const jobs: Record<string, Readonly<Record<string, unknown>>> = {};
  const eventConsumers: Record<string, Readonly<Record<string, unknown>>> = {};

  for (const api of participant.implements) {
    for (const descriptor of Object.values(api.actions)) {
      addAction(ownedApi, api, descriptor);
    }
  }
  for (const selection of participant.uses) {
    for (const selected of selection.actions) {
      const descriptor = selection.api.actions[selected.descriptorName];
      if (!descriptor) {
        throw new Error(
          `Generated action '${selected.descriptorName}' is absent from '${selection.api.identity}'`,
        );
      }
      addAction(usedApi, selection.api, descriptor);
      const name = actionName(descriptor.descriptorName);
      actions.push({
        api: selection.api,
        descriptor,
        direction: selected.direction,
        name,
        connectedName: selected.direction === "subscribe" &&
            descriptor.kind === "event"
          ? `on${pascalSurfaceName(name)}`
          : lowerCamelSurfaceName(name),
        optional: selection.optionalCapabilities.length > 0,
        optionalCapabilities: selection.optionalCapabilities,
      });
    }
  }

  for (const [name, resource] of Object.entries(participant.resources)) {
    if (resource.kind === "state") {
      state[name] = {
        kind: "value",
        value: undefined,
        schema: resource.codec,
        stateVersion: String(resource.version),
        acceptedVersions: resource.migrations &&
            typeof resource.migrations === "object" &&
            !Array.isArray(resource.migrations)
          ? resource.migrations as Record<string, unknown>
          : {},
      };
    } else if (resource.kind === "kv") {
      kv[name] = {
        required: resource.availability === "required",
        schema: resource.codec,
      };
    } else if (resource.kind === "job") {
      jobs[name] = {
        payload: resource.payload,
        update: resource.update,
        updateSchema: resource.update,
        result: resource.result,
      };
    } else if (resource.kind === "consumer") {
      const uses: Record<string, string[]> = {};
      for (
        const [api, event] of resource.events as readonly (
          readonly [string, string]
        )[]
      ) {
        const events = uses[api] ?? [];
        events.push(event);
        uses[api] = events;
      }
      eventConsumers[name] = {
        uses,
        replay: resource.replay,
        ordering: Number(resource.concurrency) > 1 ? "parallel" : "strict",
      };
    }
  }

  return {
    ownedApi,
    usedApi,
    api: {
      rpc: { ...ownedApi.rpc, ...usedApi.rpc },
      operations: { ...ownedApi.operations, ...usedApi.operations },
      events: { ...ownedApi.events, ...usedApi.events },
      feeds: { ...ownedApi.feeds, ...usedApi.feeds },
      subjects: {},
    },
    actions,
    state,
    kv,
    jobs,
    eventConsumers,
  };
}

/** Projects installed API and resource bindings into one immutable participant snapshot. */
export function participantAvailability(
  participant: GeneratedParticipant,
  apiBindings: Readonly<Record<string, unknown>>,
  resourceBindings: Readonly<{
    kv?: Readonly<Record<string, unknown>>;
    store?: Readonly<Record<string, unknown>>;
    jobs?: Readonly<{ queues: Readonly<Record<string, unknown>> }>;
    eventConsumers?: Readonly<Record<string, unknown>>;
  }>,
): TrellisAvailability {
  const capabilities: Record<string, boolean> = {};
  for (const selection of participant.uses) {
    const available = selection.api.identity in apiBindings;
    for (const capability of selection.optionalCapabilities) {
      capabilities[capability] = available;
    }
  }

  const resources: Record<string, boolean> = {};
  for (const [name, descriptor] of Object.entries(participant.resources)) {
    if (descriptor.availability !== "optional") continue;
    switch (descriptor.kind) {
      case "state":
        resources[name] = false;
        break;
      case "kv":
        resources[name] = resourceBindings.kv?.[name] !== undefined;
        break;
      case "store":
        resources[name] = resourceBindings.store?.[name] !== undefined;
        break;
      case "job":
        resources[name] = resourceBindings.jobs?.queues[name] !== undefined;
        break;
      case "consumer":
        resources[name] = resourceBindings.eventConsumers?.[name] !== undefined;
        break;
    }
  }

  return Object.freeze({
    capabilities: Object.freeze(capabilities),
    resources: Object.freeze(resources),
  });
}

/** Returns the proof-bound package evidence envelope without interpreting source. */
export function participantEvidence(participant: GeneratedParticipant): {
  packageEvidence: unknown;
  participantPath: string;
  packageDigest: string;
} {
  const evidence = participant.packageEvidence;
  const packageDigest = evidence && typeof evidence === "object"
    ? Reflect.get(evidence, "rootDigest")
    : undefined;
  if (typeof packageDigest !== "string" || packageDigest.length === 0) {
    throw new Error("Generated participant has invalid package evidence");
  }
  return {
    packageEvidence: evidence,
    participantPath: participant.path,
    packageDigest,
  };
}
