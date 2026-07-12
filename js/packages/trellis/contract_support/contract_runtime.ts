import type { ActionDescriptor } from "./descriptors.ts";
type RuntimeApiShape = {
  rpc: Record<string, unknown>;
  operations: Record<string, unknown>;
  events: Record<string, unknown>;
  feeds?: Record<string, unknown>;
  subjects: Record<string, unknown>;
};

export const CONTRACT_RUNTIME = Symbol("trellis.contract.runtime");

export type RuntimeSelectedAction<
  TAction extends ActionDescriptor = ActionDescriptor,
> = Readonly<{
  action: TAction;
  optional: boolean;
}>;

export type ContractRuntime<
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

export type ContractWithRuntime<
  TAction extends ActionDescriptor = ActionDescriptor,
  TOwnedApi extends RuntimeApiShape = RuntimeApiShape,
  TUsedApi extends RuntimeApiShape = RuntimeApiShape,
  TApi extends RuntimeApiShape = TOwnedApi & TUsedApi,
> = {
  readonly [CONTRACT_RUNTIME]: ContractRuntime<
    TAction,
    TOwnedApi,
    TUsedApi,
    TApi
  >;
};

export function getContractRuntime<TContract extends ContractWithRuntime>(
  contract: TContract,
): TContract[typeof CONTRACT_RUNTIME] {
  return contract[CONTRACT_RUNTIME];
}
