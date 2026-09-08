import type { CallerRuntime } from "@qlever-llc/trellis";
import { apis, participants } from "../../trellis/index.js";

import type { Static, TSchema } from "typebox";

export const adminParticipant = participants.appCli.participant;

export const ADMIN_USERNAME = "admin";
export const ADMIN_PARTICIPANT = {
  id: participants.appCli.participant.id,
  artifactDigest: participants.appCli.participant.digest,
} as const;

export type AdminClient = CallerRuntime<
  typeof participants.appCli.participant
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
  authDeploymentsGet: adminMethod(
    apis.auth.AuthDeploymentsGetRequestSchema,
    apis.auth.AuthDeploymentsGetResponseSchema,
    (client, input) => client.authDeploymentsGet(input).orThrow(),
  ),
  authDeploymentsApply: adminMethod(
    apis.auth.AuthDeploymentsApplyRequestSchema,
    apis.auth.AuthDeploymentsApplyResponseSchema,
    (client, input) => client.authDeploymentsApply(input).orThrow(),
  ),
  authParticipantsInstall: adminMethod(
    apis.auth.AuthParticipantsInstallRequestSchema,
    apis.auth.AuthParticipantsInstallResponseSchema,
    (client, input) => client.authParticipantsInstall(input).orThrow(),
  ),
  authGrantsGet: adminMethod(
    apis.auth.AuthGrantsGetRequestSchema,
    apis.auth.AuthGrantsGetResponseSchema,
    (client, input) => client.authGrantsGet(input).orThrow(),
  ),
  authGrantsList: adminMethod(
    apis.auth.AuthGrantsListRequestSchema,
    apis.auth.AuthGrantsListResponseSchema,
    (client, input) => client.authGrantsList(input).orThrow(),
  ),
  authGrantsSet: adminMethod(
    apis.auth.AuthGrantsSetRequestSchema,
    apis.auth.AuthGrantsMutationResponseSchema,
    (client, input) => client.authGrantsSet(input).orThrow(),
  ),
  authGrantsRevoke: adminMethod(
    apis.auth.AuthGrantsRevokeRequestSchema,
    apis.auth.AuthGrantsMutationResponseSchema,
    (client, input) => client.authGrantsRevoke(input).orThrow(),
  ),
  authServiceInstancesProvision: adminMethod(
    apis.auth.AuthServiceInstancesProvisionRequestSchema,
    apis.auth.AuthServiceInstancesProvisionResponseSchema,
    (client, input) => client.authServiceInstancesProvision(input).orThrow(),
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

export type AdminRpcInput<M extends TrellisTestAdminRpcMethod> =
  AdminRpc[M]["input"];

export type TrellisTestAdminRpcMethod = keyof typeof adminMethods;
