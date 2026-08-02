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
import {
  CONTRACT as ACTION_ARTIFACT,
  CONTRACT_DIGEST as ACTION_DIGEST,
} from "./manifest.ts";

const ACTION_SOURCE = {
  artifact: ACTION_ARTIFACT,
  digest: ACTION_DIGEST,
} as const;

const CONTRACT_ID = "trellis.state@v1" as const;

export const StateAdminDelete = rpcAction(
  CONTRACT_ID,
  "State.Admin.Delete",
  {
    subject: "rpc.v1.State.Admin.Delete",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Admin.Delete",
      action: "call",
    }),
    input: schema<Types.StateAdminDeleteInput>(StateAdminDeleteRequestSchema),
    output: schema<Types.StateAdminDeleteOutput>(
      StateAdminDeleteResponseSchema,
    ),
    callerCapabilities: ["admin"] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateAdminDelete",
  ACTION_SOURCE,
);

export const StateAdminGet = rpcAction(
  CONTRACT_ID,
  "State.Admin.Get",
  {
    subject: "rpc.v1.State.Admin.Get",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Admin.Get",
      action: "call",
    }),
    input: schema<Types.StateAdminGetInput>(StateAdminGetRequestSchema),
    output: schema<Types.StateAdminGetOutput>(StateAdminGetResponseSchema),
    callerCapabilities: ["admin"] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateAdminGet",
  ACTION_SOURCE,
);

export const StateAdminList = rpcAction(
  CONTRACT_ID,
  "State.Admin.List",
  {
    subject: "rpc.v1.State.Admin.List",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Admin.List",
      action: "call",
    }),
    input: schema<Types.StateAdminListInput>(StateAdminListRequestSchema),
    output: schema<Types.StateAdminListOutput>(StateAdminListResponseSchema),
    callerCapabilities: ["admin"] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateAdminList",
  ACTION_SOURCE,
);

export const StateDelete = rpcAction(
  CONTRACT_ID,
  "State.Delete",
  {
    subject: "rpc.v1.State.Delete",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Delete",
      action: "call",
    }),
    input: schema<Types.StateDeleteInput>(StateDeleteRequestSchema),
    output: schema<Types.StateDeleteOutput>(StateDeleteResponseSchema),
    callerCapabilities: [] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateDelete",
  ACTION_SOURCE,
);

export const StateGet = rpcAction(
  CONTRACT_ID,
  "State.Get",
  {
    subject: "rpc.v1.State.Get",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Get",
      action: "call",
    }),
    input: schema<Types.StateGetInput>(StateGetRequestSchema),
    output: schema<Types.StateGetOutput>(StateGetResponseSchema),
    callerCapabilities: [] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateGet",
  ACTION_SOURCE,
);

export const StateList = rpcAction(
  CONTRACT_ID,
  "State.List",
  {
    subject: "rpc.v1.State.List",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.List",
      action: "call",
    }),
    input: schema<Types.StateListInput>(StateListRequestSchema),
    output: schema<Types.StateListOutput>(StateListResponseSchema),
    callerCapabilities: [] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StateList",
  ACTION_SOURCE,
);

export const StatePut = rpcAction(
  CONTRACT_ID,
  "State.Put",
  {
    subject: "rpc.v1.State.Put",
    permission: Object.freeze({
      apiId: "trellis.state@v1",
      apiVersion: "v1",
      surfaceKind: "rpc",
      surfaceName: "State.Put",
      action: "call",
    }),
    input: schema<Types.StatePutInput>(StatePutRequestSchema),
    output: schema<Types.StatePutOutput>(StatePutResponseSchema),
    callerCapabilities: [] as const,
    errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
    declaredErrorTypes: [
      "AuthError",
      "UnexpectedError",
      "ValidationError",
    ] as const,
  },
  "StatePut",
  ACTION_SOURCE,
);
