import type { EventDesc, FeedDesc, OperationDesc, RPCDesc } from "./api.ts";
import {
  type ConnectedActionName,
  lowerCamelSurfaceName,
  type PascalActionName,
  pascalSurfaceName,
} from "./surface_names.ts";

export type { ConnectedActionName } from "./surface_names.ts";

export const ACTION_METADATA: unique symbol = Symbol.for(
  "trellis.action.metadata",
);

/** The callable direction represented by a contract action descriptor. */
export type ActionKind =
  | "rpc"
  | "operation"
  | "feed"
  | "event-publish"
  | "event-subscribe";

type RuntimeDescriptor = RPCDesc | OperationDesc | FeedDesc | EventDesc;

/** Exact native API evidence carried by portable generated actions. */
export type ActionSource = {
  readonly api: Readonly<Record<string, unknown>>;
  readonly apiDigest: string;
};

type ActionMetadata<TDescriptor extends RuntimeDescriptor> = {
  descriptor: TDescriptor;
  source?: ActionSource;
};

/** A frozen, portable selection of one callable contract action. */
export interface ActionDescriptor<
  TContractId extends string = string,
  TName extends string = string,
  TKind extends ActionKind = ActionKind,
  TDescriptor extends RuntimeDescriptor = RuntimeDescriptor,
  TConnectedName extends string = string,
> {
  readonly contractId: TContractId;
  readonly name: TName;
  readonly kind: TKind;
  readonly subject: string;
  readonly exportName: string;
  readonly connectedName: TConnectedName;
  readonly [ACTION_METADATA]: ActionMetadata<TDescriptor>;
}

/** Extracts the typed transport descriptor carried by an action. */
export type DescriptorForAction<TAction extends ActionDescriptor> =
  TAction[typeof ACTION_METADATA]["descriptor"];

/** The subscribe and optional delegated-publish actions for one event. */
export type EventActions<
  TSubscribe extends ActionDescriptor<
    string,
    string,
    "event-subscribe",
    EventDesc,
    string
  >,
  TPublish extends
    | ActionDescriptor<string, string, "event-publish", EventDesc, string>
    | undefined,
> = Readonly<{
  subscribe: TSubscribe;
  publish: TPublish;
}>;

/** A required or optional group of actions from one owner contract. */
export type OptionalActionGroup<
  TActions extends readonly ActionDescriptor[],
> = Readonly<{
  optional: true;
  actions: TActions;
}>;

/** A local connected-name override that does not affect canonical identity. */
export type AliasedAction<
  TAction extends ActionDescriptor,
  TConnectedName extends string,
> = ActionDescriptor<
  TAction["contractId"],
  TAction["name"],
  TAction["kind"],
  TAction[typeof ACTION_METADATA]["descriptor"],
  TConnectedName
>;

function createAction<
  const TContractId extends string,
  const TName extends string,
  const TKind extends ActionKind,
  const TDescriptor extends RuntimeDescriptor,
  const TConnectedName extends string,
>(options: {
  contractId: TContractId;
  name: TName;
  kind: TKind;
  descriptor: TDescriptor;
  exportName: string;
  connectedName: TConnectedName;
  source?: ActionSource;
}): ActionDescriptor<
  TContractId,
  TName,
  TKind,
  TDescriptor,
  TConnectedName
> {
  const action = {
    contractId: options.contractId,
    name: options.name,
    kind: options.kind,
    subject: options.descriptor.subject,
    exportName: options.exportName,
    connectedName: options.connectedName,
  } as ActionDescriptor<
    TContractId,
    TName,
    TKind,
    TDescriptor,
    TConnectedName
  >;
  Object.defineProperty(action, ACTION_METADATA, {
    value: Object.freeze({
      descriptor: options.descriptor,
      ...(options.source ? { source: options.source } : {}),
    }),
  });
  return Object.freeze(action);
}

/** Creates an RPC action descriptor for generated or locally owned vocabulary. */
export function rpcAction<
  const TContractId extends string,
  const TName extends string,
  const TDescriptor extends RPCDesc,
>(
  contractId: TContractId,
  name: TName,
  descriptor: TDescriptor,
  exportName: string,
  source?: ActionSource,
): ActionDescriptor<
  TContractId,
  TName,
  "rpc",
  TDescriptor,
  ConnectedActionName<TName>
> {
  return createAction({
    contractId,
    name,
    kind: "rpc",
    descriptor,
    exportName,
    source,
    connectedName: lowerCamelSurfaceName(name) as ConnectedActionName<TName>,
  });
}

/** Creates an operation action descriptor for generated or locally owned vocabulary. */
export function operationAction<
  const TContractId extends string,
  const TName extends string,
  const TDescriptor extends OperationDesc,
>(
  contractId: TContractId,
  name: TName,
  descriptor: TDescriptor,
  exportName: string,
  source?: ActionSource,
): ActionDescriptor<
  TContractId,
  TName,
  "operation",
  TDescriptor,
  ConnectedActionName<TName>
> {
  return createAction({
    contractId,
    name,
    kind: "operation",
    descriptor,
    exportName,
    source,
    connectedName: lowerCamelSurfaceName(name) as ConnectedActionName<TName>,
  });
}

/** Creates a feed-subscribe action descriptor for generated or locally owned vocabulary. */
export function feedAction<
  const TContractId extends string,
  const TName extends string,
  const TDescriptor extends FeedDesc,
>(
  contractId: TContractId,
  name: TName,
  descriptor: TDescriptor,
  exportName: string,
  source?: ActionSource,
): ActionDescriptor<
  TContractId,
  TName,
  "feed",
  TDescriptor,
  ConnectedActionName<TName>
> {
  return createAction({
    contractId,
    name,
    kind: "feed",
    descriptor,
    exportName,
    source,
    connectedName: lowerCamelSurfaceName(name) as ConnectedActionName<TName>,
  });
}

/** Creates event actions, omitting delegated publication for owner-only events. */
export function eventActions<
  const TContractId extends string,
  const TName extends string,
  const TDescriptor extends EventDesc,
  const TDelegatedPublish extends boolean,
>(
  contractId: TContractId,
  name: TName,
  descriptor: TDescriptor,
  exportName: string,
  delegatedPublish: TDelegatedPublish,
  source?: ActionSource,
): EventActions<
  ActionDescriptor<
    TContractId,
    TName,
    "event-subscribe",
    TDescriptor,
    `on${PascalActionName<TName>}`
  >,
  TDelegatedPublish extends true ? ActionDescriptor<
      TContractId,
      TName,
      "event-publish",
      TDescriptor,
      `publish${PascalActionName<TName>}`
    >
    : undefined
> {
  const baseName = pascalSurfaceName(name);
  const subscribe = createAction({
    contractId,
    name,
    kind: "event-subscribe",
    descriptor,
    source,
    exportName: `${exportName}Subscribe`,
    connectedName: `on${baseName}` as `on${PascalActionName<TName>}`,
  });
  const publish = delegatedPublish
    ? createAction({
      contractId,
      name,
      kind: "event-publish",
      descriptor,
      source,
      exportName: `${exportName}Publish`,
      connectedName: `publish${baseName}` as `publish${PascalActionName<
        TName
      >}`,
    })
    : undefined;
  return Object.freeze({ subscribe, publish }) as EventActions<
    ActionDescriptor<
      TContractId,
      TName,
      "event-subscribe",
      TDescriptor,
      `on${PascalActionName<TName>}`
    >,
    TDelegatedPublish extends true ? ActionDescriptor<
        TContractId,
        TName,
        "event-publish",
        TDescriptor,
        `publish${PascalActionName<TName>}`
      >
      : undefined
  >;
}

/** Marks actions from one owner contract as an optional dependency. */
export function optional<const TActions extends readonly ActionDescriptor[]>(
  ...actions: TActions
): OptionalActionGroup<TActions> {
  if (actions.length === 0) {
    throw new Error("optional(...) requires at least one action");
  }
  const owner = actions[0]!.contractId;
  if (actions.some((action) => action.contractId !== owner)) {
    throw new Error("optional(...) actions must have the same owner contract");
  }
  return Object.freeze({ optional: true, actions: Object.freeze(actions) });
}

/** Selects the control surfaces requested for one operation dependency. */
export function operationAccess<
  const TAction extends ActionDescriptor<
    string,
    string,
    "operation",
    OperationDesc,
    string
  >,
>(
  action: TAction,
  access: { readonly cancel?: boolean; readonly control?: boolean },
): TAction {
  const descriptor = action[ACTION_METADATA].descriptor;
  return createAction({
    contractId: action.contractId,
    name: action.name,
    kind: action.kind,
    descriptor: {
      ...descriptor,
      cancel: access.cancel === true && descriptor.cancel === true,
      controlCapabilities: access.control === true
        ? descriptor.controlCapabilities
        : [],
      signals: access.control === true ? descriptor.signals : undefined,
    },
    source: action[ACTION_METADATA].source,
    exportName: action.exportName,
    connectedName: action.connectedName,
  }) as TAction;
}

/** Overrides only the local connected method name for an action. */
export function as<
  const TConnectedName extends string,
  const TAction extends ActionDescriptor,
>(
  connectedName: TConnectedName,
  action: TAction,
): AliasedAction<TAction, TConnectedName> {
  if (!connectedName) {
    throw new Error("Action alias must not be empty");
  }
  return createAction({
    contractId: action.contractId,
    name: action.name,
    kind: action.kind,
    descriptor: action[ACTION_METADATA].descriptor,
    source: action[ACTION_METADATA].source,
    exportName: action.exportName,
    connectedName,
  });
}

/** Returns the private runtime descriptor associated with an action. */
export function actionRuntimeDescriptor<TDescriptor extends RuntimeDescriptor>(
  action: ActionDescriptor<string, string, ActionKind, TDescriptor>,
): TDescriptor {
  return action[ACTION_METADATA].descriptor;
}

/** Returns the private source artifact associated with a generated action. */
export function actionSource(
  action: ActionDescriptor,
): ActionSource | undefined {
  return action[ACTION_METADATA].source;
}
