import { assertEquals } from "@std/assert";
import {
  type CallerRuntime,
  defineAppContract,
  defineDeviceContract,
} from "@qlever-llc/trellis";
import {
  base64urlEncode,
  deriveDeviceConfirmationCode,
  deriveDeviceIdentity,
  waitForDeviceActivation,
} from "@qlever-llc/trellis/auth";
import * as trellisAuth from "@trellis/apis/trellis.auth";
import { ulid } from "ulid";
import { nativeProtocolPresentation } from "../../packages/trellis/contract_support/protocol_artifacts.ts";
import { resolveNativeProtocolPresentation } from "../../packages/trellis/contract_support/protocol_resolution.ts";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import { integrationSlug } from "../_support/names.ts";

type ProvisionedDevice = {
  readonly device: {
    readonly deploymentId: string;
    readonly identityPublicKey: string | null;
    readonly instanceId: string;
    readonly principalId: string;
  };
};

type PlannedAuthority = {
  readonly proposal: {
    readonly baseAuthorityVersion: number | null;
    readonly classification: "initial" | "update" | "migration";
    readonly proposalId: string;
  };
};

type DeploymentAuthority = {
  readonly authority: {
    readonly authorityId: string;
    readonly version: number;
    readonly materialization: {
      readonly authorityVersion: number;
      readonly reconciledAt: number | null;
      readonly error: string | null;
      readonly state: "available" | "unavailable" | "error";
    } | null;
  };
};

type DeviceAuthorityList = {
  readonly entries: readonly {
    readonly deploymentId: string;
    readonly instanceId: string;
    readonly identityPublicKey: string | null;
    readonly state: string | null;
  }[];
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

function requireProvisionedDevice(value: unknown): ProvisionedDevice {
  if (!isRecord(value) || !isRecord(value.device)) {
    throw new Error("expected provisioned device response");
  }
  const device = value.device;
  if (
    typeof device.deploymentId !== "string" ||
    (device.identityPublicKey !== null &&
      typeof device.identityPublicKey !== "string") ||
    typeof device.instanceId !== "string" ||
    typeof device.principalId !== "string"
  ) {
    throw new Error("expected provisioned device instance fields");
  }
  return {
    device: {
      deploymentId: device.deploymentId,
      identityPublicKey: device.identityPublicKey,
      instanceId: device.instanceId,
      principalId: device.principalId,
    },
  };
}

function requirePlannedAuthority(value: unknown): PlannedAuthority {
  if (!isRecord(value) || !isRecord(value.proposal)) {
    throw new Error("expected authority plan response");
  }
  const proposal = value.proposal;
  if (
    proposal.classification !== "initial" &&
      proposal.classification !== "update" &&
      proposal.classification !== "migration" ||
    typeof proposal.proposalId !== "string" ||
    (proposal.baseAuthorityVersion !== null &&
      typeof proposal.baseAuthorityVersion !== "number")
  ) {
    throw new Error("expected authority plan fields");
  }
  return {
    proposal: {
      baseAuthorityVersion: proposal.baseAuthorityVersion,
      classification: proposal.classification,
      proposalId: proposal.proposalId,
    },
  };
}

function requireDeploymentAuthority(value: unknown): DeploymentAuthority {
  if (!isRecord(value) || !isRecord(value.authority)) {
    throw new Error("expected deployment authority response");
  }
  const authority = value.authority;
  if (
    typeof authority.authorityId !== "string" ||
    typeof authority.version !== "number"
  ) {
    throw new Error("expected deployment authority version");
  }
  const materializedAuthority = authority.materialization;
  if (materializedAuthority !== undefined && materializedAuthority !== null) {
    if (!isRecord(materializedAuthority)) {
      throw new Error("expected materialized authority object");
    }
    if (
      materializedAuthority.state !== "available" &&
        materializedAuthority.state !== "unavailable" &&
        materializedAuthority.state !== "error" ||
      typeof materializedAuthority.authorityVersion !== "number" ||
      (materializedAuthority.reconciledAt !== null &&
        typeof materializedAuthority.reconciledAt !== "number") ||
      (materializedAuthority.error !== null &&
        typeof materializedAuthority.error !== "string")
    ) {
      throw new Error("expected materialized authority status");
    }
    return {
      authority: {
        authorityId: authority.authorityId,
        version: authority.version,
        materialization: {
          state: materializedAuthority.state,
          authorityVersion: materializedAuthority.authorityVersion,
          reconciledAt: materializedAuthority.reconciledAt,
          error: materializedAuthority.error,
        },
      },
    };
  }
  return {
    authority: {
      authorityId: authority.authorityId,
      version: authority.version,
      materialization: null,
    },
  };
}

export function requireDeviceAuthorityList(
  value: unknown,
): DeviceAuthorityList {
  if (!isRecord(value) || !Array.isArray(value.entries)) {
    throw new Error("expected device authority list response");
  }
  const entries = value.entries.map((entry) => {
    if (!isRecord(entry)) {
      throw new Error("expected device authority entry");
    }
    if (
      !isRecord(entry.device) ||
      typeof entry.device.deploymentId !== "string" ||
      typeof entry.device.instanceId !== "string" ||
      (entry.device.identityPublicKey !== null &&
        typeof entry.device.identityPublicKey !== "string") ||
      typeof entry.device.state !== "string"
    ) {
      throw new Error("expected device authority entry fields");
    }
    return {
      deploymentId: entry.device.deploymentId,
      instanceId: entry.device.instanceId,
      identityPublicKey: entry.device.identityPublicKey,
      state: entry.device.state,
    };
  });
  return { entries };
}

export function createDeviceActivationFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const adminContract = defineAppContract(() => ({
    id: `integration.device-activation-admin.${slug}@v1`,
    apiId: `integration.device-activation-admin.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Device Activation Admin (${slug})`,
    description:
      "Admin participant for the device activation integration fixture.",
    uses: [
      trellisAuth.AuthDeploymentsCreate,
      trellisAuth.AuthDeploymentsDisable,
      trellisAuth.AuthDeploymentAuthorityAcceptMigration,
      trellisAuth.AuthDeploymentAuthorityAcceptUpdate,
      trellisAuth.AuthDeploymentAuthorityGet,
      trellisAuth.AuthDeploymentAuthorityPlan,
      trellisAuth.AuthDeploymentAuthorityReconcile,
      trellisAuth.AuthDevicesProvision,
      trellisAuth.AuthDeviceUserAuthoritiesList,
      trellisAuth.AuthDeviceUserAuthoritiesRevoke,
      trellisAuth.AuthDeviceUserAuthoritiesReviewsDecide,
      trellisAuth.AuthDeviceUserAuthoritiesReviewsList,
      trellisAuth.AuthDeviceUserAuthoritiesReviewRequested.subscribe,
      trellisAuth.AuthDeviceUserAuthoritiesRequested.subscribe,
      trellisAuth.AuthDeviceUserAuthoritiesApproved.subscribe,
      trellisAuth.AuthDeviceUserAuthoritiesResolved.subscribe,
      trellisAuth.AuthConnectionsList,
      trellisAuth.AuthServiceInstancesList,
      trellisAuth.AuthSessionsList,
      trellisAuth.AuthSessionsMe,
      trellisAuth.AuthSessionsRevoke,
      trellisAuth.AuthUsersCreate,
      trellisAuth.AuthUsersPasswordResetCreate,
      trellisAuth.AuthDeviceUserAuthoritiesResolve,
    ],
  }));

  const deviceContract = defineDeviceContract(() => ({
    id: `integration.device-activation-device.${slug}@v1`,
    apiId: `integration.device-activation-device.${slug}@v1`,
    apiVersion: "1.0.0",
    displayName: `Trellis Integration Activated Device (${slug})`,
    description:
      "Activated device participant for the device activation integration fixture.",
    uses: [trellisAuth.AuthSessionsMe],
  }));

  type DeviceActivationAdmin = CallerRuntime<typeof adminContract>;

  async function setupDeviceDeployment(
    runtime: LiveTrellisRuntime,
    reviewMode: "none" | "required" = "none",
  ) {
    const admin = await runtime.connectClient({
      name: `device-activation-fixture-admin-${slug}`,
      contract: adminContract,
    });
    const deploymentName = `device-activation-${ulid()}`;

    const deployment = await admin.authDeploymentsCreate({
      displayName: deploymentName,
      expiresAt: null,
      idempotencyKey: ulid(),
      kind: "device",
      participantId: deviceContract.CONTRACT_ID,
      portalId: null,
      requiresDeviceDelegation: false,
      reviewMode,
    }).orThrow();
    const deploymentId = deployment.deployment.deploymentId;
    await approveDeviceContract(admin, deploymentId);

    return { admin, deploymentId };
  }

  async function setupProvisionedDevice(
    admin: DeviceActivationAdmin,
    deploymentId: string,
  ) {
    const rootSecret = crypto.getRandomValues(new Uint8Array(32));
    const identity = await deriveDeviceIdentity(rootSecret);
    const provisioned = requireProvisionedDevice(
      await admin.authDevicesProvision({
        deploymentId,
        idempotencyKey: ulid(),
        identityPublicKey: identity.publicIdentityKey,
        instanceId: null,
        participantId: deviceContract.CONTRACT_ID,
      }).orThrow(),
    );
    assertEquals(provisioned.device.deploymentId, deploymentId);
    assertEquals(
      provisioned.device.identityPublicKey,
      identity.publicIdentityKey,
    );

    return { rootSecret, identity, provisioned };
  }

  async function setupActivationRequest(
    runtime: LiveTrellisRuntime,
    admin: DeviceActivationAdmin,
    deploymentId: string,
    identity: Awaited<ReturnType<typeof deriveDeviceIdentity>>,
    instanceId: string,
  ) {
    const presentation = await resolveNativeProtocolPresentation(
      deviceContract,
    );
    const nonce = base64urlEncode(crypto.getRandomValues(new Uint8Array(32)));
    const confirmationCode = await deriveDeviceConfirmationCode({
      activationKey: identity.activationKey,
      publicIdentityKey: identity.publicIdentityKey,
      nonce,
    });
    const controller = new AbortController();
    const bootstrap = waitForDeviceActivation({
      trellisUrl: runtime.trellisUrl,
      publicIdentityKey: identity.publicIdentityKey,
      identitySeed: identity.identitySeed,
      activationKey: identity.activationKey,
      deploymentId,
      instanceId,
      principalId: instanceId,
      participantId: deviceContract.CONTRACT_ID,
      participantArtifactDigest: deviceContract.CONTRACT_DIGEST,
      participantNeedsDigest: presentation.participantNeedsDigest,
      nonce,
      signal: controller.signal,
      pollIntervalMs: 25,
    });
    bootstrap.catch(() => undefined);
    const review = await runtime.waitFor(async () => {
      const reviews = await admin.authDeviceUserAuthoritiesReviewsList({
        deploymentId,
        state: "pending",
        limit: 20,
      }).orThrow();
      return reviews.entries.find((entry) =>
        entry.instanceId === instanceId &&
        entry.deploymentId === deploymentId
      );
    });
    controller.abort();
    await bootstrap.catch(() => undefined);

    return {
      confirmationCode,
      flowId: review.reviewId,
      participantNeedsDigest: presentation.participantNeedsDigest,
    };
  }

  async function approveDeviceContract(
    admin: DeviceActivationAdmin,
    deploymentId: string,
  ): Promise<void> {
    const presentation = nativeProtocolPresentation(deviceContract);
    const planned = requirePlannedAuthority(
      await admin.authDeploymentAuthorityPlan({
        deploymentId,
        expiresAt: null,
        idempotencyKey: ulid(),
        participantArtifact: presentation.participant,
        referencedApiArtifacts: [
          presentation.api,
          ...presentation.referencedApis,
        ],
      }).orThrow(),
    );

    const accepted = planned.proposal.classification === "migration"
      ? await admin.authDeploymentAuthorityAcceptMigration({
        expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
        idempotencyKey: ulid(),
        proposalId: planned.proposal.proposalId,
        reason: "Approved by isolated device activation integration test.",
      }).orThrow()
      : await admin.authDeploymentAuthorityAcceptUpdate({
        expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
        idempotencyKey: ulid(),
        proposalId: planned.proposal.proposalId,
        reason: null,
      }).orThrow();

    const authority = await admin.authDeploymentAuthorityGet({
      authorityId: accepted.authority.authorityId,
    }).orThrow();
    await admin.authDeploymentAuthorityReconcile({
      authorityId: authority.authority.authorityId,
      expectedVersion: authority.authority.version,
      idempotencyKey: ulid(),
    })
      .orThrow();
    await waitForDeviceDeploymentAuthority(
      admin,
      authority.authority.authorityId,
    );
  }

  async function waitForDeviceDeploymentAuthority(
    admin: DeviceActivationAdmin,
    authorityId: string,
  ): Promise<void> {
    const deadline = Date.now() + 15_000;
    while (Date.now() < deadline) {
      const current = requireDeploymentAuthority(
        await admin.authDeploymentAuthorityGet({ authorityId }).orThrow(),
      );
      const materialized = current.authority.materialization;
      if (materialized?.state === "error") {
        throw new Error(
          `device deployment authority reconciliation failed: ${
            materialized.error ?? "unknown error"
          }`,
        );
      }
      if (
        materialized?.state === "available" &&
        materialized.authorityVersion === current.authority.version &&
        materialized.reconciledAt !== null
      ) {
        return;
      }
      await new Promise((resolve) => setTimeout(resolve, 25));
    }
    throw new Error(
      `device deployment authority did not become ready for ${authorityId}`,
    );
  }

  return {
    slug,
    adminContract,
    deviceContract,
    setupDeviceDeployment,
    setupProvisionedDevice,
    setupActivationRequest,
  };
}
