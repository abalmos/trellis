import type { ContractsModule } from "../../catalog/runtime.ts";
import type { trellisControlPlaneApi } from "../../bootstrap/control_plane_api.ts";
import type { AuthRuntimeDeps } from "../runtime_deps.ts";
import type { TrellisServiceSession } from "@qlever-llc/trellis/internal/service-runtime";

type AuthOwnedApi = typeof trellisControlPlaneApi.owned;
type ControlPlaneTrellisApi = NonNullable<
  typeof trellisControlPlaneApi.trellis
>;

type AuthService = TrellisServiceSession<
  AuthOwnedApi,
  ControlPlaneTrellisApi
>;

export type AuthRpcMethod = Extract<
  keyof AuthOwnedApi["rpc"],
  `Auth.${string}`
>;

export type RpcRegistrar = { handle: AuthService["handle"] };

export type OperationRegistrar = {
  operationCompletion: Pick<
    AuthService,
    "completeOperation"
  >;
};

export type AuthRuntime =
  & RpcRegistrar
  & OperationRegistrar
  & AuthRuntimeDeps["trellis"];

export type AuthContractsRuntime = Pick<
  ContractsModule,
  | "getActiveContractsById"
  | "getActiveEntries"
  | "getBuiltinDigests"
  | "getContract"
  | "getKnownContract"
  | "getKnownEntriesByContractId"
  | "getKnownContractsById"
  | "getActiveCatalogIssues"
  | "installDeviceContract"
  | "installServiceContract"
  | "pruneInvalidCachedContracts"
  | "validateContract"
  | "refreshActiveContracts"
  | "refreshActiveContractsForRemoval"
  | "validateActiveCatalog"
  | "validateActiveCatalogForRemoval"
>;
