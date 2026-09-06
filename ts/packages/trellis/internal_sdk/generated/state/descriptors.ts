// Generated from trellis.state@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "@qlever-llc/trellis";
import * as Types from "./types.ts";
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
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.state@v1" as const;

const __StateAdminDeleteDescriptor = {
  subject: "rpc.v1.State.Admin.Delete",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Admin.Delete",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Admin.Delete";
    readonly action: "call";
  },
  input: schema<Types.StateAdminDeleteInput>(
    StateAdminDeleteRequestSchema,
  ) as ReturnType<typeof schema<Types.StateAdminDeleteInput>>,
  output: schema<Types.StateAdminDeleteOutput>(
    StateAdminDeleteResponseSchema,
  ) as ReturnType<typeof schema<Types.StateAdminDeleteOutput>>,
  callerCapabilities: ["trellis.state::mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateAdminDelete: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "State.Admin.Delete",
    typeof __StateAdminDeleteDescriptor
  >
> = rpcAction(
  API_ID,
  "State.Admin.Delete",
  __StateAdminDeleteDescriptor,
  "StateAdminDelete",
  ACTION_SOURCE,
);

const __StateAdminGetDescriptor = {
  subject: "rpc.v1.State.Admin.Get",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Admin.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Admin.Get";
    readonly action: "call";
  },
  input: schema<Types.StateAdminGetInput>(
    StateAdminGetRequestSchema,
  ) as ReturnType<typeof schema<Types.StateAdminGetInput>>,
  output: schema<Types.StateAdminGetOutput>(
    StateAdminGetResponseSchema,
  ) as ReturnType<typeof schema<Types.StateAdminGetOutput>>,
  callerCapabilities: ["trellis.state::read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateAdminGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "State.Admin.Get",
    typeof __StateAdminGetDescriptor
  >
> = rpcAction(
  API_ID,
  "State.Admin.Get",
  __StateAdminGetDescriptor,
  "StateAdminGet",
  ACTION_SOURCE,
);

const __StateAdminListDescriptor = {
  subject: "rpc.v1.State.Admin.List",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Admin.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Admin.List";
    readonly action: "call";
  },
  input: schema<Types.StateAdminListInput>(
    StateAdminListRequestSchema,
  ) as ReturnType<typeof schema<Types.StateAdminListInput>>,
  output: schema<Types.StateAdminListOutput>(
    StateAdminListResponseSchema,
  ) as ReturnType<typeof schema<Types.StateAdminListOutput>>,
  callerCapabilities: ["trellis.state::read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateAdminList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "State.Admin.List",
    typeof __StateAdminListDescriptor
  >
> = rpcAction(
  API_ID,
  "State.Admin.List",
  __StateAdminListDescriptor,
  "StateAdminList",
  ACTION_SOURCE,
);

const __StateDeleteDescriptor = {
  subject: "rpc.v1.State.Delete",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Delete",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Delete";
    readonly action: "call";
  },
  input: schema<Types.StateDeleteInput>(StateDeleteRequestSchema) as ReturnType<
    typeof schema<Types.StateDeleteInput>
  >,
  output: schema<Types.StateDeleteOutput>(
    StateDeleteResponseSchema,
  ) as ReturnType<typeof schema<Types.StateDeleteOutput>>,
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateDelete: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "State.Delete",
    typeof __StateDeleteDescriptor
  >
> = rpcAction(
  API_ID,
  "State.Delete",
  __StateDeleteDescriptor,
  "StateDelete",
  ACTION_SOURCE,
);

const __StateGetDescriptor = {
  subject: "rpc.v1.State.Get",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Get";
    readonly action: "call";
  },
  input: schema<Types.StateGetInput>(StateGetRequestSchema) as ReturnType<
    typeof schema<Types.StateGetInput>
  >,
  output: schema<Types.StateGetOutput>(StateGetResponseSchema) as ReturnType<
    typeof schema<Types.StateGetOutput>
  >,
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateGet: ReturnType<
  typeof rpcAction<typeof API_ID, "State.Get", typeof __StateGetDescriptor>
> = rpcAction(
  API_ID,
  "State.Get",
  __StateGetDescriptor,
  "StateGet",
  ACTION_SOURCE,
);

const __StateListDescriptor = {
  subject: "rpc.v1.State.List",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.List";
    readonly action: "call";
  },
  input: schema<Types.StateListInput>(StateListRequestSchema) as ReturnType<
    typeof schema<Types.StateListInput>
  >,
  output: schema<Types.StateListOutput>(StateListResponseSchema) as ReturnType<
    typeof schema<Types.StateListOutput>
  >,
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StateList: ReturnType<
  typeof rpcAction<typeof API_ID, "State.List", typeof __StateListDescriptor>
> = rpcAction(
  API_ID,
  "State.List",
  __StateListDescriptor,
  "StateList",
  ACTION_SOURCE,
);

const __StatePutDescriptor = {
  subject: "rpc.v1.State.Put",
  permission: Object.freeze({
    apiId: "trellis.state@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "State.Put",
    action: "call",
  }) as {
    readonly apiId: "trellis.state@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "State.Put";
    readonly action: "call";
  },
  input: schema<Types.StatePutInput>(StatePutRequestSchema) as ReturnType<
    typeof schema<Types.StatePutInput>
  >,
  output: schema<Types.StatePutOutput>(StatePutResponseSchema) as ReturnType<
    typeof schema<Types.StatePutOutput>
  >,
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const StatePut: ReturnType<
  typeof rpcAction<typeof API_ID, "State.Put", typeof __StatePutDescriptor>
> = rpcAction(
  API_ID,
  "State.Put",
  __StatePutDescriptor,
  "StatePut",
  ACTION_SOURCE,
);

export const ACTIONS = {
  "State.Admin.Delete": StateAdminDelete as typeof StateAdminDelete,
  "State.Admin.Get": StateAdminGet as typeof StateAdminGet,
  "State.Admin.List": StateAdminList as typeof StateAdminList,
  "State.Delete": StateDelete as typeof StateDelete,
  "State.Get": StateGet as typeof StateGet,
  "State.List": StateList as typeof StateList,
  "State.Put": StatePut as typeof StatePut,
} as const;
