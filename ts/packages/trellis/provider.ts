import type { BaseError, Result } from "@qlever-llc/result";
import { type CallerRuntime, createCallerRuntime } from "./caller.ts";
import {
  type GeneratedParticipant,
  getParticipantRuntime,
} from "./participant_runtime/participant.ts";
import {
  lowerCamelSurfaceName,
  pascalSurfaceName,
} from "./participant_runtime/surface_names.ts";
import type { PreparedTrellisEvent } from "./session.ts";

export const PROVIDER_CALLER = Symbol("trellis.provider.caller");

export type ProviderCaller = object;

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

type ProviderResources<TService> = TService extends {
  readonly kv: infer TKv;
  readonly store: infer TStore;
  readonly jobs: infer TJobs;
} ? { readonly kv: TKv; readonly store: TStore; readonly jobs: TJobs }
  : {};

type ProviderCallerSurface<TContract extends GeneratedParticipant> = Omit<
  CallerRuntime<TContract>,
  | "connection"
  | "state"
  | "wait"
  | Extract<
    keyof CallerRuntime<TContract>,
    `on${string}`
  >
>;

/** Caller and bound-resource surface available inside provider handlers. */
export type ProviderHandlerClient<
  TContract extends GeneratedParticipant,
  TService,
> =
  & ProviderCallerSurface<TContract>
  & ProviderResources<TService>
  & (ProviderBase<TService> extends infer TBase
    ? TBase extends { connection: unknown; name: unknown }
      ? Pick<TBase, "connection" | "name">
    : {}
    : {});

/** Connected provider facade for a generated service participant. */
export type ProviderRuntime<
  TContract extends GeneratedParticipant,
  TService,
> =
  & ProviderBase<TService>
  & ProviderResources<TService>
  & ProviderCallerSurface<TContract>
  & Readonly<
    Record<
      `handle${string}` | `on${string}` | `publish${string}`,
      (...args: unknown[]) => unknown
    >
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
  readonly [PROVIDER_CALLER]: ProviderCaller;
  publishPrepared(event: unknown): unknown;
  wait(): Promise<void>;
  stop(): Promise<void>;
};

function surfacePath(name: string): readonly [string, string] {
  const [head, ...tail] = name.split(".");
  return [
    lowerCamelSurfaceName(head!),
    lowerCamelSurfaceName(tail.length === 0 ? name : tail.join(".")),
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
  for (const action of getParticipantRuntime(contract).actions) {
    const connected = caller[action.connectedName];
    if (
      action.descriptor.kind === "event" && action.direction === "subscribe"
    ) {
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
    const register = service.handle.rpc![group]![leaf]!;
    provider[`handle${pascalSurfaceName(name)}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler({ ...args, client: provider }));
  }
  for (
    const name of Object.keys(
      getParticipantRuntime(contract).ownedApi.operations,
    )
  ) {
    const [group, leaf] = surfacePath(name);
    const register = service.handle.operation![group]![leaf]!;
    const expose = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler({ ...args, client: provider }));
    provider[`handle${pascalSurfaceName(name)}`] = Object.assign(expose, {
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
    const register = service.handle.feed![group]![leaf]!;
    provider[`handle${pascalSurfaceName(name)}`] = (
      handler: (args: Record<string, unknown>) => unknown,
    ) => register((args) => handler(args));
  }
  for (
    const name of Object.keys(getParticipantRuntime(contract).ownedApi.events)
  ) {
    provider[`on${pascalSurfaceName(name)}`] = (
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
    provider[`publish${pascalSurfaceName(name)}`] = publish;
  }

  return provider as ProviderRuntime<TContract, TService>;
}
