import { type AsyncResult, type BaseError, ok } from "@qlever-llc/result";
import type { StaticDecode, TSchema } from "typebox";
import type {
  ActionDescriptor,
  DescriptorForAction,
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
  Schema,
} from "./participant_runtime/api.ts";
import type { TrellisConnection } from "./connection.ts";
import type { GeneratedParticipantEvidence } from "./participant_runtime/artifacts.ts";
import type { OperationInvoker } from "./operations.ts";
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

type SchemaValue<TSchemaLike> = TSchemaLike extends Schema<infer TValue>
  ? TValue
  : TSchemaLike extends TSchema ? StaticDecode<TSchemaLike>
  : unknown;

type ActionMethod<TAction extends ActionDescriptor> = TAction["kind"] extends
  "rpc"
  ? DescriptorForAction<TAction> extends RPCDesc<infer TInput, infer TOutput>
    ? (
      input: SchemaValue<TInput>,
      opts?: RequestOpts,
    ) => AsyncResult<SchemaValue<TOutput>, BaseError>
  : never
  : TAction["kind"] extends "operation"
    ? DescriptorForAction<TAction> extends
      infer TOperation extends OperationDesc ?
        & ((input: SchemaValue<TOperation["input"]>) => ReturnType<
          OperationInvoker<TOperation>["input"]
        >)
        & {
          resume: OperationInvoker<TOperation>["resume"];
        }
    : never
  : TAction["kind"] extends "feed"
    ? DescriptorForAction<TAction> extends FeedDesc<infer TInput, infer TEvent>
      ? (
        input: SchemaValue<TInput>,
        opts?: FeedSubscribeOpts,
      ) => AsyncResult<AsyncIterable<SchemaValue<TEvent>>, BaseError>
    : never
  : TAction["kind"] extends "event-publish"
    ? DescriptorForAction<TAction> extends EventDesc<infer TEvent> ?
        & ((event: SchemaValue<TEvent>) => AsyncResult<void, BaseError>)
        & {
          prepare(event: SchemaValue<TEvent>): ReturnType<Trellis["prepare"]>;
        }
    : never
  : TAction["kind"] extends "event-subscribe"
    ? DescriptorForAction<TAction> extends EventDesc<infer TEvent> ? (
        handler: (
          event: SchemaValue<TEvent>,
          context: EventListenerContext,
        ) => unknown | Promise<unknown>,
        opts?: EventOpts,
      ) => AsyncResult<void, BaseError>
    : never
  : never;

type UnionToIntersection<T> =
  (T extends unknown ? (value: T) => void : never) extends
    (value: infer TIntersection) => void ? TIntersection
    : never;

type SelectedAction<TContract> = TContract extends GeneratedParticipant<
  infer TAction
> ? TAction
  : never;

type ActionRecord<TAction> = TAction extends ActionDescriptor
  ? { readonly [K in TAction["connectedName"]]: ActionMethod<TAction> }
  : never;

type CallerMethods<TContract> = UnionToIntersection<
  ActionRecord<SelectedAction<TContract>>
>;

/** Minimum participant contract accepted by the public caller connector. */
export type CallerParticipant = GeneratedParticipantEvidence;

/** Flat caller surface inferred from the participant contract's selected actions. */
export type CallerRuntime<TContract> = CallerMethods<TContract> & {
  readonly connection: TrellisConnection;
  readonly state: StateFacade<RuntimeStateStoresForContract<TContract>>;
  publishPrepared(event: PreparedTrellisEvent): AsyncResult<void, BaseError>;
  transfer: Trellis["transfer"];
  wait(): AsyncResult<void, BaseError>;
};

/** Projects a private Trellis session into the selected flat caller vocabulary. */
export function createCallerRuntime<TContract extends GeneratedParticipant>(
  session: object,
  contract: TContract,
): CallerRuntime<TContract> {
  const runtime = session as Trellis;
  const caller: Record<string, unknown> = {
    connection: runtime.connection,
    state: runtime.state,
    publishPrepared: runtime.publishPrepared.bind(runtime),
    transfer: runtime.transfer.bind(runtime),
    wait: runtime.wait.bind(runtime),
  };

  for (const { action } of getParticipantRuntime(contract).actions) {
    switch (action.kind) {
      case "rpc":
        caller[action.connectedName] = (input: unknown, opts?: RequestOpts) =>
          runtime.request(action.name, input, opts);
        break;
      case "operation":
        {
          const operation = runtime.operationHandle(action.name);
          const invoke = (input: unknown) => operation.input(input);
          invoke.resume = operation.resume.bind(operation);
          caller[action.connectedName] = invoke;
        }
        break;
      case "feed":
        caller[action.connectedName] = (
          input: unknown,
          opts?: FeedSubscribeOpts,
        ) => runtime.feedHandle(action.name).input(input).subscribe(opts);
        break;
      case "event-publish":
        {
          const publish = (event: Record<string, unknown>) =>
            runtime.publish(action.name, event);
          publish.prepare = (event: Record<string, unknown>) =>
            runtime.prepare(action.name, event);
          caller[action.connectedName] = publish;
        }
        break;
      case "event-subscribe":
        caller[action.connectedName] = (
          handler: (
            event: unknown,
            context: EventListenerContext,
          ) => unknown | Promise<unknown>,
          opts?: EventOpts,
        ) =>
          runtime.listenEvent(action.name, {}, async (event, context) => {
            await handler(event, context);
            return ok(undefined);
          }, opts);
        break;
    }
  }

  return caller as CallerRuntime<TContract>;
}
