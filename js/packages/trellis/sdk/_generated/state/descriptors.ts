// Generated from ./generated/contracts/manifests/trellis.state@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import type * as Types from "./types.ts";
import {
  StateAdminDeleteRequestSchema,
  StateAdminDeleteResponseSchema,
  StateAdminGetRequestSchema,
  StateAdminGetResponseSchema,
  StateAdminListRequestSchema,
  StateAdminListResponseSchema,
  StateDeleteRequestSchema,
  StateDeleteResponseSchema,
  StateGetRequestSchema,
  StateGetResponseSchema,
  StateListRequestSchema,
  StateListResponseSchema,
  StatePutRequestSchema,
  StatePutResponseSchema,
} from "./schemas.ts";

const CONTRACT_ID = "trellis.state@v1" as const;

export const StateAdminDelete = rpcAction(CONTRACT_ID, "State.Admin.Delete", {
  subject: "rpc.v1.State.Admin.Delete",
  input: schema<Types.StateAdminDeleteInput>(StateAdminDeleteRequestSchema),
  output: schema<Types.StateAdminDeleteOutput>(StateAdminDeleteResponseSchema),
  callerCapabilities: ["admin"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateAdminDelete");

export const StateAdminGet = rpcAction(CONTRACT_ID, "State.Admin.Get", {
  subject: "rpc.v1.State.Admin.Get",
  input: schema<Types.StateAdminGetInput>(StateAdminGetRequestSchema),
  output: schema<Types.StateAdminGetOutput>(StateAdminGetResponseSchema),
  callerCapabilities: ["admin"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateAdminGet");

export const StateAdminList = rpcAction(CONTRACT_ID, "State.Admin.List", {
  subject: "rpc.v1.State.Admin.List",
  input: schema<Types.StateAdminListInput>(StateAdminListRequestSchema),
  output: schema<Types.StateAdminListOutput>(StateAdminListResponseSchema),
  callerCapabilities: ["admin"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateAdminList");

export const StateDelete = rpcAction(CONTRACT_ID, "State.Delete", {
  subject: "rpc.v1.State.Delete",
  input: schema<Types.StateDeleteInput>(StateDeleteRequestSchema),
  output: schema<Types.StateDeleteOutput>(StateDeleteResponseSchema),
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateDelete");

export const StateGet = rpcAction(CONTRACT_ID, "State.Get", {
  subject: "rpc.v1.State.Get",
  input: schema<Types.StateGetInput>(StateGetRequestSchema),
  output: schema<Types.StateGetOutput>(StateGetResponseSchema),
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateGet");

export const StateList = rpcAction(CONTRACT_ID, "State.List", {
  subject: "rpc.v1.State.List",
  input: schema<Types.StateListInput>(StateListRequestSchema),
  output: schema<Types.StateListOutput>(StateListResponseSchema),
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StateList");

export const StatePut = rpcAction(CONTRACT_ID, "State.Put", {
  subject: "rpc.v1.State.Put",
  input: schema<Types.StatePutInput>(StatePutRequestSchema),
  output: schema<Types.StatePutOutput>(StatePutResponseSchema),
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
}, "StatePut");
