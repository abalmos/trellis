import type { AsyncResult, BaseError, Result } from "@qlever-llc/result";
import type {
  ActionDescriptor,
  DescriptorForAction,
  EventActions,
} from "./contract_support/descriptors.ts";
import {
  type ContractWithRuntime,
  getContractRuntime,
} from "./contract_support/contract_runtime.ts";
import type {
  EventDesc,
  FeedDesc,
  OperationDesc,
  RPCDesc,
} from "./contract_support/runtime.ts";
import type { CallerRuntime } from "./caller.ts";
import type {
  CONTRACT_JOBS_METADATA,
  CONTRACT_KV_METADATA,
  CONTRACT_STORE_METADATA,
} from "./contract_support/mod.ts";
import type { PreparedTrellisEvent } from "./session.ts";

export const PROVIDER_CALLER = Symbol("trellis.provider.caller");

export type ProviderCaller = Readonly<{
  rpc: Record<string, Record<string, (input: unknown) => unknown>>;
  operation: Record<
    string,
    Record<
      string,
      { input(input: unknown): unknown; resume(input: unknown): unknown }
    >
  >;
  feed: Record<string, Record<string, (input: unknown) => unknown>>;
  event: Record<
    string,
    Record<
      string,
      {
        publish(event: Record<string, unknown>): unknown;
        prepare(
          event: Record<string, unknown>,
        ): Result<PreparedTrellisEvent, BaseError>;
      }
    >
  >;
}>;

type PascalSurfaceName<TName extends string> = TName extends
  `${infer THead}.${infer TTail}`
  ? `${Capitalize<THead>}${PascalSurfaceName<TTail>}`
  : Capitalize<TName>;

type EventBody<TDescriptor> = TDescriptor extends EventDesc<infer TEvent>
  ? TEvent extends { readonly __trellisType?: infer TValue } ? TValue : unknown
  : unknown;

type DirectAction<TContract> = TContract[keyof TContract] extends infer TValue
  ? TValue extends ActionDescriptor ? TValue
  : TValue extends EventActions<infer TSubscribe, infer TPublish>
    ? TSubscribe | Exclude<TPublish, undefined>
  : never
  : never;
type SelectedAction<TContract> = TContract extends ContractWithRuntime<
  infer TAction
> ? TAction
  : never;
type ProviderCallerSurface<TContract> = Omit<
  CallerRuntime<TContract>,
  | "connection"
  | "state"
  | "wait"
  | Extract<keyof CallerRuntime<TContract>, `on${string}`>
>;

type SurfaceGroup<TName extends string> = TName extends
  `${infer THead}.${string}` ? Uncapitalize<THead>
  : never;
type SurfaceLeaf<TName extends string> = TName extends
  `${string}.${infer TTail}` ? Uncapitalize<PascalSurfaceName<TTail>>
  : never;
type HandlerKind<TAction extends ActionDescriptor> = TAction["kind"] extends
  "rpc" ? "rpc"
  : TAction["kind"] extends "operation" ? "operation"
  : TAction["kind"] extends "feed" ? "feed"
  : never;
type ReplaceHandlerClient<TRegistration, TClient> = TRegistration extends (
  ...args: infer TParams
) => infer TResult
  ? TParams extends [infer THandler]
    ? THandler extends (args: infer TArgs) => infer THandlerResult ?
        & ((
          handler: (
            args: TArgs extends { client: unknown }
              ? Omit<TArgs, "client"> & { client: TClient }
              : TArgs,
          ) => THandlerResult,
        ) => TResult)
        & (TRegistration extends {
          accept: infer TAccept;
          control: infer TControl;
        } ? { accept: TAccept; control: TControl }
          : {})
    : TRegistration
  : TRegistration
  : TRegistration;
type ServiceRegistration<
  TService,
  TAction extends ActionDescriptor,
  TClient,
> = TService extends { handle: infer THandle }
  ? HandlerKind<TAction> extends keyof THandle
    ? SurfaceGroup<TAction["name"]> extends keyof THandle[HandlerKind<TAction>]
      ? SurfaceLeaf<TAction["name"]> extends
        keyof THandle[HandlerKind<TAction>][SurfaceGroup<TAction["name"]>]
        ? ReplaceHandlerClient<
          THandle[HandlerKind<TAction>][SurfaceGroup<TAction["name"]>][
            SurfaceLeaf<TAction["name"]>
          ],
          TClient
        >
      : never
    : never
  : never
  : never;
type ServiceEventListener<TService, TAction extends ActionDescriptor> =
  TService extends { event: infer TEvent }
    ? SurfaceGroup<TAction["name"]> extends keyof TEvent
      ? SurfaceLeaf<TAction["name"]> extends
        keyof TEvent[SurfaceGroup<TAction["name"]>]
        ? TEvent[SurfaceGroup<TAction["name"]>][
          SurfaceLeaf<TAction["name"]>
        ] extends {
          listen: infer TListen;
        } ? TListen
        : never
      : never
    : never
    : never;

type ProviderActionRecord<TAction, TService, TClient> = TAction extends
  ActionDescriptor ? TAction["kind"] extends "event-subscribe" ? {
      readonly [K in TAction["connectedName"]]: ServiceEventListener<
        TService,
        TAction
      >;
    }
  : TAction["kind"] extends "event-publish" ? {
      readonly [K in `publish${PascalSurfaceName<TAction["name"]>}`]:
        & ((
          event: EventBody<DescriptorForAction<TAction>>,
        ) => AsyncResult<void, BaseError>)
        & {
          prepare(
            event: EventBody<DescriptorForAction<TAction>>,
          ): Result<PreparedTrellisEvent, BaseError>;
        };
    }
  : {
    readonly [K in `handle${PascalSurfaceName<TAction["name"]>}`]:
      ServiceRegistration<TService, TAction, TClient>;
  }
  : {};

type ProviderSelectedEventRecord<TAction, TService> = TAction extends
  ActionDescriptor<string, string, "event-subscribe"> ? {
    readonly [K in TAction["connectedName"]]: ServiceEventListener<
      TService,
      TAction
    >;
  }
  : {};

type ProviderOwnedEventPublisherRecord<TAction> = TAction extends
  ActionDescriptor<string, string, "event-publish"> ? {
    readonly [K in `publish${PascalSurfaceName<TAction["name"]>}`]:
      & ((
        event: EventBody<DescriptorForAction<TAction>>,
      ) => AsyncResult<void, BaseError>)
      & {
        prepare(
          event: EventBody<DescriptorForAction<TAction>>,
        ): Result<PreparedTrellisEvent, BaseError>;
      };
  }
  : {};

type UnionToIntersection<T> =
  (T extends unknown ? (value: T) => void : never) extends
    (value: infer TIntersection) => void ? TIntersection
    : never;

type ProviderBase<TService> = TService extends {
  readonly health: infer THealth;
  readonly connection: infer TConnection;
  readonly name: infer TName;
  readonly createSqlOutbox: infer TCreateSqlOutbox;
  readonly createTransfer: infer TCreateTransfer;
} ? {
    readonly health: THealth;
    readonly connection: TConnection;
    readonly name: TName;
    readonly createSqlOutbox: TCreateSqlOutbox;
    readonly createTransfer: TCreateTransfer;
    wait(): Promise<void>;
    stop(): Promise<void>;
  }
  : {};

type ContractKv<TContract> = TContract extends {
  readonly [CONTRACT_KV_METADATA]?: infer TMetadata;
} ? NonNullable<TMetadata>
  : {};
type ContractStore<TContract> = TContract extends {
  readonly [CONTRACT_STORE_METADATA]?: infer TMetadata;
} ? NonNullable<TMetadata>
  : {};
type ContractJobs<TContract> = TContract extends {
  readonly [CONTRACT_JOBS_METADATA]?: infer TMetadata;
} ? NonNullable<TMetadata>
  : {};
type SelectedServiceProperty<TService, TKey extends PropertyKey, TMetadata> =
  TKey extends keyof TService ? {
      readonly [K in TKey]: TService[TKey] extends Readonly<
        Record<PropertyKey, unknown>
      > ? {
          readonly [TAlias in keyof TMetadata & keyof TService[TKey]]:
            TService[TKey][TAlias];
        }
        : TService[TKey];
    }
    : {};
type ProviderFeatures<TContract, TService> =
  & SelectedServiceProperty<TService, "kv", ContractKv<TContract>>
  & SelectedServiceProperty<TService, "store", ContractStore<TContract>>
  & SelectedServiceProperty<TService, "jobs", ContractJobs<TContract>>;
type ProviderResources<TContract, TService> =
  & ProviderBase<TService>
  & ProviderFeatures<TContract, TService>;

/** Outbound actions and resources available inside provider handlers. */
export type ProviderHandlerClient<TContract, TService> =
  & ProviderResources<TContract, TService>
  & ProviderCallerSurface<TContract>
  & UnionToIntersection<
    ProviderOwnedEventPublisherRecord<DirectAction<TContract>>
  >;

/** Flat provider surface inferred from owned and selected action descriptors. */
export type ProviderRuntime<TContract, TService> =
  & ProviderResources<TContract, TService>
  & ProviderCallerSurface<TContract>
  & UnionToIntersection<
    ProviderActionRecord<
      DirectAction<TContract>,
      TService,
      ProviderHandlerClient<TContract, TService>
    >
  >
  & UnionToIntersection<
    ProviderSelectedEventRecord<SelectedAction<TContract>, TService>
  >;

type ProviderService = {
  readonly kv: unknown;
  readonly store: unknown;
  readonly jobs: unknown;
  readonly health: unknown;
  readonly connection: unknown;
  readonly name: unknown;
  readonly createSqlOutbox: (...args: never[]) => unknown;
  readonly createTransfer: (...args: never[]) => unknown;
  readonly handle: Record<
    string,
    Record<
      string,
      Record<
        string,
        & ((handler: (args: Record<string, unknown>) => unknown) => unknown)
        & {
          accept?: (args: unknown) => unknown;
          control?: (operationId: string) => unknown;
        }
      >
    >
  >;
  readonly event: Record<
    string,
    Record<string, {
      publish(event: Record<string, unknown>): unknown;
      prepare(
        event: Record<string, unknown>,
      ): Result<PreparedTrellisEvent, BaseError>;
      listen(handler: (event: unknown) => unknown): unknown;
    }>
  >;
  readonly [PROVIDER_CALLER]: ProviderCaller;
  publishPrepared(event: unknown): unknown;
  wait(): Promise<void>;
  stop(): Promise<void>;
};

function surfacePath(name: string): readonly [string, string] {
  const [head, ...tail] = name.split(".");
  const lowerCamel = (value: string) =>
    value
      .split(/[^A-Za-z0-9]+/)
      .filter(Boolean)
      .map((part, index) =>
        index === 0
          ? part[0]!.toLowerCase() + part.slice(1)
          : part[0]!.toUpperCase() + part.slice(1)
      )
      .join("");
  return [lowerCamel(head!), lowerCamel(tail.join("."))];
}

/** Projects a connected service into its flat provider and caller vocabulary. */
export function createProviderRuntime<
  TContract extends ContractWithRuntime,
  TService extends object,
>(
  connectedService: TService,
  contract: TContract,
): ProviderRuntime<TContract, TService> {
  const service = connectedService as TService & ProviderService;
  const provider: Record<string, unknown> = {
    kv: service.kv,
    store: service.store,
    jobs: service.jobs,
    health: service.health,
    connection: service.connection,
    name: service.name,
    createSqlOutbox: service.createSqlOutbox.bind(service),
    createTransfer: service.createTransfer.bind(service),
    publishPrepared: service.publishPrepared.bind(service),
    wait: service.wait.bind(service),
    stop: service.stop.bind(service),
  };
  const caller = service[PROVIDER_CALLER];
  for (const { action } of getContractRuntime(contract).actions) {
    const [group, leaf] = surfacePath(action.name);
    switch (action.kind) {
      case "rpc":
        provider[action.connectedName] = caller.rpc[group]![leaf]!;
        break;
      case "operation":
        {
          const operation = caller.operation[group]![leaf]!;
          const invoke = (input: unknown) => operation.input(input);
          invoke.resume = operation.resume.bind(operation);
          provider[action.connectedName] = invoke;
        }
        break;
      case "feed":
        provider[action.connectedName] = caller.feed[group]![leaf]!;
        break;
      case "event-publish":
        {
          const event = caller.event[group]![leaf]!;
          const publish = Object.assign(event.publish.bind(event), {
            prepare: event.prepare.bind(event),
          });
          provider[action.connectedName] = publish;
        }
        break;
      case "event-subscribe":
        {
          const event = service.event[group]![leaf]!;
          provider[action.connectedName] = event.listen.bind(event);
        }
        break;
    }
  }

  for (const name of Object.keys(getContractRuntime(contract).ownedApi.rpc)) {
    const [group, leaf] = surfacePath(name);
    const exportName = name.split(".").map((part) =>
      part[0]!.toUpperCase() + part.slice(1)
    ).join("");
    const register = service.handle.rpc![group]![leaf]!;
    provider[`handle${exportName}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler({ ...args, client: provider }));
  }
  for (
    const name of Object.keys(getContractRuntime(contract).ownedApi.operations)
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = name.split(".").map((part) =>
      part[0]!.toUpperCase() + part.slice(1)
    ).join("");
    const register = service.handle.operation![group]![leaf]!;
    const expose = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler({ ...args, client: provider }));
    provider[`handle${exportName}`] = Object.assign(expose, {
      ...(register.accept ? { accept: register.accept.bind(register) } : {}),
      ...(register.control ? { control: register.control.bind(register) } : {}),
    });
  }
  for (
    const name of Object.keys(getContractRuntime(contract).ownedApi.feeds ?? {})
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = name.split(".").map((part) =>
      part[0]!.toUpperCase() + part.slice(1)
    ).join("");
    const register = service.handle.feed![group]![leaf]!;
    provider[`handle${exportName}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler(args));
  }
  for (
    const name of Object.keys(getContractRuntime(contract).ownedApi.events)
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = name.split(".").map((part) =>
      part[0]!.toUpperCase() + part.slice(1)
    ).join("");
    const event = service.event[group]![leaf]!;
    provider[`on${exportName}`] = event.listen.bind(event);
    const publish = Object.assign(event.publish.bind(event), {
      prepare: event.prepare.bind(event),
    });
    provider[`publish${exportName}`] = publish;
  }

  return provider as ProviderRuntime<TContract, TService>;
}
