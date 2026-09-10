import type { CallerRuntime } from "@qlever-llc/trellis";
import type { Codec } from "@qlever-llc/trellis/generated";
import { apis, participants } from "../../trellis/index.js";

export const adminParticipant = participants.cli.participant;

export const ADMIN_USERNAME = "admin";

export type AdminClient = CallerRuntime<
  typeof participants.cli.participant
>;

function adminMethod<I, O>(
  descriptor: { input: Codec<I>; output: Codec<O> },
  call: (client: AdminClient, input: I) => Promise<O>,
) {
  return {
    input: descriptor.input,
    output: descriptor.output,
    call: (client: AdminClient, value: unknown) => call(client, value as I),
  } as const;
}

/** @internal Concrete Auth RPCs available to the shared test host. */
export const adminMethods = {
  authCapabilityGroupsPut: adminMethod(
    apis.auth.API.actions["rpc:CapabilityGroups.Put"],
    (client, input) => client.capabilityGroupsPut(input).orThrow(),
  ),
  authConnectionsList: adminMethod(
    apis.auth.API.actions["rpc:Connections.List"],
    (client, input) => client.connectionsList(input).orThrow(),
  ),
  authPortalsGrantOverridesRemove: adminMethod(
    apis.auth.API.actions["rpc:Portals.GrantOverrides.Remove"],
    (client, input) => client.portalsGrantOverridesRemove(input).orThrow(),
  ),
  authPortalsGrantOverridesPut: adminMethod(
    apis.auth.API.actions["rpc:Portals.GrantOverrides.Put"],
    (client, input) => client.portalsGrantOverridesPut(input).orThrow(),
  ),
  authPortalsGet: adminMethod(
    apis.auth.API.actions["rpc:Portals.Get"],
    (client, input) => client.portalsGet(input).orThrow(),
  ),
  authPortalsList: adminMethod(
    apis.auth.API.actions["rpc:Portals.List"],
    (client, input) => client.portalsList(input).orThrow(),
  ),
  authPortalsLoginSettingsUpdate: adminMethod(
    apis.auth.API.actions["rpc:Portals.LoginSettings.Update"],
    (client, input) => client.portalsLoginSettingsUpdate(input).orThrow(),
  ),
  authPortalsPut: adminMethod(
    apis.auth.API.actions["rpc:Portals.Put"],
    (client, input) => client.portalsPut(input).orThrow(),
  ),
  authPortalsRoutesPut: adminMethod(
    apis.auth.API.actions["rpc:Portals.Routes.Put"],
    (client, input) => client.portalsRoutesPut(input).orThrow(),
  ),
  authDevicesProvision: adminMethod(
    apis.auth.API.actions["rpc:Devices.Provision"],
    (client, input) => client.devicesProvision(input).orThrow(),
  ),
  stateAdminDelete: adminMethod(
    apis.state.API.actions["rpc:Admin.Delete"],
    (client, input) => client.adminDelete(input).orThrow(),
  ),
  stateAdminGet: adminMethod(
    apis.state.API.actions["rpc:Admin.Get"],
    (client, input) => client.adminGet(input).orThrow(),
  ),
  stateAdminList: adminMethod(
    apis.state.API.actions["rpc:Admin.List"],
    (client, input) => client.adminList(input).orThrow(),
  ),
  authDeploymentsCreate: adminMethod(
    apis.auth.API.actions["rpc:Deployments.Create"],
    (client, input) => client.deploymentsCreate(input).orThrow(),
  ),
  authDeploymentsGet: adminMethod(
    apis.auth.API.actions["rpc:Deployments.Get"],
    (client, input) => client.deploymentsGet(input).orThrow(),
  ),
  authDeploymentsApply: adminMethod(
    apis.auth.API.actions["rpc:Deployments.Apply"],
    (client, input) => client.deploymentsApply(input).orThrow(),
  ),
  authParticipantsInstall: adminMethod(
    apis.auth.API.actions["rpc:Participants.Install"],
    (client, input) => client.participantsInstall(input).orThrow(),
  ),
  authGrantsGet: adminMethod(
    apis.auth.API.actions["rpc:Grants.Get"],
    (client, input) => client.grantsGet(input).orThrow(),
  ),
  authGrantsList: adminMethod(
    apis.auth.API.actions["rpc:Grants.List"],
    (client, input) => client.grantsList(input).orThrow(),
  ),
  authGrantsSet: adminMethod(
    apis.auth.API.actions["rpc:Grants.Set"],
    (client, input) => client.grantsSet(input).orThrow(),
  ),
  authGrantsRevoke: adminMethod(
    apis.auth.API.actions["rpc:Grants.Revoke"],
    (client, input) => client.grantsRevoke(input).orThrow(),
  ),
  authServiceInstancesProvision: adminMethod(
    apis.auth.API.actions["rpc:ServiceInstances.Provision"],
    (client, input) => client.serviceInstancesProvision(input).orThrow(),
  ),
  authSessionsRevoke: adminMethod(
    apis.auth.API.actions["rpc:Sessions.Revoke"],
    (client, input) => client.sessionsRevoke(input).orThrow(),
  ),
} as const;

export type AdminRpc = {
  [M in keyof typeof adminMethods]: {
    input: ReturnType<(typeof adminMethods)[M]["input"]["decode"]>;
    output: ReturnType<(typeof adminMethods)[M]["output"]["decode"]>;
  };
};

export type AdminRpcInput<M extends TrellisTestAdminRpcMethod> =
  AdminRpc[M]["input"];

export type TrellisTestAdminRpcMethod = keyof typeof adminMethods;
