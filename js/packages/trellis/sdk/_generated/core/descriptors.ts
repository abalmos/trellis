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
  TrellisBindingsGetRequestSchema,
  TrellisBindingsGetResponseSchema,
  TrellisCatalogRequestSchema,
  TrellisCatalogResponseSchema,
  TrellisContractGetRequestSchema,
  TrellisContractGetResponseSchema,
  TrellisSurfaceStatusRequestSchema,
  TrellisSurfaceStatusResponseSchema,
} from "./schemas.ts";

const CONTRACT_ID = "trellis.core@v1" as const;

export const TrellisCatalog = rpcAction(CONTRACT_ID, "Trellis.Catalog", {
  subject: "rpc.v1.Trellis.Catalog",
  input: schema<Types.TrellisCatalogInput>(TrellisCatalogRequestSchema),
  output: schema<Types.TrellisCatalogOutput>(TrellisCatalogResponseSchema),
  callerCapabilities: ["trellis.core::catalog.read"] as const,
  errors: ["UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: ["UnexpectedError", "ValidationError"] as const,
}, "TrellisCatalog");

export const TrellisContractGet = rpcAction(
  CONTRACT_ID,
  "Trellis.Contract.Get",
  {
    subject: "rpc.v1.Trellis.Contract.Get",
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
);

export const TrellisSurfaceStatus = rpcAction(
  CONTRACT_ID,
  "Trellis.Surface.Status",
  {
    subject: "rpc.v1.Trellis.Surface.Status",
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
);
