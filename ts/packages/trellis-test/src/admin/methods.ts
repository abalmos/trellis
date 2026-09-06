import type { CallerRuntime } from "@qlever-llc/trellis";
import { apis, participants } from "../../trellis/index.js";

import type { Static, TSchema } from "typebox";

export const adminParticipant = participants.testAdmin.participant;

export const ADMIN_USERNAME = "admin";
export const ADMIN_PARTICIPANT = {
  id: participants.testAdmin.participant.id,
  artifactDigest: participants.testAdmin.participant.digest,
} as const;

export type AdminClient = CallerRuntime<
  typeof participants.testAdmin.participant
>;

function adminMethod<const I extends TSchema, const O extends TSchema>(
  input: I,
  output: O,
  call: (client: AdminClient, input: Static<I>) => Promise<Static<O>>,
) {
  return {
    input,
    output,
    call: (client: AdminClient, value: unknown) =>
      call(client, value as Static<I>),
  } as const;
}

/** @internal Concrete Auth RPCs available to the shared test host. */
export const adminMethods = {
  authCapabilityGroupsPut: adminMethod(
    apis.auth.AuthCapabilityGroupsPutRequestSchema,
    apis.auth.AuthCapabilityGroupsPutResponseSchema,
    (client, input) => client.authCapabilityGroupsPut(input).orThrow(),
  ),
  authConnectionsList: adminMethod(
    apis.auth.AuthConnectionsListRequestSchema,
    apis.auth.AuthConnectionsListResponseSchema,
    (client, input) => client.authConnectionsList(input).orThrow(),
  ),
  authPortalsGrantOverridesRemove: adminMethod(
    apis.auth.AuthPortalsGrantOverridesRemoveRequestSchema,
    apis.auth.AuthPortalsGrantOverridesRemoveResponseSchema,
    (client, input) => client.authPortalsGrantOverridesRemove(input).orThrow(),
  ),
  authPortalsGrantOverridesPut: adminMethod(
    apis.auth.AuthPortalsGrantOverridesPutRequestSchema,
    apis.auth.AuthPortalsGrantOverridesPutResponseSchema,
    (client, input) => client.authPortalsGrantOverridesPut(input).orThrow(),
  ),
  authPortalsGet: adminMethod(
    apis.auth.AuthPortalsGetRequestSchema,
    apis.auth.AuthPortalsGetResponseSchema,
    (client, input) => client.authPortalsGet(input).orThrow(),
  ),
  authPortalsList: adminMethod(
    apis.auth.AuthPortalsListRequestSchema,
    apis.auth.AuthPortalsListResponseSchema,
    (client, input) => client.authPortalsList(input).orThrow(),
  ),
  authPortalsLoginSettingsUpdate: adminMethod(
    apis.auth.AuthPortalsLoginSettingsUpdateRequestSchema,
    apis.auth.AuthPortalsLoginSettingsGetResponseSchema,
    (client, input) => client.authPortalsLoginSettingsUpdate(input).orThrow(),
  ),
  authPortalsPut: adminMethod(
    apis.auth.AuthPortalsPutRequestSchema,
    apis.auth.AuthPortalsPutResponseSchema,
    (client, input) => client.authPortalsPut(input).orThrow(),
  ),
  authPortalsRoutesPut: adminMethod(
    apis.auth.AuthPortalsRoutesPutRequestSchema,
    apis.auth.AuthPortalsRoutesPutResponseSchema,
    (client, input) => client.authPortalsRoutesPut(input).orThrow(),
  ),
  authDevicesProvision: adminMethod(
    apis.auth.AuthDevicesProvisionRequestSchema,
    apis.auth.AuthDevicesProvisionResponseSchema,
    (client, input) => client.authDevicesProvision(input).orThrow(),
  ),
  stateAdminDelete: adminMethod(
    apis.state.StateAdminDeleteRequestSchema,
    apis.state.StateAdminDeleteResponseSchema,
    (client, input) => client.stateAdminDelete(input).orThrow(),
  ),
  stateAdminGet: adminMethod(
    apis.state.StateAdminGetRequestSchema,
    apis.state.StateAdminGetResponseSchema,
    (client, input) => client.stateAdminGet(input).orThrow(),
  ),
  stateAdminList: adminMethod(
    apis.state.StateAdminListRequestSchema,
    apis.state.StateAdminListResponseSchema,
    (client, input) => client.stateAdminList(input).orThrow(),
  ),
  authDeploymentsCreate: adminMethod(
    apis.auth.AuthDeploymentsCreateRequestSchema,
    apis.auth.AuthDeploymentsCreateResponseSchema,
    (client, input) => client.authDeploymentsCreate(input).orThrow(),
  ),
  authDeploymentAuthorityPlan: adminMethod(
    apis.auth.AuthDeploymentAuthorityPlanRequestSchema,
    apis.auth.AuthDeploymentAuthorityPlanResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlan(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptUpdate: adminMethod(
    apis.auth.AuthDeploymentAuthorityAcceptUpdateRequestSchema,
    apis.auth.AuthDeploymentAuthorityAcceptUpdateResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptUpdate(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptMigration: adminMethod(
    apis.auth.AuthDeploymentAuthorityAcceptMigrationRequestSchema,
    apis.auth.AuthDeploymentAuthorityAcceptMigrationResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptMigration(input).orThrow(),
  ),
  authDeploymentAuthorityList: adminMethod(
    apis.auth.AuthDeploymentAuthorityListRequestSchema,
    apis.auth.AuthDeploymentAuthorityListResponseSchema,
    (client, input) => client.authDeploymentAuthorityList(input).orThrow(),
  ),
  authDeploymentAuthorityReconcile: adminMethod(
    apis.auth.AuthDeploymentAuthorityReconcileRequestSchema,
    apis.auth.AuthDeploymentAuthorityReconcileResponseSchema,
    (client, input) => client.authDeploymentAuthorityReconcile(input).orThrow(),
  ),
  authDeploymentAuthorityGet: adminMethod(
    apis.auth.AuthDeploymentAuthorityGetRequestSchema,
    apis.auth.AuthDeploymentAuthorityGetResponseSchema,
    (client, input) => client.authDeploymentAuthorityGet(input).orThrow(),
  ),
  authServiceInstancesProvision: adminMethod(
    apis.auth.AuthServiceInstancesProvisionRequestSchema,
    apis.auth.AuthServiceInstancesProvisionResponseSchema,
    (client, input) => client.authServiceInstancesProvision(input).orThrow(),
  ),
  authDeploymentAuthorityPlansList: adminMethod(
    apis.auth.AuthDeploymentAuthorityPlansListRequestSchema,
    apis.auth.AuthDeploymentAuthorityPlansListResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlansList(input).orThrow(),
  ),
  authDeploymentAuthorityReject: adminMethod(
    apis.auth.AuthDeploymentAuthorityRejectRequestSchema,
    apis.auth.AuthDeploymentAuthorityRejectResponseSchema,
    (client, input) => client.authDeploymentAuthorityReject(input).orThrow(),
  ),
  authSessionsRevoke: adminMethod(
    apis.auth.AuthSessionsRevokeRequestSchema,
    apis.auth.AuthSessionsRevokeResponseSchema,
    (client, input) => client.authSessionsRevoke(input).orThrow(),
  ),
} as const;

export type AdminRpc = {
  [M in keyof typeof adminMethods]: {
    input: Static<(typeof adminMethods)[M]["input"]>;
    output: Static<(typeof adminMethods)[M]["output"]>;
  };
};

export type AdminRpcInput<M extends TrellisTestAdminRpcMethod> = M extends
  "authDeploymentAuthorityPlan" ? AdminRpc[M]["input"] & {
    referencedApiArtifacts: readonly Record<string, unknown>[];
  }
  : AdminRpc[M]["input"];

export type TrellisTestAdminRpcMethod = keyof typeof adminMethods;
