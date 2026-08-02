import { type CallerRuntime, defineAppContract } from "@qlever-llc/trellis";
import {
  AuthDeploymentAuthorityAcceptMigration,
  AuthDeploymentAuthorityAcceptMigrationRequestSchema,
  AuthDeploymentAuthorityAcceptMigrationResponseSchema,
  AuthDeploymentAuthorityAcceptUpdate,
  AuthDeploymentAuthorityAcceptUpdateRequestSchema,
  AuthDeploymentAuthorityAcceptUpdateResponseSchema,
  AuthDeploymentAuthorityGet,
  AuthDeploymentAuthorityGetRequestSchema,
  AuthDeploymentAuthorityGetResponseSchema,
  AuthDeploymentAuthorityList,
  AuthDeploymentAuthorityListRequestSchema,
  AuthDeploymentAuthorityListResponseSchema,
  AuthDeploymentAuthorityPlan,
  AuthDeploymentAuthorityPlanRequestSchema,
  AuthDeploymentAuthorityPlanResponseSchema,
  AuthDeploymentAuthorityPlansList,
  AuthDeploymentAuthorityPlansListRequestSchema,
  AuthDeploymentAuthorityPlansListResponseSchema,
  AuthDeploymentAuthorityReconcile,
  AuthDeploymentAuthorityReconcileRequestSchema,
  AuthDeploymentAuthorityReconcileResponseSchema,
  AuthDeploymentAuthorityReject,
  AuthDeploymentAuthorityRejectRequestSchema,
  AuthDeploymentAuthorityRejectResponseSchema,
  AuthDeploymentsCreate,
  AuthDeploymentsCreateRequestSchema,
  AuthDeploymentsCreateResponseSchema,
  AuthServiceInstancesProvision,
  AuthServiceInstancesProvisionRequestSchema,
  AuthServiceInstancesProvisionResponseSchema,
  AuthSessionsRevoke,
  AuthSessionsRevokeRequestSchema,
  AuthSessionsRevokeResponseSchema,
} from "@qlever-llc/trellis/sdk/auth";
import type { Static, TSchema } from "typebox";

export const ADMIN_USERNAME = "admin";
export const ADMIN_PARTICIPANT = {
  id: "trellis-platform-administration",
  artifactDigest: "lQoimvKOcLmB4Acn3Q5roDNXQe4KlY3RjUvJ10hJ6CY",
  needsDigest: "bqA3XWyeUSFZUzDOLAjpCODp__crKL4hwd6mVf7nrIU",
} as const;

export const adminContract = defineAppContract(() => ({
  id: "trellis.test.admin@v1",
  displayName: "Trellis Test Admin",
  description:
    "Automates Trellis test runtime administration through Auth RPCs.",
  uses: [
    AuthDeploymentAuthorityAcceptMigration,
    AuthDeploymentAuthorityAcceptUpdate,
    AuthDeploymentAuthorityGet,
    AuthDeploymentAuthorityList,
    AuthDeploymentAuthorityPlan,
    AuthDeploymentAuthorityReject,
    AuthDeploymentAuthorityPlansList,
    AuthDeploymentAuthorityReconcile,
    AuthDeploymentsCreate,
    AuthServiceInstancesProvision,
    AuthSessionsRevoke,
  ],
}));

export type AdminClient = CallerRuntime<typeof adminContract>;

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
  authDeploymentsCreate: adminMethod(
    AuthDeploymentsCreateRequestSchema,
    AuthDeploymentsCreateResponseSchema,
    (client, input) => client.authDeploymentsCreate(input).orThrow(),
  ),
  authDeploymentAuthorityPlan: adminMethod(
    AuthDeploymentAuthorityPlanRequestSchema,
    AuthDeploymentAuthorityPlanResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlan(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptUpdate: adminMethod(
    AuthDeploymentAuthorityAcceptUpdateRequestSchema,
    AuthDeploymentAuthorityAcceptUpdateResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptUpdate(input).orThrow(),
  ),
  authDeploymentAuthorityAcceptMigration: adminMethod(
    AuthDeploymentAuthorityAcceptMigrationRequestSchema,
    AuthDeploymentAuthorityAcceptMigrationResponseSchema,
    (client, input) =>
      client.authDeploymentAuthorityAcceptMigration(input).orThrow(),
  ),
  authDeploymentAuthorityList: adminMethod(
    AuthDeploymentAuthorityListRequestSchema,
    AuthDeploymentAuthorityListResponseSchema,
    (client, input) => client.authDeploymentAuthorityList(input).orThrow(),
  ),
  authDeploymentAuthorityReconcile: adminMethod(
    AuthDeploymentAuthorityReconcileRequestSchema,
    AuthDeploymentAuthorityReconcileResponseSchema,
    (client, input) => client.authDeploymentAuthorityReconcile(input).orThrow(),
  ),
  authDeploymentAuthorityGet: adminMethod(
    AuthDeploymentAuthorityGetRequestSchema,
    AuthDeploymentAuthorityGetResponseSchema,
    (client, input) => client.authDeploymentAuthorityGet(input).orThrow(),
  ),
  authServiceInstancesProvision: adminMethod(
    AuthServiceInstancesProvisionRequestSchema,
    AuthServiceInstancesProvisionResponseSchema,
    (client, input) => client.authServiceInstancesProvision(input).orThrow(),
  ),
  authDeploymentAuthorityPlansList: adminMethod(
    AuthDeploymentAuthorityPlansListRequestSchema,
    AuthDeploymentAuthorityPlansListResponseSchema,
    (client, input) => client.authDeploymentAuthorityPlansList(input).orThrow(),
  ),
  authDeploymentAuthorityReject: adminMethod(
    AuthDeploymentAuthorityRejectRequestSchema,
    AuthDeploymentAuthorityRejectResponseSchema,
    (client, input) => client.authDeploymentAuthorityReject(input).orThrow(),
  ),
  authSessionsRevoke: adminMethod(
    AuthSessionsRevokeRequestSchema,
    AuthSessionsRevokeResponseSchema,
    (client, input) => client.authSessionsRevoke(input).orThrow(),
  ),
} as const;

export type AdminRpc = {
  [M in keyof typeof adminMethods]: {
    input: Static<(typeof adminMethods)[M]["input"]>;
    output: Static<(typeof adminMethods)[M]["output"]>;
  };
};

export type TrellisTestAdminRpcMethod = keyof typeof adminMethods;
