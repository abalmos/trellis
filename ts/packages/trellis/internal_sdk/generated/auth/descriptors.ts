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
  AuthDeploymentAuthorityAcceptMigrationRequestSchema,
  AuthDeploymentAuthorityAcceptMigrationResponseSchema,
  AuthDeploymentAuthorityAcceptUpdateRequestSchema,
  AuthDeploymentAuthorityAcceptUpdateResponseSchema,
  AuthDeploymentAuthorityGetRequestSchema,
  AuthDeploymentAuthorityGetResponseSchema,
  AuthDeploymentAuthorityListRequestSchema,
  AuthDeploymentAuthorityListResponseSchema,
  AuthDeploymentAuthorityPlanRequestSchema,
  AuthDeploymentAuthorityPlanResponseSchema,
  AuthDeploymentAuthorityPlansGetRequestSchema,
  AuthDeploymentAuthorityPlansGetResponseSchema,
  AuthDeploymentAuthorityPlansListRequestSchema,
  AuthDeploymentAuthorityPlansListResponseSchema,
  AuthDeploymentAuthorityReconcileRequestSchema,
  AuthDeploymentAuthorityReconcileResponseSchema,
  AuthDeploymentAuthorityRejectRequestSchema,
  AuthDeploymentAuthorityRejectResponseSchema,
  AuthDeploymentsCreateRequestSchema,
  AuthDeploymentsCreateResponseSchema,
  AuthDeploymentsDisableRequestSchema,
  AuthDeploymentsDisableResponseSchema,
  AuthDeploymentsEnableRequestSchema,
  AuthDeploymentsEnableResponseSchema,
  AuthDeploymentsListRequestSchema,
  AuthDeploymentsListResponseSchema,
  AuthDeploymentsRemoveRequestSchema,
  AuthDeploymentsRemoveResponseSchema,
  AuthDevicesConnectInfoGetRequestSchema,
  AuthDevicesConnectInfoGetResponseSchema,
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
  AuthIdentityAuthorityGetRequestSchema,
  AuthIdentityAuthorityGetResponseSchema,
  AuthIdentityAuthorityListRequestSchema,
  AuthIdentityAuthorityListResponseSchema,
  AuthIdentityAuthorityRevokeRequestSchema,
  AuthIdentityAuthorityRevokeResponseSchema,
  AuthIdentityGrantsListRequestSchema,
  AuthIdentityGrantsListResponseSchema,
  AuthIdentityGrantsRevokeRequestSchema,
  AuthIdentityGrantsRevokeResponseSchema,
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
  callerCapabilities: ["trellis.auth::capabilities.delegate"] as const,
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
  callerCapabilities: ["trellis.auth::capabilities.delegate"] as const,
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

const __AuthDeploymentAuthorityAcceptMigrationDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.AcceptMigration",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.AcceptMigration",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.AcceptMigration";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityAcceptMigrationInput>(
    AuthDeploymentAuthorityAcceptMigrationRequestSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeploymentAuthorityAcceptMigrationInput>
  >,
  output: schema<Types.AuthDeploymentAuthorityAcceptMigrationOutput>(
    AuthDeploymentAuthorityAcceptMigrationResponseSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeploymentAuthorityAcceptMigrationOutput>
  >,
  callerCapabilities: [
    "trellis.auth::authorities.mutate",
    "trellis.auth::capabilities.delegate",
  ] as const,
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
export const AuthDeploymentAuthorityAcceptMigration: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.AcceptMigration",
    typeof __AuthDeploymentAuthorityAcceptMigrationDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.AcceptMigration",
  __AuthDeploymentAuthorityAcceptMigrationDescriptor,
  "AuthDeploymentAuthorityAcceptMigration",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityAcceptUpdateDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.AcceptUpdate",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.AcceptUpdate",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.AcceptUpdate";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityAcceptUpdateInput>(
    AuthDeploymentAuthorityAcceptUpdateRequestSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeploymentAuthorityAcceptUpdateInput>
  >,
  output: schema<Types.AuthDeploymentAuthorityAcceptUpdateOutput>(
    AuthDeploymentAuthorityAcceptUpdateResponseSchema,
  ) as ReturnType<
    typeof schema<Types.AuthDeploymentAuthorityAcceptUpdateOutput>
  >,
  callerCapabilities: [
    "trellis.auth::authorities.mutate",
    "trellis.auth::capabilities.delegate",
  ] as const,
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
export const AuthDeploymentAuthorityAcceptUpdate: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.AcceptUpdate",
    typeof __AuthDeploymentAuthorityAcceptUpdateDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.AcceptUpdate",
  __AuthDeploymentAuthorityAcceptUpdateDescriptor,
  "AuthDeploymentAuthorityAcceptUpdate",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityGetDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityGetInput>(
    AuthDeploymentAuthorityGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityGetInput>>,
  output: schema<Types.AuthDeploymentAuthorityGetOutput>(
    AuthDeploymentAuthorityGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityGetOutput>>,
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
export const AuthDeploymentAuthorityGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Get",
    typeof __AuthDeploymentAuthorityGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Get",
  __AuthDeploymentAuthorityGetDescriptor,
  "AuthDeploymentAuthorityGet",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityListDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityListInput>(
    AuthDeploymentAuthorityListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityListInput>>,
  output: schema<Types.AuthDeploymentAuthorityListOutput>(
    AuthDeploymentAuthorityListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityListOutput>>,
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
export const AuthDeploymentAuthorityList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.List",
    typeof __AuthDeploymentAuthorityListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.List",
  __AuthDeploymentAuthorityListDescriptor,
  "AuthDeploymentAuthorityList",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityPlanDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Plan",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Plan",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Plan";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityPlanInput>(
    AuthDeploymentAuthorityPlanRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlanInput>>,
  output: schema<Types.AuthDeploymentAuthorityPlanOutput>(
    AuthDeploymentAuthorityPlanResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlanOutput>>,
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
export const AuthDeploymentAuthorityPlan: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Plan",
    typeof __AuthDeploymentAuthorityPlanDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plan",
  __AuthDeploymentAuthorityPlanDescriptor,
  "AuthDeploymentAuthorityPlan",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityPlansGetDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Plans.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Plans.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Plans.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityPlansGetInput>(
    AuthDeploymentAuthorityPlansGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlansGetInput>>,
  output: schema<Types.AuthDeploymentAuthorityPlansGetOutput>(
    AuthDeploymentAuthorityPlansGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlansGetOutput>>,
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
export const AuthDeploymentAuthorityPlansGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Plans.Get",
    typeof __AuthDeploymentAuthorityPlansGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plans.Get",
  __AuthDeploymentAuthorityPlansGetDescriptor,
  "AuthDeploymentAuthorityPlansGet",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityPlansListDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Plans.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Plans.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Plans.List";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityPlansListInput>(
    AuthDeploymentAuthorityPlansListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlansListInput>>,
  output: schema<Types.AuthDeploymentAuthorityPlansListOutput>(
    AuthDeploymentAuthorityPlansListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityPlansListOutput>>,
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
export const AuthDeploymentAuthorityPlansList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Plans.List",
    typeof __AuthDeploymentAuthorityPlansListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plans.List",
  __AuthDeploymentAuthorityPlansListDescriptor,
  "AuthDeploymentAuthorityPlansList",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityReconcileDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Reconcile",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Reconcile",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Reconcile";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityReconcileInput>(
    AuthDeploymentAuthorityReconcileRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityReconcileInput>>,
  output: schema<Types.AuthDeploymentAuthorityReconcileOutput>(
    AuthDeploymentAuthorityReconcileResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityReconcileOutput>>,
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
export const AuthDeploymentAuthorityReconcile: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Reconcile",
    typeof __AuthDeploymentAuthorityReconcileDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Reconcile",
  __AuthDeploymentAuthorityReconcileDescriptor,
  "AuthDeploymentAuthorityReconcile",
  ACTION_SOURCE,
);

const __AuthDeploymentAuthorityRejectDescriptor = {
  subject: "rpc.v1.Auth.DeploymentAuthority.Reject",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.DeploymentAuthority.Reject",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.DeploymentAuthority.Reject";
    readonly action: "call";
  },
  input: schema<Types.AuthDeploymentAuthorityRejectInput>(
    AuthDeploymentAuthorityRejectRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityRejectInput>>,
  output: schema<Types.AuthDeploymentAuthorityRejectOutput>(
    AuthDeploymentAuthorityRejectResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDeploymentAuthorityRejectOutput>>,
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
export const AuthDeploymentAuthorityReject: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.DeploymentAuthority.Reject",
    typeof __AuthDeploymentAuthorityRejectDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Reject",
  __AuthDeploymentAuthorityRejectDescriptor,
  "AuthDeploymentAuthorityReject",
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

const __AuthDevicesConnectInfoGetDescriptor = {
  subject: "rpc.v1.Auth.Devices.ConnectInfo.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.Devices.ConnectInfo.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.Devices.ConnectInfo.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthDevicesConnectInfoGetInput>(
    AuthDevicesConnectInfoGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesConnectInfoGetInput>>,
  output: schema<Types.AuthDevicesConnectInfoGetOutput>(
    AuthDevicesConnectInfoGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthDevicesConnectInfoGetOutput>>,
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
export const AuthDevicesConnectInfoGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.Devices.ConnectInfo.Get",
    typeof __AuthDevicesConnectInfoGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.Devices.ConnectInfo.Get",
  __AuthDevicesConnectInfoGetDescriptor,
  "AuthDevicesConnectInfoGet",
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

const __AuthIdentityAuthorityGetDescriptor = {
  subject: "rpc.v1.Auth.IdentityAuthority.Get",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.IdentityAuthority.Get",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.IdentityAuthority.Get";
    readonly action: "call";
  },
  input: schema<Types.AuthIdentityAuthorityGetInput>(
    AuthIdentityAuthorityGetRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityGetInput>>,
  output: schema<Types.AuthIdentityAuthorityGetOutput>(
    AuthIdentityAuthorityGetResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityGetOutput>>,
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
export const AuthIdentityAuthorityGet: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.IdentityAuthority.Get",
    typeof __AuthIdentityAuthorityGetDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.Get",
  __AuthIdentityAuthorityGetDescriptor,
  "AuthIdentityAuthorityGet",
  ACTION_SOURCE,
);

const __AuthIdentityAuthorityListDescriptor = {
  subject: "rpc.v1.Auth.IdentityAuthority.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.IdentityAuthority.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.IdentityAuthority.List";
    readonly action: "call";
  },
  input: schema<Types.AuthIdentityAuthorityListInput>(
    AuthIdentityAuthorityListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityListInput>>,
  output: schema<Types.AuthIdentityAuthorityListOutput>(
    AuthIdentityAuthorityListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityListOutput>>,
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
export const AuthIdentityAuthorityList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.IdentityAuthority.List",
    typeof __AuthIdentityAuthorityListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.List",
  __AuthIdentityAuthorityListDescriptor,
  "AuthIdentityAuthorityList",
  ACTION_SOURCE,
);

const __AuthIdentityAuthorityRevokeDescriptor = {
  subject: "rpc.v1.Auth.IdentityAuthority.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.IdentityAuthority.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.IdentityAuthority.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthIdentityAuthorityRevokeInput>(
    AuthIdentityAuthorityRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityRevokeInput>>,
  output: schema<Types.AuthIdentityAuthorityRevokeOutput>(
    AuthIdentityAuthorityRevokeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityAuthorityRevokeOutput>>,
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
export const AuthIdentityAuthorityRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.IdentityAuthority.Revoke",
    typeof __AuthIdentityAuthorityRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.Revoke",
  __AuthIdentityAuthorityRevokeDescriptor,
  "AuthIdentityAuthorityRevoke",
  ACTION_SOURCE,
);

const __AuthIdentityGrantsListDescriptor = {
  subject: "rpc.v1.Auth.IdentityGrants.List",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.IdentityGrants.List",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.IdentityGrants.List";
    readonly action: "call";
  },
  input: schema<Types.AuthIdentityGrantsListInput>(
    AuthIdentityGrantsListRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityGrantsListInput>>,
  output: schema<Types.AuthIdentityGrantsListOutput>(
    AuthIdentityGrantsListResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityGrantsListOutput>>,
  callerCapabilities: [] as const,
  errors: ["AuthError", "UnexpectedError"] as const,
  declaredErrorTypes: ["AuthError", "UnexpectedError"] as const,
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
  ] as const,
} as const;
export const AuthIdentityGrantsList: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.IdentityGrants.List",
    typeof __AuthIdentityGrantsListDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.IdentityGrants.List",
  __AuthIdentityGrantsListDescriptor,
  "AuthIdentityGrantsList",
  ACTION_SOURCE,
);

const __AuthIdentityGrantsRevokeDescriptor = {
  subject: "rpc.v1.Auth.IdentityGrants.Revoke",
  permission: Object.freeze({
    apiId: "trellis.auth@v1",
    apiVersion: "v1",
    surfaceKind: "rpc",
    surfaceName: "Auth.IdentityGrants.Revoke",
    action: "call",
  }) as {
    readonly apiId: "trellis.auth@v1";
    readonly apiVersion: "v1";
    readonly surfaceKind: "rpc";
    readonly surfaceName: "Auth.IdentityGrants.Revoke";
    readonly action: "call";
  },
  input: schema<Types.AuthIdentityGrantsRevokeInput>(
    AuthIdentityGrantsRevokeRequestSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityGrantsRevokeInput>>,
  output: schema<Types.AuthIdentityGrantsRevokeOutput>(
    AuthIdentityGrantsRevokeResponseSchema,
  ) as ReturnType<typeof schema<Types.AuthIdentityGrantsRevokeOutput>>,
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
export const AuthIdentityGrantsRevoke: ReturnType<
  typeof rpcAction<
    typeof API_ID,
    "Auth.IdentityGrants.Revoke",
    typeof __AuthIdentityGrantsRevokeDescriptor
  >
> = rpcAction(
  API_ID,
  "Auth.IdentityGrants.Revoke",
  __AuthIdentityGrantsRevokeDescriptor,
  "AuthIdentityGrantsRevoke",
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
  callerCapabilities: [
    "trellis.auth::capabilities.delegate",
    "trellis.auth::portals.mutate",
  ] as const,
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
  callerCapabilities: [
    "trellis.auth::capabilities.delegate",
    "trellis.auth::portals.mutate",
  ] as const,
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
  "Auth.DeploymentAuthority.AcceptMigration":
    AuthDeploymentAuthorityAcceptMigration as typeof AuthDeploymentAuthorityAcceptMigration,
  "Auth.DeploymentAuthority.AcceptUpdate":
    AuthDeploymentAuthorityAcceptUpdate as typeof AuthDeploymentAuthorityAcceptUpdate,
  "Auth.DeploymentAuthority.Get":
    AuthDeploymentAuthorityGet as typeof AuthDeploymentAuthorityGet,
  "Auth.DeploymentAuthority.List":
    AuthDeploymentAuthorityList as typeof AuthDeploymentAuthorityList,
  "Auth.DeploymentAuthority.Plan":
    AuthDeploymentAuthorityPlan as typeof AuthDeploymentAuthorityPlan,
  "Auth.DeploymentAuthority.Plans.Get":
    AuthDeploymentAuthorityPlansGet as typeof AuthDeploymentAuthorityPlansGet,
  "Auth.DeploymentAuthority.Plans.List":
    AuthDeploymentAuthorityPlansList as typeof AuthDeploymentAuthorityPlansList,
  "Auth.DeploymentAuthority.Reconcile":
    AuthDeploymentAuthorityReconcile as typeof AuthDeploymentAuthorityReconcile,
  "Auth.DeploymentAuthority.Reject":
    AuthDeploymentAuthorityReject as typeof AuthDeploymentAuthorityReject,
  "Auth.Deployments.Create":
    AuthDeploymentsCreate as typeof AuthDeploymentsCreate,
  "Auth.Deployments.Disable":
    AuthDeploymentsDisable as typeof AuthDeploymentsDisable,
  "Auth.Deployments.Enable":
    AuthDeploymentsEnable as typeof AuthDeploymentsEnable,
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
  "Auth.Devices.ConnectInfo.Get":
    AuthDevicesConnectInfoGet as typeof AuthDevicesConnectInfoGet,
  "Auth.Devices.Disable": AuthDevicesDisable as typeof AuthDevicesDisable,
  "Auth.Devices.Enable": AuthDevicesEnable as typeof AuthDevicesEnable,
  "Auth.Devices.List": AuthDevicesList as typeof AuthDevicesList,
  "Auth.Devices.Provision": AuthDevicesProvision as typeof AuthDevicesProvision,
  "Auth.Devices.Remove": AuthDevicesRemove as typeof AuthDevicesRemove,
  "Auth.IdentityAuthority.Get":
    AuthIdentityAuthorityGet as typeof AuthIdentityAuthorityGet,
  "Auth.IdentityAuthority.List":
    AuthIdentityAuthorityList as typeof AuthIdentityAuthorityList,
  "Auth.IdentityAuthority.Revoke":
    AuthIdentityAuthorityRevoke as typeof AuthIdentityAuthorityRevoke,
  "Auth.IdentityGrants.List":
    AuthIdentityGrantsList as typeof AuthIdentityGrantsList,
  "Auth.IdentityGrants.Revoke":
    AuthIdentityGrantsRevoke as typeof AuthIdentityGrantsRevoke,
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
  "Auth.Sessions.Revoked": AuthSessionsRevoked as typeof AuthSessionsRevoked,
} as const;
