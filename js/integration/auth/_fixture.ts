import {
  defineAgentContract,
  defineAppContract,
  defineServiceContract,
  Result,
} from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import type { AuthSessionsMeOutput } from "@qlever-llc/trellis/sdk/auth";
import * as trellisAuth from "@qlever-llc/trellis/sdk/auth";
import { Type } from "typebox";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import {
  caseScopedContractId,
  caseScopedName,
  caseScopedSubject,
  integrationSlug,
} from "../_support/names.ts";

export function createAuthLocalLoginFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const schemas = {
    PingInput: Type.Object({ message: Type.String() }),
    PingOutput: Type.Object({
      message: Type.String(),
      accepted: Type.Boolean(),
      participantKind: Type.Optional(Type.String()),
      serviceActive: Type.Optional(Type.Boolean()),
      serviceCapabilities: Type.Optional(Type.Array(Type.String())),
    }),
  } as const;

  const serviceContractId = caseScopedContractId(
    "trellis.integration.auth-local-login-service",
    caseId,
  );
  const pingCapability = serviceContractId.replace(/@v\d+$/, "") +
    "::authLocalLoginPing";
  const deploymentId = caseScopedName("auth-local-login-deployment", caseId);

  const serviceContract = defineServiceContract({ schemas }, (ref) => ({
    id: serviceContractId,
    displayName: `Trellis Integration Auth Local Login Service (${slug})`,
    description:
      "Service RPC used to prove an approved local-login app session can call services.",
    capabilities: {
      authLocalLoginPing: {
        displayName: "Call local-login ping",
        description: "Call the RPC used by the auth local-login fixture.",
      },
    },
    uses: [trellisAuth.AuthSessionsMe],
    rpc: {
      "AuthLogin.Ping": {
        version: "v1",
        subject: caseScopedSubject(
          "rpc.v1.Integration.AuthLocalLogin",
          caseId,
          "AuthLogin.Ping",
        ),
        input: ref.schema("PingInput"),
        output: ref.schema("PingOutput"),
        capabilities: { call: ["authLocalLoginPing"] },
        errors: [],
      },
    },
  }));

  const clientDisplayName =
    `Trellis Integration Auth Local Login Client (${slug})`;
  const updatedClientDisplayName =
    `Trellis Integration Auth Local Login Client Updated (${slug})`;
  const agentDisplayName =
    `Trellis Integration Auth Local Login Agent (${slug})`;

  const clientContract = defineAppContract(() => ({
    id: caseScopedContractId(
      "trellis.integration.auth-local-login-client",
      caseId,
    ),
    displayName: clientDisplayName,
    description: "App participant for the auth local-login binding fixture.",
    uses: [
      trellisAuth.AuthSessionsLogout,
      trellisAuth.AuthSessionsMe,
      serviceContract.AuthLoginPing,
    ],
  }));

  const agentContract = defineAgentContract(() => ({
    id: caseScopedContractId(
      "trellis.integration.auth-local-login-agent",
      caseId,
    ),
    displayName: agentDisplayName,
    description: "Agent participant for the auth local-login binding fixture.",
    uses: [trellisAuth.AuthSessionsMe, serviceContract.AuthLoginPing],
  }));

  const updatedClientContract = defineAppContract(() => ({
    id: caseScopedContractId(
      "trellis.integration.auth-local-login-client",
      caseId,
    ),
    displayName: updatedClientDisplayName,
    description:
      "Updated app participant for proving local-login rebinds refresh authority.",
    uses: [
      trellisAuth.AuthSessionsMe,
      trellisAuth.AuthConnectionsList,
      serviceContract.AuthLoginPing,
    ],
  }));

  const sessionAdminContract = defineAppContract(() => ({
    id: caseScopedContractId(
      "trellis.integration.auth-session-revoke-admin",
      caseId,
    ),
    displayName: `Trellis Integration Auth Session Revoke Admin (${slug})`,
    description:
      "Admin participant for revoking app sessions through public Auth RPCs.",
    uses: [
      trellisAuth.AuthConnectionsList,
      trellisAuth.AuthCapabilityGroupsDelete,
      trellisAuth.AuthCapabilityGroupsGet,
      trellisAuth.AuthCapabilityGroupsList,
      trellisAuth.AuthCapabilityGroupsPut,
      trellisAuth.AuthDeploymentAuthorityGrantOverridesList,
      trellisAuth.AuthDeploymentAuthorityGrantOverridesPut,
      trellisAuth.AuthDeploymentAuthorityGrantOverridesRemove,
      trellisAuth.AuthIdentityGrantsList,
      trellisAuth.AuthIdentityGrantsRevoke,
      trellisAuth.AuthPortalsLoginSettingsUpdate,
      trellisAuth.AuthPortalsList,
      trellisAuth.AuthPortalsPut,
      trellisAuth.AuthPortalsRemove,
      trellisAuth.AuthPortalsRoutesPut,
      trellisAuth.AuthPortalsRoutesRemove,
      trellisAuth.AuthSessionsList,
      trellisAuth.AuthSessionsMe,
      trellisAuth.AuthSessionsRevoke,
      trellisAuth.AuthUserIdentitiesList,
      trellisAuth.AuthUserIdentitiesUnlink,
      trellisAuth.AuthUsersCreate,
      trellisAuth.AuthUsersGet,
      trellisAuth.AuthUsersList,
      trellisAuth.AuthUsersPasswordResetCreate,
      trellisAuth.AuthUsersUpdate,
    ],
  }));

  const serviceName = caseScopedName(
    "auth-local-login-fixture-service",
    caseId,
  );
  const clientName = caseScopedName("auth-local-login-fixture-client", caseId);

  async function setupServiceWithKey(
    runtime: LiveTrellisRuntime,
    deployment = deploymentId,
  ) {
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
      deployment,
    });
    const service = await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      identity: serviceKey,
      telemetry: false,
      server: { log: false },
    }).orThrow();

    service.handleAuthLoginPing(async ({ input, client }) => {
      if (input.message !== "sessions-me") {
        return Result.ok({ message: input.message, accepted: true });
      }
      const me: AuthSessionsMeOutput = await client.authSessionsMe({})
        .orThrow();
      return Result.ok({
        message: input.message,
        accepted: true,
        participantKind: me.participantKind,
        serviceActive: me.service?.active,
        serviceCapabilities: me.service?.capabilities,
      });
    });

    return { service, serviceKey };
  }

  async function setupService(
    runtime: LiveTrellisRuntime,
    deployment = deploymentId,
  ) {
    const { service } = await setupServiceWithKey(runtime, deployment);
    return service;
  }

  async function setupClientRegistration(runtime: LiveTrellisRuntime) {
    const clientKey = await runtime.registerClient({
      name: clientName,
      contract: clientContract,
    });
    return { clientKey, clientAuth: runtime.clientAuth(clientKey) };
  }

  async function setupSessionAdmin(runtime: LiveTrellisRuntime) {
    return await runtime.connectClient({
      name: caseScopedName("auth-session-revoke-fixture-admin", caseId),
      contract: sessionAdminContract,
    });
  }

  return {
    agentContract,
    agentDisplayName,
    clientContract,
    clientDisplayName,
    clientName,
    deploymentId,
    pingMessage: caseScopedName("auth-local-login", caseId),
    pingCapability,
    serviceContractId,
    setupClientRegistration,
    setupSessionAdmin,
    setupService,
    setupServiceWithKey,
    sessionAdminContract,
    updatedClientContract,
    updatedClientDisplayName,
  };
}
