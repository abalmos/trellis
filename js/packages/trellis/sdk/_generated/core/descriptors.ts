// Generated from ./generated/contracts/manifests/trellis.core@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import type * as Types from "./types.ts";
import {
  TrellisCatalogRequestSchema,
  TrellisCatalogResponseSchema,
  TrellisContractGetRequestSchema,
  TrellisContractGetResponseSchema,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "./schemas.ts";
import {
  CONTRACT as ACTION_ARTIFACT,
  CONTRACT_DIGEST as ACTION_DIGEST,
} from "./manifest.ts";

const ACTION_SOURCE = {
  artifact: ACTION_ARTIFACT,
  digest: ACTION_DIGEST,
} as const;

const CONTRACT_ID = "trellis.core@v1" as const;

export const TrellisCatalog = rpcAction(
  CONTRACT_ID,
  "Trellis.Catalog",
  {
    subject: "rpc.v1.Trellis.Catalog",
    permission: Object.freeze({
      apiId: "trellis.core@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Trellis.Catalog",
      action: "call",
    }),
    input: schema<Types.TrellisCatalogInput>(TrellisCatalogRequestSchema),
    output: schema<Types.TrellisCatalogOutput>(TrellisCatalogResponseSchema),
    callerCapabilities: ["trellis.core::catalog.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "TrellisCatalog",
  ACTION_SOURCE,
);

export const TrellisContractGet = rpcAction(
  CONTRACT_ID,
  "Trellis.Contract.Get",
  {
    subject: "rpc.v1.Trellis.Contract.Get",
    permission: Object.freeze({
      apiId: "trellis.core@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Trellis.Contract.Get",
      action: "call",
    }),
    input: schema<Types.TrellisContractGetInput>(
      TrellisContractGetRequestSchema,
    ),
    output: schema<Types.TrellisContractGetOutput>(
      TrellisContractGetResponseSchema,
    ),
    callerCapabilities: ["trellis.core::contract.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "TrellisContractGet",
  ACTION_SOURCE,
);

export const TrellisSurfaceStatus = rpcAction(
  CONTRACT_ID,
  "Trellis.Surface.Status",
  {
    subject: "rpc.v1.Trellis.Surface.Status",
    permission: Object.freeze({
      apiId: "trellis.core@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "Trellis.Surface.Status",
      action: "call",
    }),
    input: schema<Types.TrellisSurfaceStatusInput>(
      TrellisSurfaceStatusRequestSchema,
    ),
    output: schema<Types.TrellisSurfaceStatusOutput>(
      TrellisSurfaceStatusResponseSchema,
    ),
    callerCapabilities: ["trellis.core::catalog.read"] as const,
    errors: ["UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
  },
  "TrellisSurfaceStatus",
  ACTION_SOURCE,
);
