import type { AsyncResult, BaseError, Result } from "@qlever-llc/result";
import type {
  ActionDescriptor,
  DescriptorForAction,
  EventActions,
} from "./participant_runtime/descriptors.ts";
import {
  type GeneratedParticipant,
  getParticipantRuntime,
} from "./participant_runtime/participant.ts";
import type {
  EventDesc,
  FeedDesc,
  OperationDesc,
  RPCDesc,
} from "./participant_runtime/api.ts";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import type {
  PARTICIPANT_JOBS_METADATA,
  PARTICIPANT_KV_METADATA,
  PARTICIPANT_STORE_METADATA,
} from "./participant_runtime/metadata.ts";
import type { PreparedTrellisEvent } from "./session.ts";
import {
  type ConnectedActionName,
  lowerCamelSurfaceName,
  type PascalActionName,
  pascalSurfaceName,
} from "./participant_runtime/surface_names.ts";

export const PROVIDER_CALLER = Symbol("trellis.provider.caller");

export type ProviderCaller = object;

type EventBody<TDescriptor> = TDescriptor extends EventDesc<infer TEvent>
  ? TEvent extends { readonly __trellisType?: infer TValue } ? TValue : unknown
  : unknown;

type DirectAction<TContract> = TContract[keyof TContract] extends infer TValue
  ? TValue extends ActionDescriptor ? TValue
  : TValue extends EventActions<infer TSubscribe, infer TPublish>
    ? TSubscribe | Exclude<TPublish, undefined>
  : never
  : never;
type SelectedAction<TContract> = TContract extends GeneratedParticipant<
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
  `${infer THead}.${string}` ? ConnectedActionName<THead>
  : never;
type SurfaceLeaf<TName extends string> = TName extends
  `${string}.${infer TTail}` ? ConnectedActionName<TTail>
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
type ServiceEventListener<
  TContract,
  TService,
  TAction extends ActionDescriptor,
> = TService extends { event: infer TEvent }
  ? SurfaceGroup<TAction["name"]> extends keyof TEvent
    ? SurfaceLeaf<TAction["name"]> extends
      keyof TEvent[SurfaceGroup<TAction["name"]>]
      ? TEvent[SurfaceGroup<TAction["name"]>][
        SurfaceLeaf<TAction["name"]>
      ] extends {
        listen: infer TListen;
      } ? TListen extends (
          handler: (
            event: infer TEvent,
            context: infer TContext,
          ) => infer THandlerResult,
          ...args: infer TRest
        ) => infer TResult ? (
            handler: (args: {
              event: TEvent;
              context: TContext;
              client: ProviderHandlerClient<TContract, TService>;
            }) => THandlerResult,
            ...args: TRest
          ) => TResult
        : never
      : never
    : never
  : never
  : never;

type ProviderActionRecord<TContract, TAction, TService, TClient> =
  TAction extends ActionDescriptor
    ? TAction["kind"] extends "event-subscribe" ? {
        readonly [K in TAction["connectedName"]]: ServiceEventListener<
          TContract,
          TService,
          TAction
        >;
      }
    : TAction["kind"] extends "event-publish" ? {
        readonly [K in `publish${PascalActionName<TAction["name"]>}`]:
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
      readonly [K in `handle${PascalActionName<TAction["name"]>}`]:
        ServiceRegistration<TService, TAction, TClient>;
    }
    : {};

type ProviderSelectedEventRecord<TContract, TAction, TService> = TAction extends
  ActionDescriptor<string, string, "event-subscribe"> ? {
    readonly [K in TAction["connectedName"]]: ServiceEventListener<
      TContract,
      TService,
      TAction
    >;
  }
  : {};

type ProviderOwnedEventPublisherRecord<TAction> = TAction extends
  ActionDescriptor<string, string, "event-publish"> ? {
    readonly [K in `publish${PascalActionName<TAction["name"]>}`]:
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
  readonly [PARTICIPANT_KV_METADATA]?: infer TMetadata;
} ? NonNullable<TMetadata>
  : {};
type ContractStore<TContract> = TContract extends {
  readonly [PARTICIPANT_STORE_METADATA]?: infer TMetadata;
} ? NonNullable<TMetadata>
  : {};
type ContractJobs<TContract> = TContract extends {
  readonly [PARTICIPANT_JOBS_METADATA]?: infer TMetadata;
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
type ProviderJobs<TContract, TService> = TService extends {
  readonly jobs: infer TJobs;
} ? {
    readonly jobs: {
      readonly [K in keyof ContractJobs<TContract> & keyof TJobs]:
        TJobs[K] extends { handle: infer THandle }
          ? Omit<TJobs[K], "handle"> & {
            handle: THandle extends (
              handler: (args: infer TArgs) => infer THandlerResult,
              ...args: infer TRest
            ) => infer TResult ? (
                handler: (
                  args: TArgs extends { client: unknown }
                    ? Omit<TArgs, "client"> & {
                      client: ProviderHandlerClient<TContract, TService>;
                    }
                    : TArgs,
                ) => THandlerResult,
                ...args: TRest
              ) => TResult
              : never;
          }
          : TJobs[K];
    };
  }
  : {};
type ProviderFeatures<TContract, TService> =
  & SelectedServiceProperty<TService, "kv", ContractKv<TContract>>
  & SelectedServiceProperty<TService, "store", ContractStore<TContract>>
  & ProviderJobs<TContract, TService>;
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
      TContract,
      DirectAction<TContract>,
      TService,
      ProviderHandlerClient<TContract, TService>
    >
  >
  & UnionToIntersection<
    ProviderSelectedEventRecord<TContract, SelectedAction<TContract>, TService>
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
      listen(
        handler: (event: unknown, context: unknown) => unknown,
        subjectData?: Record<string, unknown>,
        options?: unknown,
      ): unknown;
    }>
  >;
  readonly [PROVIDER_CALLER]: ProviderCaller;
  publishPrepared(event: unknown): unknown;
  wait(): Promise<void>;
  stop(): Promise<void>;
};

function surfacePath(name: string): readonly [string, string] {
  const [head, ...tail] = name.split(".");
  return [
    lowerCamelSurfaceName(head!),
    lowerCamelSurfaceName(tail.join(".")),
  ];
}

/** Projects a connected service into its flat provider and caller vocabulary. */
export function createProviderRuntime<
  TContract extends GeneratedParticipant,
  TService extends object,
>(
  connectedService: TService,
  contract: TContract,
): ProviderRuntime<TContract, TService> {
  const service = connectedService as TService & ProviderService;
  const provider: Record<string, unknown> = {
    kv: service.kv,
    store: service.store,
    health: service.health,
    connection: service.connection,
    name: service.name,
    createSqlOutbox: service.createSqlOutbox.bind(service),
    createTransfer: service.createTransfer.bind(service),
    publishPrepared: service.publishPrepared.bind(service),
    wait: service.wait.bind(service),
    stop: service.stop.bind(service),
  };
  provider.jobs = Object.fromEntries(
    Object.entries(service.jobs as Record<string, Record<string, unknown>>).map(
      ([name, queue]) => [name, {
        ...queue,
        handle: (
          handler: (args: Record<string, unknown>) => unknown,
          options?: unknown,
        ) =>
          (queue.handle as (
            handler: (args: Record<string, unknown>) => unknown,
            options?: unknown,
          ) => unknown)(
            (args) => handler({ ...args, client: provider }),
            options,
          ),
      }],
    ),
  );
  const caller = createCallerRuntime(service[PROVIDER_CALLER], contract) as
    & Record<string, unknown>
    & CallerRuntime<TContract>;
  for (const { action } of getParticipantRuntime(contract).actions) {
    const connected = caller[action.connectedName];
    if (action.kind === "event-subscribe") {
      provider[action.connectedName] = (
        handler: (args: Record<string, unknown>) => unknown,
        subjectData?: Record<string, unknown>,
        options?: unknown,
      ) =>
        (service[PROVIDER_CALLER] as {
          listenEvent(
            event: string,
            subjectData: Record<string, unknown>,
            handler: (message: unknown, context: unknown) => unknown,
            options?: unknown,
          ): unknown;
        }).listenEvent(
          action.name,
          subjectData ?? {},
          (message, context) =>
            handler({ event: message, context, client: provider }),
          options,
        );
    } else {
      provider[action.connectedName] = connected;
    }
  }

  for (
    const name of Object.keys(getParticipantRuntime(contract).ownedApi.rpc)
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = pascalSurfaceName(name);
    const register = service.handle.rpc![group]![leaf]!;
    provider[`handle${exportName}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler({ ...args, client: provider }));
  }
  for (
    const name of Object.keys(
      getParticipantRuntime(contract).ownedApi.operations,
    )
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = pascalSurfaceName(name);
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
    const name of Object.keys(
      getParticipantRuntime(contract).ownedApi.feeds ?? {},
    )
  ) {
    const [group, leaf] = surfacePath(name);
    const exportName = pascalSurfaceName(name);
    const register = service.handle.feed![group]![leaf]!;
    provider[`handle${exportName}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler(args));
  }
  for (
    const name of Object.keys(getParticipantRuntime(contract).ownedApi.events)
  ) {
    const exportName = pascalSurfaceName(name);
    provider[`on${exportName}`] = (
      handler: (args: Record<string, unknown>) => unknown,
      subjectData?: Record<string, unknown>,
      options?: unknown,
    ) =>
      (service[PROVIDER_CALLER] as {
        listenEvent(
          event: string,
          subjectData: Record<string, unknown>,
          handler: (message: unknown, context: unknown) => unknown,
          options?: unknown,
        ): unknown;
      }).listenEvent(
        name,
        subjectData ?? {},
        (message, context) =>
          handler({ event: message, context, client: provider }),
        options,
      );
    const publish = Object.assign(
      (event: Record<string, unknown>) =>
        (service[PROVIDER_CALLER] as {
          publish(event: string, data: Record<string, unknown>): unknown;
        }).publish(name, event),
      {
        prepare: (event: Record<string, unknown>) =>
          (service[PROVIDER_CALLER] as {
            prepare(
              event: string,
              data: Record<string, unknown>,
            ): Result<PreparedTrellisEvent, BaseError>;
          }).prepare(name, event),
      },
    );
    provider[`publish${exportName}`] = publish;
  }

  return provider as ProviderRuntime<TContract, TService>;
}
