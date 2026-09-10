import { AsyncResult, type BaseError, err, ok } from "@qlever-llc/result";
import type { Codec } from "./generated.ts";
import type { TrellisConnection } from "./connection.ts";
import type { OperationInvoker } from "./operations.ts";
import {
  type GeneratedActionDescriptor,
  type GeneratedActionSelection,
  type GeneratedParticipant,
  getParticipantRuntime,
} from "./participant_runtime/participant.ts";
import { createActionUnavailableError } from "./session.ts";
import type {
  EventListenerContext,
  EventOpts,
  FeedSubscribeOpts,
  PreparedTrellisEvent,
  RequestOpts,
  RuntimeStateStoresForContract,
  StateFacade,
  Trellis,
} from "./session.ts";
import type { ConnectedActionName } from "./participant_runtime/surface_names.ts";

type CodecValue<T> = T extends Codec<infer TValue> ? TValue : never;
type DescriptorName<T> = T extends `${string}:${infer TName}` ? TName : never;
type SelectedDescriptor<TSelection> = TSelection extends
  GeneratedActionSelection
  ? TSelection["actions"][number] extends infer TSelected
    ? TSelected extends { descriptorName: infer TName extends string }
      ? TName extends keyof TSelection["api"]["actions"]
        ? TSelection["api"]["actions"][TName] & {
          direction: TSelected extends { direction: infer TDirection }
            ? TDirection
            : never;
        }
      : never
    : never
  : never
  : never;
type SelectedAction<TContract extends GeneratedParticipant> =
  SelectedDescriptor<TContract["uses"][number]>;

type ActionMethod<
  TAction extends GeneratedActionDescriptor & {
    direction: unknown;
  },
> = TAction["kind"] extends "rpc"
  ? TAction["input"] extends Codec<unknown>
    ? TAction["output"] extends Codec<unknown> ? (
        input: CodecValue<TAction["input"]>,
        opts?: RequestOpts,
      ) => AsyncResult<CodecValue<TAction["output"]>, BaseError>
    : never
  : never
  : TAction["kind"] extends "operation"
    ? TAction["input"] extends Codec<unknown> ?
        & ((input: CodecValue<TAction["input"]>) => ReturnType<
          OperationInvoker<never>["input"]
        >)
        & { resume: OperationInvoker<never>["resume"] }
    : never
  : TAction["kind"] extends "feed"
    ? TAction["input"] extends Codec<unknown>
      ? TAction["event"] extends Codec<unknown> ? (
          input: CodecValue<TAction["input"]>,
          opts?: FeedSubscribeOpts,
        ) => AsyncResult<AsyncIterable<CodecValue<TAction["event"]>>, BaseError>
      : never
    : never
  : TAction["kind"] extends "event"
    ? TAction["payload"] extends Codec<unknown>
      ? TAction["direction"] extends "publish" ?
          & ((
            event: CodecValue<TAction["payload"]>,
          ) => AsyncResult<void, BaseError>)
          & {
            prepare(
              event: CodecValue<TAction["payload"]>,
            ): ReturnType<Trellis["prepare"]>;
          }
      : (
        handler: (
          event: CodecValue<TAction["payload"]>,
          context: EventListenerContext,
        ) => unknown | Promise<unknown>,
        opts?: EventOpts,
      ) => AsyncResult<void, BaseError>
    : never
  : never;

type ActionRecord<TAction> = TAction extends
  GeneratedActionDescriptor & { direction: unknown } ? {
    readonly [
      K in ConnectedActionName<DescriptorName<TAction["descriptorName"]>>
    ]: ActionMethod<TAction>;
  }
  : never;
type UnionToIntersection<T> =
  (T extends unknown ? (value: T) => void : never) extends
    (value: infer TIntersection) => void ? TIntersection
    : never;

/** Minimum participant contract accepted by the public caller connector. */
export type CallerParticipant = GeneratedParticipant;

/** Flat caller surface inferred from a generated participant's selected actions. */
export type CallerRuntime<TContract extends GeneratedParticipant> =
  & UnionToIntersection<ActionRecord<SelectedAction<TContract>>>
  & {
    readonly connection: TrellisConnection;
    availability(): ParticipantAvailability<TContract>;
    watchAvailability(): AsyncIterable<ParticipantAvailability<TContract>>;
    readonly state: StateFacade<RuntimeStateStoresForContract<TContract>>;
    publishPrepared(event: PreparedTrellisEvent): AsyncResult<void, BaseError>;
    transfer: Trellis["transfer"];
    wait(): AsyncResult<void, BaseError>;
  };

type OptionalCapability<TContract extends GeneratedParticipant> =
  TContract["uses"][number]["optionalCapabilities"][number];
type OptionalResource<TContract extends GeneratedParticipant> = {
  [Name in keyof TContract["resources"]]:
    TContract["resources"][Name]["availability"] extends "optional" ? Name
      : never;
}[keyof TContract["resources"]];
type ParticipantAvailability<TContract extends GeneratedParticipant> = Readonly<
  {
    capabilities: Readonly<Record<OptionalCapability<TContract>, boolean>>;
    resources: Readonly<Record<OptionalResource<TContract>, boolean>>;
  }
>;

/** Projects a private Trellis session into the selected flat caller vocabulary. */
export function createCallerRuntime<TContract extends GeneratedParticipant>(
  session: object,
  contract: TContract,
): CallerRuntime<TContract> {
  const runtime = session as Trellis;
  const caller: Record<string, unknown> = {
    connection: runtime.connection,
    availability: () => runtime.connection.availability(),
    watchAvailability: () => runtime.connection.watchAvailability(),
    state: runtime.state,
    publishPrepared: runtime.publishPrepared.bind(runtime),
    transfer: runtime.transfer.bind(runtime),
    wait: runtime.wait.bind(runtime),
  };

  for (const action of getParticipantRuntime(contract).actions) {
    const unavailable = () => {
      const current = runtime.connection.availability().capabilities;
      return action.optionalCapabilities.length > 0 &&
          !action.optionalCapabilities.every((capability) =>
            current[capability]
          )
        ? createActionUnavailableError(action.name, action.optionalCapabilities)
        : undefined;
    };
    switch (action.descriptor.kind) {
      case "rpc":
        caller[action.connectedName] = (input: unknown, opts?: RequestOpts) => {
          const error = unavailable();
          return error
            ? AsyncResult.from(Promise.resolve(err(error)))
            : runtime.request(action.name, input, opts);
        };
        break;
      case "operation":
        {
          const operation = runtime.operationHandle(action.name, unavailable);
          const invoke = (input: unknown) => operation.input(input);
          invoke.resume = operation.resume.bind(operation);
          caller[action.connectedName] = invoke;
        }
        break;
      case "feed":
        caller[action.connectedName] = (
          input: unknown,
          opts?: FeedSubscribeOpts,
        ) => {
          const error = unavailable();
          return error
            ? AsyncResult.from(Promise.resolve(err(error)))
            : runtime.feedHandle(action.name).input(input).subscribe(opts);
        };
        break;
      case "event":
        if (action.direction === "publish") {
          const publish = (event: Record<string, unknown>) => {
            const error = unavailable();
            return error
              ? AsyncResult.from(Promise.resolve(err(error)))
              : runtime.publish(action.name, event);
          };
          publish.prepare = (event: Record<string, unknown>) =>
            runtime.prepare(action.name, event);
          caller[action.connectedName] = publish;
        } else {
          caller[action.connectedName] = (
            handler: (
              event: unknown,
              context: EventListenerContext,
            ) => unknown | Promise<unknown>,
            opts?: EventOpts,
          ) => {
            const error = unavailable();
            return error
              ? AsyncResult.from(Promise.resolve(err(error)))
              : runtime.listenEvent(action.name, {}, async (event, context) => {
                await handler(event, context);
                return ok(undefined);
              }, opts);
          };
        }
        break;
    }
  }

  return caller as CallerRuntime<TContract>;
}
