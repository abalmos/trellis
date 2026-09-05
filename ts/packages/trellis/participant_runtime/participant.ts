import { ACTION_METADATA, type ActionDescriptor } from "./descriptors.ts";
import type {
  EventDesc,
  FeedDesc,
  OperationDesc,
  RPCDesc,
  RuntimeApi,
} from "./api.ts";
type RuntimeApiShape = {
  rpc: Record<string, unknown>;
  operations: Record<string, unknown>;
  events: Record<string, unknown>;
  feeds?: Record<string, unknown>;
  subjects: Record<string, unknown>;
};
export const PARTICIPANT_RUNTIME: unique symbol = Symbol.for(
  "trellis.participant.runtime",
);

export type RuntimeSelectedAction<
  TAction extends ActionDescriptor = ActionDescriptor,
> = Readonly<{
  action: TAction;
  optional: boolean;
}>;

export type ParticipantRuntime<
  TAction extends ActionDescriptor = ActionDescriptor,
  TOwnedApi extends RuntimeApiShape = RuntimeApiShape,
  TUsedApi extends RuntimeApiShape = RuntimeApiShape,
  TApi extends RuntimeApiShape = TOwnedApi & TUsedApi,
> = Readonly<{
  ownedApi: TOwnedApi;
  usedApi: TUsedApi;
  api: TApi;
  actions: readonly RuntimeSelectedAction<TAction>[];
}>;

type ActionDescriptorValue<TAction> = TAction extends ActionDescriptor<
  string,
  string,
  ActionDescriptor["kind"],
  infer TDescriptor,
  string
> ? TDescriptor
  : never;

type ActionDescriptorsByKind<
  TActions extends readonly ActionDescriptor[],
  TKind extends ActionDescriptor["kind"],
> = {
  [TAction in Extract<TActions[number], { kind: TKind }> as TAction["name"]]:
    ActionDescriptorValue<TAction>;
};

/** Runtime API shape inferred from generated action descriptors. */
export type RuntimeApiForActions<
  TActions extends readonly ActionDescriptor[],
> = {
  rpc: ActionDescriptorsByKind<TActions, "rpc">;
  operations: ActionDescriptorsByKind<TActions, "operation">;
  events: ActionDescriptorsByKind<
    TActions,
    "event-publish" | "event-subscribe"
  >;
  feeds: ActionDescriptorsByKind<TActions, "feed">;
  subjects: Record<string, unknown>;
};

/** Build runtime API lookup tables from generated action descriptors. */
export function runtimeApiFromActions<
  const TActions extends readonly ActionDescriptor[],
>(
  actions: TActions,
  operationTransfers: Readonly<
    Record<string, Omit<NonNullable<OperationDesc["transfer"]>, "direction">>
  > = {},
): RuntimeApiForActions<TActions> {
  const api: RuntimeApi = { rpc: {}, operations: {}, events: {}, subjects: {} };
  for (const action of actions) {
    const descriptor = action[ACTION_METADATA].descriptor;
    if (action.kind === "rpc") api.rpc[action.name] = descriptor as RPCDesc;
    else if (action.kind === "operation") {
      const operation = descriptor as OperationDesc;
      api.operations[action.name] = {
        ...operation,
        ...(operation.transfer
          ? {
            transfer: {
              ...operation.transfer,
              ...operationTransfers[action.name],
            },
          }
          : {}),
      };
    } else if (action.kind.startsWith("event-")) {
      api.events[action.name] = descriptor as EventDesc;
    } else {
      api.feeds ??= {};
      api.feeds[action.name] = descriptor as FeedDesc;
    }
  }
  return api as RuntimeApiForActions<TActions>;
}

export type GeneratedParticipant<
  TAction extends ActionDescriptor = ActionDescriptor,
  TOwnedApi extends RuntimeApiShape = RuntimeApiShape,
  TUsedApi extends RuntimeApiShape = RuntimeApiShape,
  TApi extends RuntimeApiShape = TOwnedApi & TUsedApi,
> = {
  readonly [PARTICIPANT_RUNTIME]: ParticipantRuntime<
    TAction,
    TOwnedApi,
    TUsedApi,
    TApi
  >;
};

export function getParticipantRuntime<TContract extends GeneratedParticipant>(
  contract: TContract,
): TContract[typeof PARTICIPANT_RUNTIME] {
  return contract[PARTICIPANT_RUNTIME];
}
