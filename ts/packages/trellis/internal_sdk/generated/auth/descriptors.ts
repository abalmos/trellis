// Generated from trellis.auth@v1
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../generated.ts";
import * as Types from "./types.ts";
import {
  AuthCapabilitiesListRequestSchema,
  AuthCapabilitiesListResponseSchema,
  AuthCapabilityGroupsDeleteRequestSchema,
  AuthCapabilityGroupsDeleteResponseSchema,
  AuthCapabilityGroupsGetRequestSchema,
  AuthCapabilityGroupsGetResponseSchema,
  AuthCapabilityGroupsListRequestSchema,
  AuthCapabilityGroupsListResponseSchema,
  AuthCapabilityGroupsPutRequestSchema,
  AuthCapabilityGroupsPutResponseSchema,
  AuthConnectionsClosedEventSchema,
  AuthConnectionsKickedEventSchema,
  AuthConnectionsKickRequestSchema,
  AuthConnectionsKickResponseSchema,
  AuthConnectionsListRequestSchema,
  AuthConnectionsListResponseSchema,
  AuthConnectionsOpenedEventSchema,
  AuthDeploymentsApplyRequestSchema,
  AuthDeploymentsApplyResponseSchema,
  AuthDeploymentsCreateRequestSchema,
  AuthDeploymentsCreateResponseSchema,
  AuthDeploymentsDisableRequestSchema,
  AuthDeploymentsDisableResponseSchema,
  AuthDeploymentsEnableRequestSchema,
  AuthDeploymentsEnableResponseSchema,
  AuthDeploymentsGetRequestSchema,
  AuthDeploymentsGetResponseSchema,
  AuthDeploymentsListRequestSchema,
  AuthDeploymentsListResponseSchema,
  AuthDeploymentsRemoveRequestSchema,
  AuthDeploymentsRemoveResponseSchema,
  AuthDevicesDisableRequestSchema,
  AuthDevicesDisableResponseSchema,
  AuthDevicesEnableRequestSchema,
  AuthDevicesEnableResponseSchema,
  AuthDevicesListRequestSchema,
  AuthDevicesListResponseSchema,
  AuthDevicesProvisionRequestSchema,
  AuthDevicesProvisionResponseSchema,
  AuthDevicesRemoveRequestSchema,
  AuthDevicesRemoveResponseSchema,
  AuthDeviceUserAuthoritiesApprovedEventSchema,
  AuthDeviceUserAuthoritiesListRequestSchema,
  AuthDeviceUserAuthoritiesListResponseSchema,
  AuthDeviceUserAuthoritiesRequestedEventSchema,
  AuthDeviceUserAuthoritiesResolvedEventSchema,
  AuthDeviceUserAuthoritiesResolveProgressSchema,
  AuthDeviceUserAuthoritiesResolveRequestSchema,
  AuthDeviceUserAuthoritiesResolveResponseSchema,
  AuthDeviceUserAuthoritiesReviewRequestedEventSchema,
  AuthDeviceUserAuthoritiesReviewsDecideRequestSchema,
  AuthDeviceUserAuthoritiesReviewsDecideResponseSchema,
  AuthDeviceUserAuthoritiesReviewsListRequestSchema,
  AuthDeviceUserAuthoritiesReviewsListResponseSchema,
  AuthDeviceUserAuthoritiesRevokeRequestSchema,
  AuthDeviceUserAuthoritiesRevokeResponseSchema,
  AuthErrorDetailsSchema,
  AuthGrantsChangedEventSchema,
  AuthGrantsGetRequestSchema,
  AuthGrantsGetResponseSchema,
  AuthGrantsListRequestSchema,
  AuthGrantsListResponseSchema,
  AuthGrantsMutationResponseSchema,
  AuthGrantsRevokeRequestSchema,
  AuthGrantsSetRequestSchema,
  AuthIssuersRevokedEventSchema,
  AuthIssuersRevokeRequestSchema,
  AuthIssuersRevokeResponseSchema,
  AuthParticipantsGetRequestSchema,
  AuthParticipantsGetResponseSchema,
  AuthParticipantsInstallRequestSchema,
  AuthParticipantsInstallResponseSchema,
  AuthPortalsGetRequestSchema,
  AuthPortalsGetResponseSchema,
  AuthPortalsGrantOverridesListRequestSchema,
  AuthPortalsGrantOverridesListResponseSchema,
  AuthPortalsGrantOverridesPutRequestSchema,
  AuthPortalsGrantOverridesPutResponseSchema,
  AuthPortalsGrantOverridesRemoveRequestSchema,
  AuthPortalsGrantOverridesRemoveResponseSchema,
  AuthPortalsListRequestSchema,
  AuthPortalsListResponseSchema,
  AuthPortalsLoginSettingsGetRequestSchema,
  AuthPortalsLoginSettingsGetResponseSchema,
  AuthPortalsLoginSettingsUpdateRequestSchema,
  AuthPortalsLoginSettingsUpdateResponseSchema,
  AuthPortalsPutRequestSchema,
  AuthPortalsPutResponseSchema,
  AuthPortalsRemoveRequestSchema,
  AuthPortalsRemoveResponseSchema,
  AuthPortalsRoutesPutRequestSchema,
  AuthPortalsRoutesPutResponseSchema,
  AuthPortalsRoutesRemoveRequestSchema,
  AuthPortalsRoutesRemoveResponseSchema,
  AuthServiceInstancesDisableRequestSchema,
  AuthServiceInstancesDisableResponseSchema,
  AuthServiceInstancesEnableRequestSchema,
  AuthServiceInstancesEnableResponseSchema,
  AuthServiceInstancesListRequestSchema,
  AuthServiceInstancesListResponseSchema,
  AuthServiceInstancesProvisionRequestSchema,
  AuthServiceInstancesProvisionResponseSchema,
  AuthServiceInstancesRemoveRequestSchema,
  AuthServiceInstancesRemoveResponseSchema,
  AuthSessionsListRequestSchema,
  AuthSessionsListResponseSchema,
  AuthSessionsLogoutRequestSchema,
  AuthSessionsLogoutResponseSchema,
  AuthSessionsMeRequestSchema,
  AuthSessionsMeResponseSchema,
  AuthSessionsRevokedEventSchema,
  AuthSessionsRevokeRequestSchema,
  AuthSessionsRevokeResponseSchema,
  AuthUserIdentitiesListRequestSchema,
  AuthUserIdentitiesListResponseSchema,
  AuthUserIdentitiesUnlinkRequestSchema,
  AuthUserIdentitiesUnlinkResponseSchema,
  AuthUsersCreateRequestSchema,
  AuthUsersCreateResponseSchema,
  AuthUsersGetRequestSchema,
  AuthUsersGetResponseSchema,
  AuthUsersIdentityLinkCreateRequestSchema,
  AuthUsersIdentityLinkCreateResponseSchema,
  AuthUsersListRequestSchema,
  AuthUsersListResponseSchema,
  AuthUsersPasswordChangeRequestSchema,
  AuthUsersPasswordChangeResponseSchema,
  AuthUsersPasswordResetCreateRequestSchema,
  AuthUsersPasswordResetCreateResponseSchema,
  AuthUsersResolveRequestSchema,
  AuthUsersResolveResponseSchema,
  AuthUsersUpdateRequestSchema,
  AuthUsersUpdateResponseSchema,
} from "./schemas.ts";
import { API as ACTION_ARTIFACT, API_DIGEST as ACTION_DIGEST } from "./api.ts";

const ACTION_SOURCE = {
  api: ACTION_ARTIFACT,
  apiDigest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.auth@v1" as const;

const __AuthCapabilitiesListDescriptor = {
  subject: "rpc.v1.Auth.Capabilities.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Capabilities.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Capabilities.List";
    readonly action: "call";
  },
  input: schema<Types.AuthCapabilitiesListInput>(
    AuthCapabilitiesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilitiesListInput>>,
  output: schema<Types.AuthCapabilitiesListOutput>(
    AuthCapabilitiesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilitiesListOutput>>,
  callerCapabilities: ["trellis.auth::capabilities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthCapabilitiesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Capabilities.List",
    typeof __AuthCapabilitiesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Capabilities.List",
  __AuthCapabilitiesListDescriptor,
  "AuthCapabilitiesList",
  ACTION_SOURCE,
);

const __AuthCapabilityGroupsDeleteDescriptor = {
  subject: "rpc.v1.Auth.CapabilityGroups.Delete",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.CapabilityGroups.Delete",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.CapabilityGroups.Delete";
    readonly action: "call";
  },
  input: schema<Types.AuthCapabilityGroupsDeleteInput>(
    AuthCapabilityGroupsDeleteRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsDeleteInput>>,
  output: schema<Types.AuthCapabilityGroupsDeleteOutput>(
    AuthCapabilityGroupsDeleteResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsDeleteOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthCapabilityGroupsDelete: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.CapabilityGroups.Delete",
    typeof __AuthCapabilityGroupsDeleteDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.CapabilityGroups.Delete",
  __AuthCapabilityGroupsDeleteDescriptor,
  "AuthCapabilityGroupsDelete",
  ACTION_SOURCE,
);

const __AuthCapabilityGroupsGetDescriptor = {
  subject: "rpc.v1.Auth.CapabilityGroups.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.CapabilityGroups.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.CapabilityGroups.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthCapabilityGroupsGetInput>(
    AuthCapabilityGroupsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsGetInput>>,
  output: schema<Types.AuthCapabilityGroupsGetOutput>(
    AuthCapabilityGroupsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsGetOutput>>,
  callerCapabilities: ["trellis.auth::capabilities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthCapabilityGroupsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.CapabilityGroups.Get",
    typeof __AuthCapabilityGroupsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.CapabilityGroups.Get",
  __AuthCapabilityGroupsGetDescriptor,
  "AuthCapabilityGroupsGet",
  ACTION_SOURCE,
);

const __AuthCapabilityGroupsListDescriptor = {
  subject: "rpc.v1.Auth.CapabilityGroups.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.CapabilityGroups.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.CapabilityGroups.List";
    readonly action: "call";
  },
  input: schema<Types.AuthCapabilityGroupsListInput>(
    AuthCapabilityGroupsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsListInput>>,
  output: schema<Types.AuthCapabilityGroupsListOutput>(
    AuthCapabilityGroupsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsListOutput>>,
  callerCapabilities: ["trellis.auth::capabilities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthCapabilityGroupsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.CapabilityGroups.List",
    typeof __AuthCapabilityGroupsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.CapabilityGroups.List",
  __AuthCapabilityGroupsListDescriptor,
  "AuthCapabilityGroupsList",
  ACTION_SOURCE,
);

const __AuthCapabilityGroupsPutDescriptor = {
  subject: "rpc.v1.Auth.CapabilityGroups.Put",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.CapabilityGroups.Put",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.CapabilityGroups.Put";
    readonly action: "call";
  },
  input: schema<Types.AuthCapabilityGroupsPutInput>(
    AuthCapabilityGroupsPutRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsPutInput>>,
  output: schema<Types.AuthCapabilityGroupsPutOutput>(
    AuthCapabilityGroupsPutResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthCapabilityGroupsPutOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthCapabilityGroupsPut: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.CapabilityGroups.Put",
    typeof __AuthCapabilityGroupsPutDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.CapabilityGroups.Put",
  __AuthCapabilityGroupsPutDescriptor,
  "AuthCapabilityGroupsPut",
  ACTION_SOURCE,
);

const __AuthConnectionsKickDescriptor = {
  subject: "rpc.v1.Auth.Connections.Kick",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Connections.Kick",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Connections.Kick";
    readonly action: "call";
  },
  input: schema<Types.AuthConnectionsKickInput>(
    AuthConnectionsKickRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsKickInput>>,
  output: schema<Types.AuthConnectionsKickOutput>(
    AuthConnectionsKickResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsKickOutput>>,
  callerCapabilities: ["trellis.auth::connections.kick"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthConnectionsKick: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Connections.Kick",
    typeof __AuthConnectionsKickDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Connections.Kick",
  __AuthConnectionsKickDescriptor,
  "AuthConnectionsKick",
  ACTION_SOURCE,
);

const __AuthConnectionsListDescriptor = {
  subject: "rpc.v1.Auth.Connections.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Connections.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Connections.List";
    readonly action: "call";
  },
  input: schema<Types.AuthConnectionsListInput>(
    AuthConnectionsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsListInput>>,
  output: schema<Types.AuthConnectionsListOutput>(
    AuthConnectionsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsListOutput>>,
  callerCapabilities: ["trellis.auth::connections.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthConnectionsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Connections.List",
    typeof __AuthConnectionsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Connections.List",
  __AuthConnectionsListDescriptor,
  "AuthConnectionsList",
  ACTION_SOURCE,
);

const __AuthDeploymentsApplyDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Apply",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Apply",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Apply";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsApplyInput>(
    AuthDeploymentsApplyRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsApplyInput>>,
  output: schema<Types.AuthDeploymentsApplyOutput>(
    AuthDeploymentsApplyResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsApplyOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsApply: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Apply",
    typeof __AuthDeploymentsApplyDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Apply",
  __AuthDeploymentsApplyDescriptor,
  "AuthDeploymentsApply",
  ACTION_SOURCE,
);

const __AuthDeploymentsCreateDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Create",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Create",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Create";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsCreateInput>(
    AuthDeploymentsCreateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsCreateInput>>,
  output: schema<Types.AuthDeploymentsCreateOutput>(
    AuthDeploymentsCreateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsCreateOutput>>,
  callerCapabilities: ["trellis.auth::deployments.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsCreate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Create",
    typeof __AuthDeploymentsCreateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Create",
  __AuthDeploymentsCreateDescriptor,
  "AuthDeploymentsCreate",
  ACTION_SOURCE,
);

const __AuthDeploymentsDisableDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Disable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Disable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Disable";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsDisableInput>(
    AuthDeploymentsDisableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsDisableInput>>,
  output: schema<Types.AuthDeploymentsDisableOutput>(
    AuthDeploymentsDisableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsDisableOutput>>,
  callerCapabilities: ["trellis.auth::deployments.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsDisable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Disable",
    typeof __AuthDeploymentsDisableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Disable",
  __AuthDeploymentsDisableDescriptor,
  "AuthDeploymentsDisable",
  ACTION_SOURCE,
);

const __AuthDeploymentsEnableDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Enable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Enable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Enable";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsEnableInput>(
    AuthDeploymentsEnableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsEnableInput>>,
  output: schema<Types.AuthDeploymentsEnableOutput>(
    AuthDeploymentsEnableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsEnableOutput>>,
  callerCapabilities: ["trellis.auth::deployments.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsEnable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Enable",
    typeof __AuthDeploymentsEnableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Enable",
  __AuthDeploymentsEnableDescriptor,
  "AuthDeploymentsEnable",
  ACTION_SOURCE,
);

const __AuthDeploymentsGetDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsGetInput>(
    AuthDeploymentsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsGetInput>>,
  output: schema<Types.AuthDeploymentsGetOutput>(
    AuthDeploymentsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsGetOutput>>,
  callerCapabilities: ["trellis.auth::deployments.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Get",
    typeof __AuthDeploymentsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Get",
  __AuthDeploymentsGetDescriptor,
  "AuthDeploymentsGet",
  ACTION_SOURCE,
);

const __AuthDeploymentsListDescriptor = {
  subject: "rpc.v1.Auth.Deployments.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsListInput>(
    AuthDeploymentsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsListInput>>,
  output: schema<Types.AuthDeploymentsListOutput>(
    AuthDeploymentsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsListOutput>>,
  callerCapabilities: ["trellis.auth::deployments.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.List",
    typeof __AuthDeploymentsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.List",
  __AuthDeploymentsListDescriptor,
  "AuthDeploymentsList",
  ACTION_SOURCE,
);

const __AuthDeploymentsRemoveDescriptor = {
  subject: "rpc.v1.Auth.Deployments.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Deployments.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Deployments.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentsRemoveInput>(
    AuthDeploymentsRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsRemoveInput>>,
  output: schema<Types.AuthDeploymentsRemoveOutput>(
    AuthDeploymentsRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentsRemoveOutput>>,
  callerCapabilities: ["trellis.auth::deployments.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeploymentsRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Deployments.Remove",
    typeof __AuthDeploymentsRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Deployments.Remove",
  __AuthDeploymentsRemoveDescriptor,
  "AuthDeploymentsRemove",
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesListDescriptor = {
  subject: "rpc.v1.Auth.DeviceUserAuthorities.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeviceUserAuthorities.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeviceUserAuthorities.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDeviceUserAuthoritiesListInput>(
    AuthDeviceUserAuthoritiesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesListInput>>,
  output: schema<Types.AuthDeviceUserAuthoritiesListOutput>(
    AuthDeviceUserAuthoritiesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesListOutput>>,
  callerCapabilities: ["trellis.auth::devices.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeviceUserAuthoritiesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.List",
    typeof __AuthDeviceUserAuthoritiesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.List",
  __AuthDeviceUserAuthoritiesListDescriptor,
  "AuthDeviceUserAuthoritiesList",
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesReviewsDecideDescriptor = {
  subject: "rpc.v1.Auth.DeviceUserAuthorities.Reviews.Decide",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeviceUserAuthorities.Reviews.Decide",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Reviews.Decide";
    readonly action: "call";
  },
  input: schema<Types.AuthDeviceUserAuthoritiesReviewsDecideInput>(
    AuthDeviceUserAuthoritiesReviewsDecideRequestSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesReviewsDecideInput>
  >,
  output: schema<Types.AuthDeviceUserAuthoritiesReviewsDecideOutput>(
    AuthDeviceUserAuthoritiesReviewsDecideResponseSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesReviewsDecideOutput>
  >,
  callerCapabilities: ["trellis.auth::devices.review"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeviceUserAuthoritiesReviewsDecide: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Reviews.Decide",
    typeof __AuthDeviceUserAuthoritiesReviewsDecideDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Reviews.Decide",
  __AuthDeviceUserAuthoritiesReviewsDecideDescriptor,
  "AuthDeviceUserAuthoritiesReviewsDecide",
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesReviewsListDescriptor = {
  subject: "rpc.v1.Auth.DeviceUserAuthorities.Reviews.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeviceUserAuthorities.Reviews.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Reviews.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDeviceUserAuthoritiesReviewsListInput>(
    AuthDeviceUserAuthoritiesReviewsListRequestSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesReviewsListInput>
  >,
  output: schema<Types.AuthDeviceUserAuthoritiesReviewsListOutput>(
    AuthDeviceUserAuthoritiesReviewsListResponseSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesReviewsListOutput>
  >,
  callerCapabilities: ["trellis.auth::devices.review"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeviceUserAuthoritiesReviewsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Reviews.List",
    typeof __AuthDeviceUserAuthoritiesReviewsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Reviews.List",
  __AuthDeviceUserAuthoritiesReviewsListDescriptor,
  "AuthDeviceUserAuthoritiesReviewsList",
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesRevokeDescriptor = {
  subject: "rpc.v1.Auth.DeviceUserAuthorities.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeviceUserAuthorities.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthDeviceUserAuthoritiesRevokeInput>(
    AuthDeviceUserAuthoritiesRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesRevokeInput>>,
  output: schema<Types.AuthDeviceUserAuthoritiesRevokeOutput>(
    AuthDeviceUserAuthoritiesRevokeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesRevokeOutput>>,
  callerCapabilities: ["trellis.auth::devices.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDeviceUserAuthoritiesRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Revoke",
    typeof __AuthDeviceUserAuthoritiesRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Revoke",
  __AuthDeviceUserAuthoritiesRevokeDescriptor,
  "AuthDeviceUserAuthoritiesRevoke",
  ACTION_SOURCE,
);

const __AuthDevicesDisableDescriptor = {
  subject: "rpc.v1.Auth.Devices.Disable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.Disable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.Disable";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesDisableInput>(
    AuthDevicesDisableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesDisableInput>>,
  output: schema<Types.AuthDevicesDisableOutput>(
    AuthDevicesDisableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesDisableOutput>>,
  callerCapabilities: ["trellis.auth::devices.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDevicesDisable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.Disable",
    typeof __AuthDevicesDisableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.Disable",
  __AuthDevicesDisableDescriptor,
  "AuthDevicesDisable",
  ACTION_SOURCE,
);

const __AuthDevicesEnableDescriptor = {
  subject: "rpc.v1.Auth.Devices.Enable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.Enable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.Enable";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesEnableInput>(
    AuthDevicesEnableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesEnableInput>>,
  output: schema<Types.AuthDevicesEnableOutput>(
    AuthDevicesEnableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesEnableOutput>>,
  callerCapabilities: ["trellis.auth::devices.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDevicesEnable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.Enable",
    typeof __AuthDevicesEnableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.Enable",
  __AuthDevicesEnableDescriptor,
  "AuthDevicesEnable",
  ACTION_SOURCE,
);

const __AuthDevicesListDescriptor = {
  subject: "rpc.v1.Auth.Devices.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesListInput>(
    AuthDevicesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesListInput>>,
  output: schema<Types.AuthDevicesListOutput>(
    AuthDevicesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesListOutput>>,
  callerCapabilities: ["trellis.auth::devices.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDevicesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.List",
    typeof __AuthDevicesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.List",
  __AuthDevicesListDescriptor,
  "AuthDevicesList",
  ACTION_SOURCE,
);

const __AuthDevicesProvisionDescriptor = {
  subject: "rpc.v1.Auth.Devices.Provision",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.Provision",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.Provision";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesProvisionInput>(
    AuthDevicesProvisionRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesProvisionInput>>,
  output: schema<Types.AuthDevicesProvisionOutput>(
    AuthDevicesProvisionResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesProvisionOutput>>,
  callerCapabilities: ["trellis.auth::devices.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDevicesProvision: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.Provision",
    typeof __AuthDevicesProvisionDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.Provision",
  __AuthDevicesProvisionDescriptor,
  "AuthDevicesProvision",
  ACTION_SOURCE,
);

const __AuthDevicesRemoveDescriptor = {
  subject: "rpc.v1.Auth.Devices.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesRemoveInput>(
    AuthDevicesRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesRemoveInput>>,
  output: schema<Types.AuthDevicesRemoveOutput>(
    AuthDevicesRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesRemoveOutput>>,
  callerCapabilities: ["trellis.auth::devices.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthDevicesRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.Remove",
    typeof __AuthDevicesRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.Remove",
  __AuthDevicesRemoveDescriptor,
  "AuthDevicesRemove",
  ACTION_SOURCE,
);

const __AuthGrantsGetDescriptor = {
  subject: "rpc.v1.Auth.Grants.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Grants.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Grants.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthGrantsGetInput>(
    AuthGrantsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsGetInput>>,
  output: schema<Types.AuthGrantsGetOutput>(
    AuthGrantsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsGetOutput>>,
  callerCapabilities: ["trellis.auth::authorities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthGrantsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Grants.Get",
    typeof __AuthGrantsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Grants.Get",
  __AuthGrantsGetDescriptor,
  "AuthGrantsGet",
  ACTION_SOURCE,
);

const __AuthGrantsListDescriptor = {
  subject: "rpc.v1.Auth.Grants.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Grants.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Grants.List";
    readonly action: "call";
  },
  input: schema<Types.AuthGrantsListInput>(
    AuthGrantsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsListInput>>,
  output: schema<Types.AuthGrantsListOutput>(
    AuthGrantsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsListOutput>>,
  callerCapabilities: ["trellis.auth::authorities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthGrantsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Grants.List",
    typeof __AuthGrantsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Grants.List",
  __AuthGrantsListDescriptor,
  "AuthGrantsList",
  ACTION_SOURCE,
);

const __AuthGrantsRevokeDescriptor = {
  subject: "rpc.v1.Auth.Grants.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Grants.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Grants.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthGrantsRevokeInput>(
    AuthGrantsRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsRevokeInput>>,
  output: schema<Types.AuthGrantsRevokeOutput>(
    AuthGrantsMutationResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsRevokeOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthGrantsRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Grants.Revoke",
    typeof __AuthGrantsRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Grants.Revoke",
  __AuthGrantsRevokeDescriptor,
  "AuthGrantsRevoke",
  ACTION_SOURCE,
);

const __AuthGrantsSetDescriptor = {
  subject: "rpc.v1.Auth.Grants.Set",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Grants.Set",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Grants.Set";
    readonly action: "call";
  },
  input: schema<Types.AuthGrantsSetInput>(
    AuthGrantsSetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsSetInput>>,
  output: schema<Types.AuthGrantsSetOutput>(
    AuthGrantsMutationResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsSetOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthGrantsSet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Grants.Set",
    typeof __AuthGrantsSetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Grants.Set",
  __AuthGrantsSetDescriptor,
  "AuthGrantsSet",
  ACTION_SOURCE,
);

const __AuthIssuersRevokeDescriptor = {
  subject: "rpc.v1.Auth.Issuers.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Issuers.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Issuers.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthIssuersRevokeInput>(
    AuthIssuersRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIssuersRevokeInput>>,
  output: schema<Types.AuthIssuersRevokeOutput>(
    AuthIssuersRevokeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIssuersRevokeOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthIssuersRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Issuers.Revoke",
    typeof __AuthIssuersRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Issuers.Revoke",
  __AuthIssuersRevokeDescriptor,
  "AuthIssuersRevoke",
  ACTION_SOURCE,
);

const __AuthParticipantsGetDescriptor = {
  subject: "rpc.v1.Auth.Participants.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Participants.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Participants.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthParticipantsGetInput>(
    AuthParticipantsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthParticipantsGetInput>>,
  output: schema<Types.AuthParticipantsGetOutput>(
    AuthParticipantsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthParticipantsGetOutput>>,
  callerCapabilities: ["trellis.auth::authorities.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthParticipantsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Participants.Get",
    typeof __AuthParticipantsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Participants.Get",
  __AuthParticipantsGetDescriptor,
  "AuthParticipantsGet",
  ACTION_SOURCE,
);

const __AuthParticipantsInstallDescriptor = {
  subject: "rpc.v1.Auth.Participants.Install",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Participants.Install",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Participants.Install";
    readonly action: "call";
  },
  input: schema<Types.AuthParticipantsInstallInput>(
    AuthParticipantsInstallRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthParticipantsInstallInput>>,
  output: schema<Types.AuthParticipantsInstallOutput>(
    AuthParticipantsInstallResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthParticipantsInstallOutput>>,
  callerCapabilities: ["trellis.auth::authorities.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthParticipantsInstall: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Participants.Install",
    typeof __AuthParticipantsInstallDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Participants.Install",
  __AuthParticipantsInstallDescriptor,
  "AuthParticipantsInstall",
  ACTION_SOURCE,
);

const __AuthPortalsGetDescriptor = {
  subject: "rpc.v1.Auth.Portals.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsGetInput>(
    AuthPortalsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGetInput>>,
  output: schema<Types.AuthPortalsGetOutput>(
    AuthPortalsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGetOutput>>,
  callerCapabilities: ["trellis.auth::portals.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.Get",
    typeof __AuthPortalsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.Get",
  __AuthPortalsGetDescriptor,
  "AuthPortalsGet",
  ACTION_SOURCE,
);

const __AuthPortalsGrantOverridesListDescriptor = {
  subject: "rpc.v1.Auth.Portals.GrantOverrides.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.GrantOverrides.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.GrantOverrides.List";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsGrantOverridesListInput>(
    AuthPortalsGrantOverridesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesListInput>>,
  output: schema<Types.AuthPortalsGrantOverridesListOutput>(
    AuthPortalsGrantOverridesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesListOutput>>,
  callerCapabilities: ["trellis.auth::portals.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsGrantOverridesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.GrantOverrides.List",
    typeof __AuthPortalsGrantOverridesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.GrantOverrides.List",
  __AuthPortalsGrantOverridesListDescriptor,
  "AuthPortalsGrantOverridesList",
  ACTION_SOURCE,
);

const __AuthPortalsGrantOverridesPutDescriptor = {
  subject: "rpc.v1.Auth.Portals.GrantOverrides.Put",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.GrantOverrides.Put",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.GrantOverrides.Put";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsGrantOverridesPutInput>(
    AuthPortalsGrantOverridesPutRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesPutInput>>,
  output: schema<Types.AuthPortalsGrantOverridesPutOutput>(
    AuthPortalsGrantOverridesPutResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesPutOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsGrantOverridesPut: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.GrantOverrides.Put",
    typeof __AuthPortalsGrantOverridesPutDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.GrantOverrides.Put",
  __AuthPortalsGrantOverridesPutDescriptor,
  "AuthPortalsGrantOverridesPut",
  ACTION_SOURCE,
);

const __AuthPortalsGrantOverridesRemoveDescriptor = {
  subject: "rpc.v1.Auth.Portals.GrantOverrides.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.GrantOverrides.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.GrantOverrides.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsGrantOverridesRemoveInput>(
    AuthPortalsGrantOverridesRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesRemoveInput>>,
  output: schema<Types.AuthPortalsGrantOverridesRemoveOutput>(
    AuthPortalsGrantOverridesRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsGrantOverridesRemoveOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsGrantOverridesRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.GrantOverrides.Remove",
    typeof __AuthPortalsGrantOverridesRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.GrantOverrides.Remove",
  __AuthPortalsGrantOverridesRemoveDescriptor,
  "AuthPortalsGrantOverridesRemove",
  ACTION_SOURCE,
);

const __AuthPortalsListDescriptor = {
  subject: "rpc.v1.Auth.Portals.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.List";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsListInput>(
    AuthPortalsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsListInput>>,
  output: schema<Types.AuthPortalsListOutput>(
    AuthPortalsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsListOutput>>,
  callerCapabilities: ["trellis.auth::portals.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.List",
    typeof __AuthPortalsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.List",
  __AuthPortalsListDescriptor,
  "AuthPortalsList",
  ACTION_SOURCE,
);

const __AuthPortalsLoginSettingsGetDescriptor = {
  subject: "rpc.v1.Auth.Portals.LoginSettings.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.LoginSettings.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.LoginSettings.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsLoginSettingsGetInput>(
    AuthPortalsLoginSettingsGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsLoginSettingsGetInput>>,
  output: schema<Types.AuthPortalsLoginSettingsGetOutput>(
    AuthPortalsLoginSettingsGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsLoginSettingsGetOutput>>,
  callerCapabilities: ["trellis.auth::portals.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsLoginSettingsGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.LoginSettings.Get",
    typeof __AuthPortalsLoginSettingsGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.LoginSettings.Get",
  __AuthPortalsLoginSettingsGetDescriptor,
  "AuthPortalsLoginSettingsGet",
  ACTION_SOURCE,
);

const __AuthPortalsLoginSettingsUpdateDescriptor = {
  subject: "rpc.v1.Auth.Portals.LoginSettings.Update",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.LoginSettings.Update",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.LoginSettings.Update";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsLoginSettingsUpdateInput>(
    AuthPortalsLoginSettingsUpdateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsLoginSettingsUpdateInput>>,
  output: schema<Types.AuthPortalsLoginSettingsUpdateOutput>(
    AuthPortalsLoginSettingsUpdateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsLoginSettingsUpdateOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsLoginSettingsUpdate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.LoginSettings.Update",
    typeof __AuthPortalsLoginSettingsUpdateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.LoginSettings.Update",
  __AuthPortalsLoginSettingsUpdateDescriptor,
  "AuthPortalsLoginSettingsUpdate",
  ACTION_SOURCE,
);

const __AuthPortalsPutDescriptor = {
  subject: "rpc.v1.Auth.Portals.Put",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.Put",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.Put";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsPutInput>(
    AuthPortalsPutRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsPutInput>>,
  output: schema<Types.AuthPortalsPutOutput>(
    AuthPortalsPutResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsPutOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsPut: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.Put",
    typeof __AuthPortalsPutDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.Put",
  __AuthPortalsPutDescriptor,
  "AuthPortalsPut",
  ACTION_SOURCE,
);

const __AuthPortalsRemoveDescriptor = {
  subject: "rpc.v1.Auth.Portals.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsRemoveInput>(
    AuthPortalsRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRemoveInput>>,
  output: schema<Types.AuthPortalsRemoveOutput>(
    AuthPortalsRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRemoveOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.Remove",
    typeof __AuthPortalsRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.Remove",
  __AuthPortalsRemoveDescriptor,
  "AuthPortalsRemove",
  ACTION_SOURCE,
);

const __AuthPortalsRoutesPutDescriptor = {
  subject: "rpc.v1.Auth.Portals.Routes.Put",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.Routes.Put",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.Routes.Put";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsRoutesPutInput>(
    AuthPortalsRoutesPutRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRoutesPutInput>>,
  output: schema<Types.AuthPortalsRoutesPutOutput>(
    AuthPortalsRoutesPutResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRoutesPutOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsRoutesPut: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.Routes.Put",
    typeof __AuthPortalsRoutesPutDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.Routes.Put",
  __AuthPortalsRoutesPutDescriptor,
  "AuthPortalsRoutesPut",
  ACTION_SOURCE,
);

const __AuthPortalsRoutesRemoveDescriptor = {
  subject: "rpc.v1.Auth.Portals.Routes.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Portals.Routes.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Portals.Routes.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthPortalsRoutesRemoveInput>(
    AuthPortalsRoutesRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRoutesRemoveInput>>,
  output: schema<Types.AuthPortalsRoutesRemoveOutput>(
    AuthPortalsRoutesRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthPortalsRoutesRemoveOutput>>,
  callerCapabilities: ["trellis.auth::portals.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthPortalsRoutesRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Portals.Routes.Remove",
    typeof __AuthPortalsRoutesRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Portals.Routes.Remove",
  __AuthPortalsRoutesRemoveDescriptor,
  "AuthPortalsRoutesRemove",
  ACTION_SOURCE,
);

const __AuthServiceInstancesDisableDescriptor = {
  subject: "rpc.v1.Auth.ServiceInstances.Disable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.ServiceInstances.Disable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.ServiceInstances.Disable";
    readonly action: "call";
  },
  input: schema<Types.AuthServiceInstancesDisableInput>(
    AuthServiceInstancesDisableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesDisableInput>>,
  output: schema<Types.AuthServiceInstancesDisableOutput>(
    AuthServiceInstancesDisableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesDisableOutput>>,
  callerCapabilities: ["trellis.auth::services.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthServiceInstancesDisable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.ServiceInstances.Disable",
    typeof __AuthServiceInstancesDisableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Disable",
  __AuthServiceInstancesDisableDescriptor,
  "AuthServiceInstancesDisable",
  ACTION_SOURCE,
);

const __AuthServiceInstancesEnableDescriptor = {
  subject: "rpc.v1.Auth.ServiceInstances.Enable",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.ServiceInstances.Enable",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.ServiceInstances.Enable";
    readonly action: "call";
  },
  input: schema<Types.AuthServiceInstancesEnableInput>(
    AuthServiceInstancesEnableRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesEnableInput>>,
  output: schema<Types.AuthServiceInstancesEnableOutput>(
    AuthServiceInstancesEnableResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesEnableOutput>>,
  callerCapabilities: ["trellis.auth::services.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthServiceInstancesEnable: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.ServiceInstances.Enable",
    typeof __AuthServiceInstancesEnableDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Enable",
  __AuthServiceInstancesEnableDescriptor,
  "AuthServiceInstancesEnable",
  ACTION_SOURCE,
);

const __AuthServiceInstancesListDescriptor = {
  subject: "rpc.v1.Auth.ServiceInstances.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.ServiceInstances.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.ServiceInstances.List";
    readonly action: "call";
  },
  input: schema<Types.AuthServiceInstancesListInput>(
    AuthServiceInstancesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesListInput>>,
  output: schema<Types.AuthServiceInstancesListOutput>(
    AuthServiceInstancesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesListOutput>>,
  callerCapabilities: ["trellis.auth::services.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthServiceInstancesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.ServiceInstances.List",
    typeof __AuthServiceInstancesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.ServiceInstances.List",
  __AuthServiceInstancesListDescriptor,
  "AuthServiceInstancesList",
  ACTION_SOURCE,
);

const __AuthServiceInstancesProvisionDescriptor = {
  subject: "rpc.v1.Auth.ServiceInstances.Provision",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.ServiceInstances.Provision",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.ServiceInstances.Provision";
    readonly action: "call";
  },
  input: schema<Types.AuthServiceInstancesProvisionInput>(
    AuthServiceInstancesProvisionRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesProvisionInput>>,
  output: schema<Types.AuthServiceInstancesProvisionOutput>(
    AuthServiceInstancesProvisionResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesProvisionOutput>>,
  callerCapabilities: ["trellis.auth::services.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthServiceInstancesProvision: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.ServiceInstances.Provision",
    typeof __AuthServiceInstancesProvisionDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Provision",
  __AuthServiceInstancesProvisionDescriptor,
  "AuthServiceInstancesProvision",
  ACTION_SOURCE,
);

const __AuthServiceInstancesRemoveDescriptor = {
  subject: "rpc.v1.Auth.ServiceInstances.Remove",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.ServiceInstances.Remove",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.ServiceInstances.Remove";
    readonly action: "call";
  },
  input: schema<Types.AuthServiceInstancesRemoveInput>(
    AuthServiceInstancesRemoveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesRemoveInput>>,
  output: schema<Types.AuthServiceInstancesRemoveOutput>(
    AuthServiceInstancesRemoveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthServiceInstancesRemoveOutput>>,
  callerCapabilities: ["trellis.auth::services.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthServiceInstancesRemove: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.ServiceInstances.Remove",
    typeof __AuthServiceInstancesRemoveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Remove",
  __AuthServiceInstancesRemoveDescriptor,
  "AuthServiceInstancesRemove",
  ACTION_SOURCE,
);

const __AuthSessionsListDescriptor = {
  subject: "rpc.v1.Auth.Sessions.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Sessions.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Sessions.List";
    readonly action: "call";
  },
  input: schema<Types.AuthSessionsListInput>(
    AuthSessionsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsListInput>>,
  output: schema<Types.AuthSessionsListOutput>(
    AuthSessionsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsListOutput>>,
  callerCapabilities: ["trellis.auth::sessions.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthSessionsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Sessions.List",
    typeof __AuthSessionsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Sessions.List",
  __AuthSessionsListDescriptor,
  "AuthSessionsList",
  ACTION_SOURCE,
);

const __AuthSessionsLogoutDescriptor = {
  subject: "rpc.v1.Auth.Sessions.Logout",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Sessions.Logout",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Sessions.Logout";
    readonly action: "call";
  },
  input: schema<Types.AuthSessionsLogoutInput>(
    AuthSessionsLogoutRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsLogoutInput>>,
  output: schema<Types.AuthSessionsLogoutOutput>(
    AuthSessionsLogoutResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsLogoutOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthSessionsLogout: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Sessions.Logout",
    typeof __AuthSessionsLogoutDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Sessions.Logout",
  __AuthSessionsLogoutDescriptor,
  "AuthSessionsLogout",
  ACTION_SOURCE,
);

const __AuthSessionsMeDescriptor = {
  subject: "rpc.v1.Auth.Sessions.Me",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Sessions.Me",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Sessions.Me";
    readonly action: "call";
  },
  input: schema<Types.AuthSessionsMeInput>(
    AuthSessionsMeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsMeInput>>,
  output: schema<Types.AuthSessionsMeOutput>(
    AuthSessionsMeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsMeOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthSessionsMe: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Sessions.Me",
    typeof __AuthSessionsMeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Sessions.Me",
  __AuthSessionsMeDescriptor,
  "AuthSessionsMe",
  ACTION_SOURCE,
);

const __AuthSessionsRevokeDescriptor = {
  subject: "rpc.v1.Auth.Sessions.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Sessions.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Sessions.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthSessionsRevokeInput>(
    AuthSessionsRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsRevokeInput>>,
  output: schema<Types.AuthSessionsRevokeOutput>(
    AuthSessionsRevokeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsRevokeOutput>>,
  callerCapabilities: ["trellis.auth::sessions.revoke"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthSessionsRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Sessions.Revoke",
    typeof __AuthSessionsRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Sessions.Revoke",
  __AuthSessionsRevokeDescriptor,
  "AuthSessionsRevoke",
  ACTION_SOURCE,
);

const __AuthUserIdentitiesListDescriptor = {
  subject: "rpc.v1.Auth.UserIdentities.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.UserIdentities.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.UserIdentities.List";
    readonly action: "call";
  },
  input: schema<Types.AuthUserIdentitiesListInput>(
    AuthUserIdentitiesListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUserIdentitiesListInput>>,
  output: schema<Types.AuthUserIdentitiesListOutput>(
    AuthUserIdentitiesListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUserIdentitiesListOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUserIdentitiesList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.UserIdentities.List",
    typeof __AuthUserIdentitiesListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.UserIdentities.List",
  __AuthUserIdentitiesListDescriptor,
  "AuthUserIdentitiesList",
  ACTION_SOURCE,
);

const __AuthUserIdentitiesUnlinkDescriptor = {
  subject: "rpc.v1.Auth.UserIdentities.Unlink",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.UserIdentities.Unlink",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.UserIdentities.Unlink";
    readonly action: "call";
  },
  input: schema<Types.AuthUserIdentitiesUnlinkInput>(
    AuthUserIdentitiesUnlinkRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUserIdentitiesUnlinkInput>>,
  output: schema<Types.AuthUserIdentitiesUnlinkOutput>(
    AuthUserIdentitiesUnlinkResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUserIdentitiesUnlinkOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUserIdentitiesUnlink: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.UserIdentities.Unlink",
    typeof __AuthUserIdentitiesUnlinkDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.UserIdentities.Unlink",
  __AuthUserIdentitiesUnlinkDescriptor,
  "AuthUserIdentitiesUnlink",
  ACTION_SOURCE,
);

const __AuthUsersCreateDescriptor = {
  subject: "rpc.v1.Auth.Users.Create",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.Create",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.Create";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersCreateInput>(
    AuthUsersCreateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersCreateInput>>,
  output: schema<Types.AuthUsersCreateOutput>(
    AuthUsersCreateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersCreateOutput>>,
  callerCapabilities: ["trellis.auth::users.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersCreate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.Create",
    typeof __AuthUsersCreateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.Create",
  __AuthUsersCreateDescriptor,
  "AuthUsersCreate",
  ACTION_SOURCE,
);

const __AuthUsersGetDescriptor = {
  subject: "rpc.v1.Auth.Users.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersGetInput>(
    AuthUsersGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersGetInput>>,
  output: schema<Types.AuthUsersGetOutput>(
    AuthUsersGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersGetOutput>>,
  callerCapabilities: ["trellis.auth::users.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.Get",
    typeof __AuthUsersGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.Get",
  __AuthUsersGetDescriptor,
  "AuthUsersGet",
  ACTION_SOURCE,
);

const __AuthUsersIdentityLinkCreateDescriptor = {
  subject: "rpc.v1.Auth.Users.IdentityLink.Create",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.IdentityLink.Create",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.IdentityLink.Create";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersIdentityLinkCreateInput>(
    AuthUsersIdentityLinkCreateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersIdentityLinkCreateInput>>,
  output: schema<Types.AuthUsersIdentityLinkCreateOutput>(
    AuthUsersIdentityLinkCreateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersIdentityLinkCreateOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersIdentityLinkCreate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.IdentityLink.Create",
    typeof __AuthUsersIdentityLinkCreateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.IdentityLink.Create",
  __AuthUsersIdentityLinkCreateDescriptor,
  "AuthUsersIdentityLinkCreate",
  ACTION_SOURCE,
);

const __AuthUsersListDescriptor = {
  subject: "rpc.v1.Auth.Users.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.List";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersListInput>(
    AuthUsersListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersListInput>>,
  output: schema<Types.AuthUsersListOutput>(
    AuthUsersListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersListOutput>>,
  callerCapabilities: ["trellis.auth::users.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.List",
    typeof __AuthUsersListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.List",
  __AuthUsersListDescriptor,
  "AuthUsersList",
  ACTION_SOURCE,
);

const __AuthUsersPasswordChangeDescriptor = {
  subject: "rpc.v1.Auth.Users.Password.Change",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.Password.Change",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.Password.Change";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersPasswordChangeInput>(
    AuthUsersPasswordChangeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersPasswordChangeInput>>,
  output: schema<Types.AuthUsersPasswordChangeOutput>(
    AuthUsersPasswordChangeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersPasswordChangeOutput>>,
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
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersPasswordChange: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.Password.Change",
    typeof __AuthUsersPasswordChangeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.Password.Change",
  __AuthUsersPasswordChangeDescriptor,
  "AuthUsersPasswordChange",
  ACTION_SOURCE,
);

const __AuthUsersPasswordResetCreateDescriptor = {
  subject: "rpc.v1.Auth.Users.PasswordReset.Create",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.PasswordReset.Create",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.PasswordReset.Create";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersPasswordResetCreateInput>(
    AuthUsersPasswordResetCreateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersPasswordResetCreateInput>>,
  output: schema<Types.AuthUsersPasswordResetCreateOutput>(
    AuthUsersPasswordResetCreateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersPasswordResetCreateOutput>>,
  callerCapabilities: ["trellis.auth::users.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersPasswordResetCreate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.PasswordReset.Create",
    typeof __AuthUsersPasswordResetCreateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.PasswordReset.Create",
  __AuthUsersPasswordResetCreateDescriptor,
  "AuthUsersPasswordResetCreate",
  ACTION_SOURCE,
);

const __AuthUsersResolveDescriptor = {
  subject: "rpc.v1.Auth.Users.Resolve",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.Resolve",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.Resolve";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersResolveInput>(
    AuthUsersResolveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersResolveInput>>,
  output: schema<Types.AuthUsersResolveOutput>(
    AuthUsersResolveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersResolveOutput>>,
  callerCapabilities: ["trellis.auth::users.read"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersResolve: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.Resolve",
    typeof __AuthUsersResolveDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.Resolve",
  __AuthUsersResolveDescriptor,
  "AuthUsersResolve",
  ACTION_SOURCE,
);

const __AuthUsersUpdateDescriptor = {
  subject: "rpc.v1.Auth.Users.Update",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Users.Update",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Users.Update";
    readonly action: "call";
  },
  input: schema<Types.AuthUsersUpdateInput>(
    AuthUsersUpdateRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersUpdateInput>>,
  output: schema<Types.AuthUsersUpdateOutput>(
    AuthUsersUpdateResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthUsersUpdateOutput>>,
  callerCapabilities: ["trellis.auth::users.mutate"] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
export const AuthUsersUpdate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Users.Update",
    typeof __AuthUsersUpdateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Users.Update",
  __AuthUsersUpdateDescriptor,
  "AuthUsersUpdate",
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesResolveDescriptor = {
  subject: "operations.v1.Auth.DeviceUserAuthorities.Resolve",
  permissions: {
    invoke: Object.freeze({
      apiId: "trellis.auth@v1",
      apiVersion: "v1",
      surfaceKind: "operation",
      surfaceName: "Auth.DeviceUserAuthorities.Resolve",
      action: "invoke",
    }) as {
      readonly apiId: "trellis.auth@v1";
      readonly apiVersion: "v1";
      readonly surfaceKind: "operation";
      readonly surfaceName: "Auth.DeviceUserAuthorities.Resolve";
      readonly action: "invoke";
    },
    observe: Object.freeze({
      apiId: "trellis.auth@v1",
      apiVersion: "v1",
      surfaceKind: "operation",
      surfaceName: "Auth.DeviceUserAuthorities.Resolve",
      action: "observe",
    }) as {
      readonly apiId: "trellis.auth@v1";
      readonly apiVersion: "v1";
      readonly surfaceKind: "operation";
      readonly surfaceName: "Auth.DeviceUserAuthorities.Resolve";
      readonly action: "observe";
    },
    cancel: Object.freeze({
      apiId: "trellis.auth@v1",
      apiVersion: "v1",
      surfaceKind: "operation",
      surfaceName: "Auth.DeviceUserAuthorities.Resolve",
      action: "cancel",
    }) as {
      readonly apiId: "trellis.auth@v1";
      readonly apiVersion: "v1";
      readonly surfaceKind: "operation";
      readonly surfaceName: "Auth.DeviceUserAuthorities.Resolve";
      readonly action: "cancel";
    },
    control: {},
  },
  input: schema<Types.AuthDeviceUserAuthoritiesResolveInput>(
    AuthDeviceUserAuthoritiesResolveRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesResolveInput>>,
  progress: schema<Types.AuthDeviceUserAuthoritiesResolveProgress>(
    AuthDeviceUserAuthoritiesResolveProgressSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesResolveProgress>
  >,
  output: schema<Types.AuthDeviceUserAuthoritiesResolveOutput>(
    AuthDeviceUserAuthoritiesResolveResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesResolveOutput>>,
  callerCapabilities: [] as const,
  observeCapabilities: [] as const,
  cancelCapabilities: [] as const,
  controlCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError", "ValidationError"] as const,
  declaredErrorTypes: [
    "AuthError",
    "UnexpectedError",
    "ValidationError",
  ] as const,
  runtimeErrors: [
    {
      type: "AuthError",
      schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema) as ReturnType<
        typeof schema<Types.AuthErrorData>
      >,
      fromSerializable: Types.AuthError
        .fromSerializable as typeof Types.AuthError.fromSerializable,
    },
    {
      type: "UnexpectedError",
      schema: schema<Types.UnexpectedErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.UnexpectedErrorData>>,
      fromSerializable: Types.UnexpectedError
        .fromSerializable as typeof Types.UnexpectedError.fromSerializable,
    },
    {
      type: "ValidationError",
      schema: schema<Types.ValidationErrorData>(
        AuthErrorDetailsSchema,
      ) as ReturnType<typeof schema<Types.ValidationErrorData>>,
      fromSerializable: Types.ValidationError
        .fromSerializable as typeof Types.ValidationError.fromSerializable,
    },
  ] as const,
} as const;
Object.freeze(__AuthDeviceUserAuthoritiesResolveDescriptor.permissions.control);
Object.freeze(__AuthDeviceUserAuthoritiesResolveDescriptor.permissions);
export const AuthDeviceUserAuthoritiesResolve: ReturnType<
  typeof operationAction<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Resolve",
    typeof __AuthDeviceUserAuthoritiesResolveDescriptor
  >
> = operationAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Resolve",
  __AuthDeviceUserAuthoritiesResolveDescriptor,
  "AuthDeviceUserAuthoritiesResolve",
  ACTION_SOURCE,
);

const __AuthConnectionsClosedDescriptor = {
  subject: "events.v1.Auth.Connections.Closed",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Closed",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Closed";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Closed",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Closed";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthConnectionsClosedEvent>(
    AuthConnectionsClosedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsClosedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthConnectionsClosed: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Connections.Closed",
    typeof __AuthConnectionsClosedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Connections.Closed",
  __AuthConnectionsClosedDescriptor,
  "AuthConnectionsClosed",
  true,
  ACTION_SOURCE,
);

const __AuthConnectionsKickedDescriptor = {
  subject: "events.v1.Auth.Connections.Kicked",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Kicked",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Kicked";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Kicked",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Kicked";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthConnectionsKickedEvent>(
    AuthConnectionsKickedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsKickedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthConnectionsKicked: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Connections.Kicked",
    typeof __AuthConnectionsKickedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Connections.Kicked",
  __AuthConnectionsKickedDescriptor,
  "AuthConnectionsKicked",
  true,
  ACTION_SOURCE,
);

const __AuthConnectionsOpenedDescriptor = {
  subject: "events.v1.Auth.Connections.Opened",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Opened",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Opened";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Connections.Opened",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Connections.Opened";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthConnectionsOpenedEvent>(
    AuthConnectionsOpenedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthConnectionsOpenedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthConnectionsOpened: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Connections.Opened",
    typeof __AuthConnectionsOpenedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Connections.Opened",
  __AuthConnectionsOpenedDescriptor,
  "AuthConnectionsOpened",
  true,
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesApprovedDescriptor = {
  subject: "events.v1.Auth.DeviceUserAuthorities.Approved.{/deploymentId}",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Approved",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Approved";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Approved",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Approved";
    readonly action: "subscribe";
  },
  params: ["/deploymentId"] as const,
  event: schema<Types.AuthDeviceUserAuthoritiesApprovedEvent>(
    AuthDeviceUserAuthoritiesApprovedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesApprovedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthDeviceUserAuthoritiesApproved: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Approved",
    typeof __AuthDeviceUserAuthoritiesApprovedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Approved",
  __AuthDeviceUserAuthoritiesApprovedDescriptor,
  "AuthDeviceUserAuthoritiesApproved",
  true,
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesRequestedDescriptor = {
  subject: "events.v1.Auth.DeviceUserAuthorities.Requested.{/deploymentId}",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Requested",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Requested";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Requested",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Requested";
    readonly action: "subscribe";
  },
  params: ["/deploymentId"] as const,
  event: schema<Types.AuthDeviceUserAuthoritiesRequestedEvent>(
    AuthDeviceUserAuthoritiesRequestedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesRequestedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthDeviceUserAuthoritiesRequested: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Requested",
    typeof __AuthDeviceUserAuthoritiesRequestedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Requested",
  __AuthDeviceUserAuthoritiesRequestedDescriptor,
  "AuthDeviceUserAuthoritiesRequested",
  true,
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesResolvedDescriptor = {
  subject: "events.v1.Auth.DeviceUserAuthorities.Resolved.{/deploymentId}",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Resolved",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Resolved";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.Resolved",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.Resolved";
    readonly action: "subscribe";
  },
  params: ["/deploymentId"] as const,
  event: schema<Types.AuthDeviceUserAuthoritiesResolvedEvent>(
    AuthDeviceUserAuthoritiesResolvedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthDeviceUserAuthoritiesResolvedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthDeviceUserAuthoritiesResolved: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.Resolved",
    typeof __AuthDeviceUserAuthoritiesResolvedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Resolved",
  __AuthDeviceUserAuthoritiesResolvedDescriptor,
  "AuthDeviceUserAuthoritiesResolved",
  true,
  ACTION_SOURCE,
);

const __AuthDeviceUserAuthoritiesReviewRequestedDescriptor = {
  subject:
    "events.v1.Auth.DeviceUserAuthorities.ReviewRequested.{/deploymentId}",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.ReviewRequested",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.ReviewRequested";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.DeviceUserAuthorities.ReviewRequested",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.DeviceUserAuthorities.ReviewRequested";
    readonly action: "subscribe";
  },
  params: ["/deploymentId"] as const,
  event: schema<Types.AuthDeviceUserAuthoritiesReviewRequestedEvent>(
    AuthDeviceUserAuthoritiesReviewRequestedEventSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeviceUserAuthoritiesReviewRequestedEvent>
  >,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthDeviceUserAuthoritiesReviewRequested: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.DeviceUserAuthorities.ReviewRequested",
    typeof __AuthDeviceUserAuthoritiesReviewRequestedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.ReviewRequested",
  __AuthDeviceUserAuthoritiesReviewRequestedDescriptor,
  "AuthDeviceUserAuthoritiesReviewRequested",
  true,
  ACTION_SOURCE,
);

const __AuthGrantsChangedDescriptor = {
  subject: "events.v1.Auth.Grants.Changed",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Grants.Changed",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Grants.Changed";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Grants.Changed",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Grants.Changed";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthGrantsChangedEvent>(
    AuthGrantsChangedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthGrantsChangedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: [] as const,
} as const;
export const AuthGrantsChanged: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Grants.Changed",
    typeof __AuthGrantsChangedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Grants.Changed",
  __AuthGrantsChangedDescriptor,
  "AuthGrantsChanged",
  true,
  ACTION_SOURCE,
);

const __AuthIssuersRevokedDescriptor = {
  subject: "events.v1.Auth.Issuers.Revoked",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Issuers.Revoked",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Issuers.Revoked";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Issuers.Revoked",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Issuers.Revoked";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthIssuersRevokedEvent>(
    AuthIssuersRevokedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthIssuersRevokedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: [] as const,
} as const;
export const AuthIssuersRevoked: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Issuers.Revoked",
    typeof __AuthIssuersRevokedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Issuers.Revoked",
  __AuthIssuersRevokedDescriptor,
  "AuthIssuersRevoked",
  true,
  ACTION_SOURCE,
);

const __AuthSessionsRevokedDescriptor = {
  subject: "events.v1.Auth.Sessions.Revoked",
  publishPermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Sessions.Revoked",
    action: "publish",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Sessions.Revoked";
    readonly action: "publish";
  },
  subscribePermission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "event",
    surfaceName: "Auth.Sessions.Revoked",
    action: "subscribe",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "event";
    readonly surfaceName: "Auth.Sessions.Revoked";
    readonly action: "subscribe";
  },
  event: schema<Types.AuthSessionsRevokedEvent>(
    AuthSessionsRevokedEventSchema,
  ) as ReturnType<typeof schema<Types.AuthSessionsRevokedEvent>>,
  publishCapabilities: [] as const,
  subscribeCapabilities: ["trellis.auth::events.stream"] as const,
} as const;
export const AuthSessionsRevoked: ReturnType<
  typeof eventActions<
    typeof API_ID,
    "Auth.Sessions.Revoked",
    typeof __AuthSessionsRevokedDescriptor,
    true
  >
> = eventActions(
  API_ID,
  "Auth.Sessions.Revoked",
  __AuthSessionsRevokedDescriptor,
  "AuthSessionsRevoked",
  true,
  ACTION_SOURCE,
);

export const ACTIONS = {
  "Auth.Capabilities.List": AuthCapabilitiesList as typeof AuthCapabilitiesList,
  "Auth.CapabilityGroups.Delete":
    AuthCapabilityGroupsDelete as typeof AuthCapabilityGroupsDelete,
  "Auth.CapabilityGroups.Get":
    AuthCapabilityGroupsGet as typeof AuthCapabilityGroupsGet,
  "Auth.CapabilityGroups.List":
    AuthCapabilityGroupsList as typeof AuthCapabilityGroupsList,
  "Auth.CapabilityGroups.Put":
    AuthCapabilityGroupsPut as typeof AuthCapabilityGroupsPut,
  "Auth.Connections.Kick": AuthConnectionsKick as typeof AuthConnectionsKick,
  "Auth.Connections.List": AuthConnectionsList as typeof AuthConnectionsList,
  "Auth.Deployments.Apply": AuthDeploymentsApply as typeof AuthDeploymentsApply,
  "Auth.Deployments.Create":
    AuthDeploymentsCreate as typeof AuthDeploymentsCreate,
  "Auth.Deployments.Disable":
    AuthDeploymentsDisable as typeof AuthDeploymentsDisable,
  "Auth.Deployments.Enable":
    AuthDeploymentsEnable as typeof AuthDeploymentsEnable,
  "Auth.Deployments.Get": AuthDeploymentsGet as typeof AuthDeploymentsGet,
  "Auth.Deployments.List": AuthDeploymentsList as typeof AuthDeploymentsList,
  "Auth.Deployments.Remove":
    AuthDeploymentsRemove as typeof AuthDeploymentsRemove,
  "Auth.DeviceUserAuthorities.List":
    AuthDeviceUserAuthoritiesList as typeof AuthDeviceUserAuthoritiesList,
  "Auth.DeviceUserAuthorities.Reviews.Decide":
    AuthDeviceUserAuthoritiesReviewsDecide as typeof AuthDeviceUserAuthoritiesReviewsDecide,
  "Auth.DeviceUserAuthorities.Reviews.List":
    AuthDeviceUserAuthoritiesReviewsList as typeof AuthDeviceUserAuthoritiesReviewsList,
  "Auth.DeviceUserAuthorities.Revoke":
    AuthDeviceUserAuthoritiesRevoke as typeof AuthDeviceUserAuthoritiesRevoke,
  "Auth.Devices.Disable": AuthDevicesDisable as typeof AuthDevicesDisable,
  "Auth.Devices.Enable": AuthDevicesEnable as typeof AuthDevicesEnable,
  "Auth.Devices.List": AuthDevicesList as typeof AuthDevicesList,
  "Auth.Devices.Provision": AuthDevicesProvision as typeof AuthDevicesProvision,
  "Auth.Devices.Remove": AuthDevicesRemove as typeof AuthDevicesRemove,
  "Auth.Grants.Get": AuthGrantsGet as typeof AuthGrantsGet,
  "Auth.Grants.List": AuthGrantsList as typeof AuthGrantsList,
  "Auth.Grants.Revoke": AuthGrantsRevoke as typeof AuthGrantsRevoke,
  "Auth.Grants.Set": AuthGrantsSet as typeof AuthGrantsSet,
  "Auth.Issuers.Revoke": AuthIssuersRevoke as typeof AuthIssuersRevoke,
  "Auth.Participants.Get": AuthParticipantsGet as typeof AuthParticipantsGet,
  "Auth.Participants.Install":
    AuthParticipantsInstall as typeof AuthParticipantsInstall,
  "Auth.Portals.Get": AuthPortalsGet as typeof AuthPortalsGet,
  "Auth.Portals.GrantOverrides.List":
    AuthPortalsGrantOverridesList as typeof AuthPortalsGrantOverridesList,
  "Auth.Portals.GrantOverrides.Put":
    AuthPortalsGrantOverridesPut as typeof AuthPortalsGrantOverridesPut,
  "Auth.Portals.GrantOverrides.Remove":
    AuthPortalsGrantOverridesRemove as typeof AuthPortalsGrantOverridesRemove,
  "Auth.Portals.List": AuthPortalsList as typeof AuthPortalsList,
  "Auth.Portals.LoginSettings.Get":
    AuthPortalsLoginSettingsGet as typeof AuthPortalsLoginSettingsGet,
  "Auth.Portals.LoginSettings.Update":
    AuthPortalsLoginSettingsUpdate as typeof AuthPortalsLoginSettingsUpdate,
  "Auth.Portals.Put": AuthPortalsPut as typeof AuthPortalsPut,
  "Auth.Portals.Remove": AuthPortalsRemove as typeof AuthPortalsRemove,
  "Auth.Portals.Routes.Put":
    AuthPortalsRoutesPut as typeof AuthPortalsRoutesPut,
  "Auth.Portals.Routes.Remove":
    AuthPortalsRoutesRemove as typeof AuthPortalsRoutesRemove,
  "Auth.ServiceInstances.Disable":
    AuthServiceInstancesDisable as typeof AuthServiceInstancesDisable,
  "Auth.ServiceInstances.Enable":
    AuthServiceInstancesEnable as typeof AuthServiceInstancesEnable,
  "Auth.ServiceInstances.List":
    AuthServiceInstancesList as typeof AuthServiceInstancesList,
  "Auth.ServiceInstances.Provision":
    AuthServiceInstancesProvision as typeof AuthServiceInstancesProvision,
  "Auth.ServiceInstances.Remove":
    AuthServiceInstancesRemove as typeof AuthServiceInstancesRemove,
  "Auth.Sessions.List": AuthSessionsList as typeof AuthSessionsList,
  "Auth.Sessions.Logout": AuthSessionsLogout as typeof AuthSessionsLogout,
  "Auth.Sessions.Me": AuthSessionsMe as typeof AuthSessionsMe,
  "Auth.Sessions.Revoke": AuthSessionsRevoke as typeof AuthSessionsRevoke,
  "Auth.UserIdentities.List":
    AuthUserIdentitiesList as typeof AuthUserIdentitiesList,
  "Auth.UserIdentities.Unlink":
    AuthUserIdentitiesUnlink as typeof AuthUserIdentitiesUnlink,
  "Auth.Users.Create": AuthUsersCreate as typeof AuthUsersCreate,
  "Auth.Users.Get": AuthUsersGet as typeof AuthUsersGet,
  "Auth.Users.IdentityLink.Create":
    AuthUsersIdentityLinkCreate as typeof AuthUsersIdentityLinkCreate,
  "Auth.Users.List": AuthUsersList as typeof AuthUsersList,
  "Auth.Users.Password.Change":
    AuthUsersPasswordChange as typeof AuthUsersPasswordChange,
  "Auth.Users.PasswordReset.Create":
    AuthUsersPasswordResetCreate as typeof AuthUsersPasswordResetCreate,
  "Auth.Users.Resolve": AuthUsersResolve as typeof AuthUsersResolve,
  "Auth.Users.Update": AuthUsersUpdate as typeof AuthUsersUpdate,
  "Auth.DeviceUserAuthorities.Resolve":
    AuthDeviceUserAuthoritiesResolve as typeof AuthDeviceUserAuthoritiesResolve,
  "Auth.Connections.Closed":
    AuthConnectionsClosed as typeof AuthConnectionsClosed,
  "Auth.Connections.Kicked":
    AuthConnectionsKicked as typeof AuthConnectionsKicked,
  "Auth.Connections.Opened":
    AuthConnectionsOpened as typeof AuthConnectionsOpened,
  "Auth.DeviceUserAuthorities.Approved":
    AuthDeviceUserAuthoritiesApproved as typeof AuthDeviceUserAuthoritiesApproved,
  "Auth.DeviceUserAuthorities.Requested":
    AuthDeviceUserAuthoritiesRequested as typeof AuthDeviceUserAuthoritiesRequested,
  "Auth.DeviceUserAuthorities.Resolved":
    AuthDeviceUserAuthoritiesResolved as typeof AuthDeviceUserAuthoritiesResolved,
  "Auth.DeviceUserAuthorities.ReviewRequested":
    AuthDeviceUserAuthoritiesReviewRequested as typeof AuthDeviceUserAuthoritiesReviewRequested,
  "Auth.Grants.Changed": AuthGrantsChanged as typeof AuthGrantsChanged,
  "Auth.Issuers.Revoked": AuthIssuersRevoked as typeof AuthIssuersRevoked,
  "Auth.Sessions.Revoked": AuthSessionsRevoked as typeof AuthSessionsRevoked,
} as const;
