// Generated from ./generated/apis/trellis.auth@v1.json
import {
  eventActions,
  feedAction,
  operationAction,
  rpcAction,
  schema,
} from "../../../contracts.ts";
import * as Types from "./types.ts";
import {
  AuthCapabilitiesListRequestSchema,
  AuthCapabilitiesListResponseSchema,
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
  AuthPortalsGetRequestSchema,
  AuthPortalsGetResponseSchema,
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
import {
  API as ACTION_ARTIFACT,
  API_DIGEST as ACTION_DIGEST,
} from "./manifest.ts";

const ACTION_SOURCE = {
  artifact: ACTION_ARTIFACT,
  digest: ACTION_DIGEST,
} as const;

const API_ID = "trellis.auth@v1" as const;

export const AuthCapabilitiesList = rpcAction(
  API_ID,
  "Auth.Capabilities.List",
  {
    subject: "rpc.v1.Auth.Capabilities.List",
    input: schema<Types.AuthCapabilitiesListInput>(
      AuthCapabilitiesListRequestSchema,
    ),
    output: schema<Types.AuthCapabilitiesListOutput>(
      AuthCapabilitiesListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthCapabilitiesList",
  ACTION_SOURCE,
);

export const AuthConnectionsKick = rpcAction(
  API_ID,
  "Auth.Connections.Kick",
  {
    subject: "rpc.v1.Auth.Connections.Kick",
    input: schema<Types.AuthConnectionsKickInput>(
      AuthConnectionsKickRequestSchema,
    ),
    output: schema<Types.AuthConnectionsKickOutput>(
      AuthConnectionsKickResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthConnectionsKick",
  ACTION_SOURCE,
);

export const AuthConnectionsList = rpcAction(
  API_ID,
  "Auth.Connections.List",
  {
    subject: "rpc.v1.Auth.Connections.List",
    input: schema<Types.AuthConnectionsListInput>(
      AuthConnectionsListRequestSchema,
    ),
    output: schema<Types.AuthConnectionsListOutput>(
      AuthConnectionsListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthConnectionsList",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityAcceptMigration = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.AcceptMigration",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.AcceptMigration",
    input: schema<Types.AuthDeploymentAuthorityAcceptMigrationInput>(
      AuthDeploymentAuthorityAcceptMigrationRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityAcceptMigrationOutput>(
      AuthDeploymentAuthorityAcceptMigrationResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityAcceptMigration",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityAcceptUpdate = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.AcceptUpdate",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.AcceptUpdate",
    input: schema<Types.AuthDeploymentAuthorityAcceptUpdateInput>(
      AuthDeploymentAuthorityAcceptUpdateRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityAcceptUpdateOutput>(
      AuthDeploymentAuthorityAcceptUpdateResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityAcceptUpdate",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityGet = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Get",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Get",
    input: schema<Types.AuthDeploymentAuthorityGetInput>(
      AuthDeploymentAuthorityGetRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityGetOutput>(
      AuthDeploymentAuthorityGetResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityGet",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityList = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.List",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.List",
    input: schema<Types.AuthDeploymentAuthorityListInput>(
      AuthDeploymentAuthorityListRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityListOutput>(
      AuthDeploymentAuthorityListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityList",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityPlan = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plan",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Plan",
    input: schema<Types.AuthDeploymentAuthorityPlanInput>(
      AuthDeploymentAuthorityPlanRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityPlanOutput>(
      AuthDeploymentAuthorityPlanResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityPlan",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityPlansGet = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plans.Get",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Plans.Get",
    input: schema<Types.AuthDeploymentAuthorityPlansGetInput>(
      AuthDeploymentAuthorityPlansGetRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityPlansGetOutput>(
      AuthDeploymentAuthorityPlansGetResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityPlansGet",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityPlansList = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Plans.List",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Plans.List",
    input: schema<Types.AuthDeploymentAuthorityPlansListInput>(
      AuthDeploymentAuthorityPlansListRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityPlansListOutput>(
      AuthDeploymentAuthorityPlansListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityPlansList",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityReconcile = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Reconcile",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Reconcile",
    input: schema<Types.AuthDeploymentAuthorityReconcileInput>(
      AuthDeploymentAuthorityReconcileRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityReconcileOutput>(
      AuthDeploymentAuthorityReconcileResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityReconcile",
  ACTION_SOURCE,
);

export const AuthDeploymentAuthorityReject = rpcAction(
  API_ID,
  "Auth.DeploymentAuthority.Reject",
  {
    subject: "rpc.v1.Auth.DeploymentAuthority.Reject",
    input: schema<Types.AuthDeploymentAuthorityRejectInput>(
      AuthDeploymentAuthorityRejectRequestSchema,
    ),
    output: schema<Types.AuthDeploymentAuthorityRejectOutput>(
      AuthDeploymentAuthorityRejectResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentAuthorityReject",
  ACTION_SOURCE,
);

export const AuthDeploymentsCreate = rpcAction(
  API_ID,
  "Auth.Deployments.Create",
  {
    subject: "rpc.v1.Auth.Deployments.Create",
    input: schema<Types.AuthDeploymentsCreateInput>(
      AuthDeploymentsCreateRequestSchema,
    ),
    output: schema<Types.AuthDeploymentsCreateOutput>(
      AuthDeploymentsCreateResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentsCreate",
  ACTION_SOURCE,
);

export const AuthDeploymentsDisable = rpcAction(
  API_ID,
  "Auth.Deployments.Disable",
  {
    subject: "rpc.v1.Auth.Deployments.Disable",
    input: schema<Types.AuthDeploymentsDisableInput>(
      AuthDeploymentsDisableRequestSchema,
    ),
    output: schema<Types.AuthDeploymentsDisableOutput>(
      AuthDeploymentsDisableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentsDisable",
  ACTION_SOURCE,
);

export const AuthDeploymentsEnable = rpcAction(
  API_ID,
  "Auth.Deployments.Enable",
  {
    subject: "rpc.v1.Auth.Deployments.Enable",
    input: schema<Types.AuthDeploymentsEnableInput>(
      AuthDeploymentsEnableRequestSchema,
    ),
    output: schema<Types.AuthDeploymentsEnableOutput>(
      AuthDeploymentsEnableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentsEnable",
  ACTION_SOURCE,
);

export const AuthDeploymentsList = rpcAction(
  API_ID,
  "Auth.Deployments.List",
  {
    subject: "rpc.v1.Auth.Deployments.List",
    input: schema<Types.AuthDeploymentsListInput>(
      AuthDeploymentsListRequestSchema,
    ),
    output: schema<Types.AuthDeploymentsListOutput>(
      AuthDeploymentsListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentsList",
  ACTION_SOURCE,
);

export const AuthDeploymentsRemove = rpcAction(
  API_ID,
  "Auth.Deployments.Remove",
  {
    subject: "rpc.v1.Auth.Deployments.Remove",
    input: schema<Types.AuthDeploymentsRemoveInput>(
      AuthDeploymentsRemoveRequestSchema,
    ),
    output: schema<Types.AuthDeploymentsRemoveOutput>(
      AuthDeploymentsRemoveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeploymentsRemove",
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesList = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.List",
  {
    subject: "rpc.v1.Auth.DeviceUserAuthorities.List",
    input: schema<Types.AuthDeviceUserAuthoritiesListInput>(
      AuthDeviceUserAuthoritiesListRequestSchema,
    ),
    output: schema<Types.AuthDeviceUserAuthoritiesListOutput>(
      AuthDeviceUserAuthoritiesListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeviceUserAuthoritiesList",
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesReviewsDecide = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Reviews.Decide",
  {
    subject: "rpc.v1.Auth.DeviceUserAuthorities.Reviews.Decide",
    input: schema<Types.AuthDeviceUserAuthoritiesReviewsDecideInput>(
      AuthDeviceUserAuthoritiesReviewsDecideRequestSchema,
    ),
    output: schema<Types.AuthDeviceUserAuthoritiesReviewsDecideOutput>(
      AuthDeviceUserAuthoritiesReviewsDecideResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeviceUserAuthoritiesReviewsDecide",
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesReviewsList = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Reviews.List",
  {
    subject: "rpc.v1.Auth.DeviceUserAuthorities.Reviews.List",
    input: schema<Types.AuthDeviceUserAuthoritiesReviewsListInput>(
      AuthDeviceUserAuthoritiesReviewsListRequestSchema,
    ),
    output: schema<Types.AuthDeviceUserAuthoritiesReviewsListOutput>(
      AuthDeviceUserAuthoritiesReviewsListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeviceUserAuthoritiesReviewsList",
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesRevoke = rpcAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Revoke",
  {
    subject: "rpc.v1.Auth.DeviceUserAuthorities.Revoke",
    input: schema<Types.AuthDeviceUserAuthoritiesRevokeInput>(
      AuthDeviceUserAuthoritiesRevokeRequestSchema,
    ),
    output: schema<Types.AuthDeviceUserAuthoritiesRevokeOutput>(
      AuthDeviceUserAuthoritiesRevokeResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDeviceUserAuthoritiesRevoke",
  ACTION_SOURCE,
);

export const AuthDevicesConnectInfoGet = rpcAction(
  API_ID,
  "Auth.Devices.ConnectInfo.Get",
  {
    subject: "rpc.v1.Auth.Devices.ConnectInfo.Get",
    input: schema<Types.AuthDevicesConnectInfoGetInput>(
      AuthDevicesConnectInfoGetRequestSchema,
    ),
    output: schema<Types.AuthDevicesConnectInfoGetOutput>(
      AuthDevicesConnectInfoGetResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesConnectInfoGet",
  ACTION_SOURCE,
);

export const AuthDevicesDisable = rpcAction(
  API_ID,
  "Auth.Devices.Disable",
  {
    subject: "rpc.v1.Auth.Devices.Disable",
    input: schema<Types.AuthDevicesDisableInput>(
      AuthDevicesDisableRequestSchema,
    ),
    output: schema<Types.AuthDevicesDisableOutput>(
      AuthDevicesDisableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesDisable",
  ACTION_SOURCE,
);

export const AuthDevicesEnable = rpcAction(
  API_ID,
  "Auth.Devices.Enable",
  {
    subject: "rpc.v1.Auth.Devices.Enable",
    input: schema<Types.AuthDevicesEnableInput>(AuthDevicesEnableRequestSchema),
    output: schema<Types.AuthDevicesEnableOutput>(
      AuthDevicesEnableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesEnable",
  ACTION_SOURCE,
);

export const AuthDevicesList = rpcAction(
  API_ID,
  "Auth.Devices.List",
  {
    subject: "rpc.v1.Auth.Devices.List",
    input: schema<Types.AuthDevicesListInput>(AuthDevicesListRequestSchema),
    output: schema<Types.AuthDevicesListOutput>(AuthDevicesListResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesList",
  ACTION_SOURCE,
);

export const AuthDevicesProvision = rpcAction(
  API_ID,
  "Auth.Devices.Provision",
  {
    subject: "rpc.v1.Auth.Devices.Provision",
    input: schema<Types.AuthDevicesProvisionInput>(
      AuthDevicesProvisionRequestSchema,
    ),
    output: schema<Types.AuthDevicesProvisionOutput>(
      AuthDevicesProvisionResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesProvision",
  ACTION_SOURCE,
);

export const AuthDevicesRemove = rpcAction(
  API_ID,
  "Auth.Devices.Remove",
  {
    subject: "rpc.v1.Auth.Devices.Remove",
    input: schema<Types.AuthDevicesRemoveInput>(AuthDevicesRemoveRequestSchema),
    output: schema<Types.AuthDevicesRemoveOutput>(
      AuthDevicesRemoveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthDevicesRemove",
  ACTION_SOURCE,
);

export const AuthIdentityAuthorityGet = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.Get",
  {
    subject: "rpc.v1.Auth.IdentityAuthority.Get",
    input: schema<Types.AuthIdentityAuthorityGetInput>(
      AuthIdentityAuthorityGetRequestSchema,
    ),
    output: schema<Types.AuthIdentityAuthorityGetOutput>(
      AuthIdentityAuthorityGetResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthIdentityAuthorityGet",
  ACTION_SOURCE,
);

export const AuthIdentityAuthorityList = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.List",
  {
    subject: "rpc.v1.Auth.IdentityAuthority.List",
    input: schema<Types.AuthIdentityAuthorityListInput>(
      AuthIdentityAuthorityListRequestSchema,
    ),
    output: schema<Types.AuthIdentityAuthorityListOutput>(
      AuthIdentityAuthorityListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthIdentityAuthorityList",
  ACTION_SOURCE,
);

export const AuthIdentityAuthorityRevoke = rpcAction(
  API_ID,
  "Auth.IdentityAuthority.Revoke",
  {
    subject: "rpc.v1.Auth.IdentityAuthority.Revoke",
    input: schema<Types.AuthIdentityAuthorityRevokeInput>(
      AuthIdentityAuthorityRevokeRequestSchema,
    ),
    output: schema<Types.AuthIdentityAuthorityRevokeOutput>(
      AuthIdentityAuthorityRevokeResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthIdentityAuthorityRevoke",
  ACTION_SOURCE,
);

export const AuthPortalsGet = rpcAction(
  API_ID,
  "Auth.Portals.Get",
  {
    subject: "rpc.v1.Auth.Portals.Get",
    input: schema<Types.AuthPortalsGetInput>(AuthPortalsGetRequestSchema),
    output: schema<Types.AuthPortalsGetOutput>(AuthPortalsGetResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsGet",
  ACTION_SOURCE,
);

export const AuthPortalsList = rpcAction(
  API_ID,
  "Auth.Portals.List",
  {
    subject: "rpc.v1.Auth.Portals.List",
    input: schema<Types.AuthPortalsListInput>(AuthPortalsListRequestSchema),
    output: schema<Types.AuthPortalsListOutput>(AuthPortalsListResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsList",
  ACTION_SOURCE,
);

export const AuthPortalsLoginSettingsGet = rpcAction(
  API_ID,
  "Auth.Portals.LoginSettings.Get",
  {
    subject: "rpc.v1.Auth.Portals.LoginSettings.Get",
    input: schema<Types.AuthPortalsLoginSettingsGetInput>(
      AuthPortalsLoginSettingsGetRequestSchema,
    ),
    output: schema<Types.AuthPortalsLoginSettingsGetOutput>(
      AuthPortalsLoginSettingsGetResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsLoginSettingsGet",
  ACTION_SOURCE,
);

export const AuthPortalsLoginSettingsUpdate = rpcAction(
  API_ID,
  "Auth.Portals.LoginSettings.Update",
  {
    subject: "rpc.v1.Auth.Portals.LoginSettings.Update",
    input: schema<Types.AuthPortalsLoginSettingsUpdateInput>(
      AuthPortalsLoginSettingsUpdateRequestSchema,
    ),
    output: schema<Types.AuthPortalsLoginSettingsUpdateOutput>(
      AuthPortalsLoginSettingsUpdateResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsLoginSettingsUpdate",
  ACTION_SOURCE,
);

export const AuthPortalsPut = rpcAction(
  API_ID,
  "Auth.Portals.Put",
  {
    subject: "rpc.v1.Auth.Portals.Put",
    input: schema<Types.AuthPortalsPutInput>(AuthPortalsPutRequestSchema),
    output: schema<Types.AuthPortalsPutOutput>(AuthPortalsPutResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsPut",
  ACTION_SOURCE,
);

export const AuthPortalsRemove = rpcAction(
  API_ID,
  "Auth.Portals.Remove",
  {
    subject: "rpc.v1.Auth.Portals.Remove",
    input: schema<Types.AuthPortalsRemoveInput>(AuthPortalsRemoveRequestSchema),
    output: schema<Types.AuthPortalsRemoveOutput>(
      AuthPortalsRemoveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsRemove",
  ACTION_SOURCE,
);

export const AuthPortalsRoutesPut = rpcAction(
  API_ID,
  "Auth.Portals.Routes.Put",
  {
    subject: "rpc.v1.Auth.Portals.Routes.Put",
    input: schema<Types.AuthPortalsRoutesPutInput>(
      AuthPortalsRoutesPutRequestSchema,
    ),
    output: schema<Types.AuthPortalsRoutesPutOutput>(
      AuthPortalsRoutesPutResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsRoutesPut",
  ACTION_SOURCE,
);

export const AuthPortalsRoutesRemove = rpcAction(
  API_ID,
  "Auth.Portals.Routes.Remove",
  {
    subject: "rpc.v1.Auth.Portals.Routes.Remove",
    input: schema<Types.AuthPortalsRoutesRemoveInput>(
      AuthPortalsRoutesRemoveRequestSchema,
    ),
    output: schema<Types.AuthPortalsRoutesRemoveOutput>(
      AuthPortalsRoutesRemoveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthPortalsRoutesRemove",
  ACTION_SOURCE,
);

export const AuthServiceInstancesDisable = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Disable",
  {
    subject: "rpc.v1.Auth.ServiceInstances.Disable",
    input: schema<Types.AuthServiceInstancesDisableInput>(
      AuthServiceInstancesDisableRequestSchema,
    ),
    output: schema<Types.AuthServiceInstancesDisableOutput>(
      AuthServiceInstancesDisableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthServiceInstancesDisable",
  ACTION_SOURCE,
);

export const AuthServiceInstancesEnable = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Enable",
  {
    subject: "rpc.v1.Auth.ServiceInstances.Enable",
    input: schema<Types.AuthServiceInstancesEnableInput>(
      AuthServiceInstancesEnableRequestSchema,
    ),
    output: schema<Types.AuthServiceInstancesEnableOutput>(
      AuthServiceInstancesEnableResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthServiceInstancesEnable",
  ACTION_SOURCE,
);

export const AuthServiceInstancesList = rpcAction(
  API_ID,
  "Auth.ServiceInstances.List",
  {
    subject: "rpc.v1.Auth.ServiceInstances.List",
    input: schema<Types.AuthServiceInstancesListInput>(
      AuthServiceInstancesListRequestSchema,
    ),
    output: schema<Types.AuthServiceInstancesListOutput>(
      AuthServiceInstancesListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthServiceInstancesList",
  ACTION_SOURCE,
);

export const AuthServiceInstancesProvision = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Provision",
  {
    subject: "rpc.v1.Auth.ServiceInstances.Provision",
    input: schema<Types.AuthServiceInstancesProvisionInput>(
      AuthServiceInstancesProvisionRequestSchema,
    ),
    output: schema<Types.AuthServiceInstancesProvisionOutput>(
      AuthServiceInstancesProvisionResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthServiceInstancesProvision",
  ACTION_SOURCE,
);

export const AuthServiceInstancesRemove = rpcAction(
  API_ID,
  "Auth.ServiceInstances.Remove",
  {
    subject: "rpc.v1.Auth.ServiceInstances.Remove",
    input: schema<Types.AuthServiceInstancesRemoveInput>(
      AuthServiceInstancesRemoveRequestSchema,
    ),
    output: schema<Types.AuthServiceInstancesRemoveOutput>(
      AuthServiceInstancesRemoveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthServiceInstancesRemove",
  ACTION_SOURCE,
);

export const AuthSessionsList = rpcAction(
  API_ID,
  "Auth.Sessions.List",
  {
    subject: "rpc.v1.Auth.Sessions.List",
    input: schema<Types.AuthSessionsListInput>(AuthSessionsListRequestSchema),
    output: schema<Types.AuthSessionsListOutput>(
      AuthSessionsListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthSessionsList",
  ACTION_SOURCE,
);

export const AuthSessionsLogout = rpcAction(
  API_ID,
  "Auth.Sessions.Logout",
  {
    subject: "rpc.v1.Auth.Sessions.Logout",
    input: schema<Types.AuthSessionsLogoutInput>(
      AuthSessionsLogoutRequestSchema,
    ),
    output: schema<Types.AuthSessionsLogoutOutput>(
      AuthSessionsLogoutResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthSessionsLogout",
  ACTION_SOURCE,
);

export const AuthSessionsMe = rpcAction(
  API_ID,
  "Auth.Sessions.Me",
  {
    subject: "rpc.v1.Auth.Sessions.Me",
    input: schema<Types.AuthSessionsMeInput>(AuthSessionsMeRequestSchema),
    output: schema<Types.AuthSessionsMeOutput>(AuthSessionsMeResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthSessionsMe",
  ACTION_SOURCE,
);

export const AuthSessionsRevoke = rpcAction(
  API_ID,
  "Auth.Sessions.Revoke",
  {
    subject: "rpc.v1.Auth.Sessions.Revoke",
    input: schema<Types.AuthSessionsRevokeInput>(
      AuthSessionsRevokeRequestSchema,
    ),
    output: schema<Types.AuthSessionsRevokeOutput>(
      AuthSessionsRevokeResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthSessionsRevoke",
  ACTION_SOURCE,
);

export const AuthUserIdentitiesList = rpcAction(
  API_ID,
  "Auth.UserIdentities.List",
  {
    subject: "rpc.v1.Auth.UserIdentities.List",
    input: schema<Types.AuthUserIdentitiesListInput>(
      AuthUserIdentitiesListRequestSchema,
    ),
    output: schema<Types.AuthUserIdentitiesListOutput>(
      AuthUserIdentitiesListResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUserIdentitiesList",
  ACTION_SOURCE,
);

export const AuthUserIdentitiesUnlink = rpcAction(
  API_ID,
  "Auth.UserIdentities.Unlink",
  {
    subject: "rpc.v1.Auth.UserIdentities.Unlink",
    input: schema<Types.AuthUserIdentitiesUnlinkInput>(
      AuthUserIdentitiesUnlinkRequestSchema,
    ),
    output: schema<Types.AuthUserIdentitiesUnlinkOutput>(
      AuthUserIdentitiesUnlinkResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUserIdentitiesUnlink",
  ACTION_SOURCE,
);

export const AuthUsersCreate = rpcAction(
  API_ID,
  "Auth.Users.Create",
  {
    subject: "rpc.v1.Auth.Users.Create",
    input: schema<Types.AuthUsersCreateInput>(AuthUsersCreateRequestSchema),
    output: schema<Types.AuthUsersCreateOutput>(AuthUsersCreateResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersCreate",
  ACTION_SOURCE,
);

export const AuthUsersGet = rpcAction(
  API_ID,
  "Auth.Users.Get",
  {
    subject: "rpc.v1.Auth.Users.Get",
    input: schema<Types.AuthUsersGetInput>(AuthUsersGetRequestSchema),
    output: schema<Types.AuthUsersGetOutput>(AuthUsersGetResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersGet",
  ACTION_SOURCE,
);

export const AuthUsersIdentityLinkCreate = rpcAction(
  API_ID,
  "Auth.Users.IdentityLink.Create",
  {
    subject: "rpc.v1.Auth.Users.IdentityLink.Create",
    input: schema<Types.AuthUsersIdentityLinkCreateInput>(
      AuthUsersIdentityLinkCreateRequestSchema,
    ),
    output: schema<Types.AuthUsersIdentityLinkCreateOutput>(
      AuthUsersIdentityLinkCreateResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersIdentityLinkCreate",
  ACTION_SOURCE,
);

export const AuthUsersList = rpcAction(
  API_ID,
  "Auth.Users.List",
  {
    subject: "rpc.v1.Auth.Users.List",
    input: schema<Types.AuthUsersListInput>(AuthUsersListRequestSchema),
    output: schema<Types.AuthUsersListOutput>(AuthUsersListResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersList",
  ACTION_SOURCE,
);

export const AuthUsersPasswordChange = rpcAction(
  API_ID,
  "Auth.Users.Password.Change",
  {
    subject: "rpc.v1.Auth.Users.Password.Change",
    input: schema<Types.AuthUsersPasswordChangeInput>(
      AuthUsersPasswordChangeRequestSchema,
    ),
    output: schema<Types.AuthUsersPasswordChangeOutput>(
      AuthUsersPasswordChangeResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersPasswordChange",
  ACTION_SOURCE,
);

export const AuthUsersPasswordResetCreate = rpcAction(
  API_ID,
  "Auth.Users.PasswordReset.Create",
  {
    subject: "rpc.v1.Auth.Users.PasswordReset.Create",
    input: schema<Types.AuthUsersPasswordResetCreateInput>(
      AuthUsersPasswordResetCreateRequestSchema,
    ),
    output: schema<Types.AuthUsersPasswordResetCreateOutput>(
      AuthUsersPasswordResetCreateResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersPasswordResetCreate",
  ACTION_SOURCE,
);

export const AuthUsersResolve = rpcAction(
  API_ID,
  "Auth.Users.Resolve",
  {
    subject: "rpc.v1.Auth.Users.Resolve",
    input: schema<Types.AuthUsersResolveInput>(AuthUsersResolveRequestSchema),
    output: schema<Types.AuthUsersResolveOutput>(
      AuthUsersResolveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersResolve",
  ACTION_SOURCE,
);

export const AuthUsersUpdate = rpcAction(
  API_ID,
  "Auth.Users.Update",
  {
    subject: "rpc.v1.Auth.Users.Update",
    input: schema<Types.AuthUsersUpdateInput>(AuthUsersUpdateRequestSchema),
    output: schema<Types.AuthUsersUpdateOutput>(AuthUsersUpdateResponseSchema),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
  },
  "AuthUsersUpdate",
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesResolve = operationAction(
  API_ID,
  "Auth.DeviceUserAuthorities.Resolve",
  {
    subject: "operations.v1.Auth.DeviceUserAuthorities.Resolve",
    input: schema<Types.AuthDeviceUserAuthoritiesResolveInput>(
      AuthDeviceUserAuthoritiesResolveRequestSchema,
    ),
    progress: schema<Types.AuthDeviceUserAuthoritiesResolveProgress>(
      AuthDeviceUserAuthoritiesResolveProgressSchema,
    ),
    output: schema<Types.AuthDeviceUserAuthoritiesResolveOutput>(
      AuthDeviceUserAuthoritiesResolveResponseSchema,
    ),
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
        schema: schema<Types.AuthErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.AuthError.fromSerializable,
      },
      {
        type: "UnexpectedError",
        schema: schema<Types.UnexpectedErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.UnexpectedError.fromSerializable,
      },
      {
        type: "ValidationError",
        schema: schema<Types.ValidationErrorData>(AuthErrorDetailsSchema),
        fromSerializable: Types.ValidationError.fromSerializable,
      },
    ] as const,
    cancel: true,
  },
  "AuthDeviceUserAuthoritiesResolve",
  ACTION_SOURCE,
);

export const AuthConnectionsClosed = eventActions(
  API_ID,
  "Auth.Connections.Closed",
  {
    subject: "events.v1.Auth.Connections.Closed",
    event: schema<Types.AuthConnectionsClosedEvent>(
      AuthConnectionsClosedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthConnectionsClosed",
  false,
  ACTION_SOURCE,
);

export const AuthConnectionsKicked = eventActions(
  API_ID,
  "Auth.Connections.Kicked",
  {
    subject: "events.v1.Auth.Connections.Kicked",
    event: schema<Types.AuthConnectionsKickedEvent>(
      AuthConnectionsKickedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthConnectionsKicked",
  false,
  ACTION_SOURCE,
);

export const AuthConnectionsOpened = eventActions(
  API_ID,
  "Auth.Connections.Opened",
  {
    subject: "events.v1.Auth.Connections.Opened",
    event: schema<Types.AuthConnectionsOpenedEvent>(
      AuthConnectionsOpenedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthConnectionsOpened",
  false,
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesApproved = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Approved",
  {
    subject: "events.v1.Auth.DeviceUserAuthorities.Approved",
    params: ["/deploymentId"] as const,
    event: schema<Types.AuthDeviceUserAuthoritiesApprovedEvent>(
      AuthDeviceUserAuthoritiesApprovedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthDeviceUserAuthoritiesApproved",
  false,
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesRequested = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Requested",
  {
    subject: "events.v1.Auth.DeviceUserAuthorities.Requested",
    params: ["/deploymentId"] as const,
    event: schema<Types.AuthDeviceUserAuthoritiesRequestedEvent>(
      AuthDeviceUserAuthoritiesRequestedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthDeviceUserAuthoritiesRequested",
  false,
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesResolved = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.Resolved",
  {
    subject: "events.v1.Auth.DeviceUserAuthorities.Resolved",
    params: ["/deploymentId"] as const,
    event: schema<Types.AuthDeviceUserAuthoritiesResolvedEvent>(
      AuthDeviceUserAuthoritiesResolvedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthDeviceUserAuthoritiesResolved",
  false,
  ACTION_SOURCE,
);

export const AuthDeviceUserAuthoritiesReviewRequested = eventActions(
  API_ID,
  "Auth.DeviceUserAuthorities.ReviewRequested",
  {
    subject: "events.v1.Auth.DeviceUserAuthorities.ReviewRequested",
    params: ["/deploymentId"] as const,
    event: schema<Types.AuthDeviceUserAuthoritiesReviewRequestedEvent>(
      AuthDeviceUserAuthoritiesReviewRequestedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthDeviceUserAuthoritiesReviewRequested",
  false,
  ACTION_SOURCE,
);

export const AuthSessionsRevoked = eventActions(
  API_ID,
  "Auth.Sessions.Revoked",
  {
    subject: "events.v1.Auth.Sessions.Revoked",
    event: schema<Types.AuthSessionsRevokedEvent>(
      AuthSessionsRevokedEventSchema,
    ),
    publishCapabilities: [] as const,
    subscribeCapabilities: [] as const,
  },
  "AuthSessionsRevoked",
  false,
  ACTION_SOURCE,
);
