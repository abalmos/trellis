import { createAuth, isJsonValue, type JsonValue } from "@qlever-llc/trellis";
import { ulid } from "ulid";

import { generateSessionSeed } from "../control_plane_config.ts";
import type {
  TrellisTestParticipantApproval,
  TrellisTestParticipantLike,
  TrellisTestServiceKey,
} from "../types.ts";
import { recordTrellisDuration } from "./metrics.ts";
import type {
  AdminRpc,
  AdminRpcInput,
  TrellisTestAdminRpcMethod,
} from "./methods.ts";

type JsonObject = Record<string, JsonValue>;

function checkedObject(value: Readonly<Record<string, unknown>>): JsonObject {
  if (!Object.values(value).every(isJsonValue)) {
    throw new Error(
      "Generated participant evidence must contain only JSON values",
    );
  }
  return value as JsonObject;
}

function participantPresentation(participant: TrellisTestParticipantLike) {
  const api = checkedObject(participant.api);
  const artifact = checkedObject(participant.artifact);
  const implementsApi = artifact.implements &&
      typeof artifact.implements === "object" &&
      !Array.isArray(artifact.implements) &&
      "self" in artifact.implements &&
      artifact.implements.self &&
      typeof artifact.implements.self === "object" &&
      !Array.isArray(artifact.implements.self)
    ? artifact.implements.self.api
    : undefined;
  if (api.id !== implementsApi || artifact.id !== participant.id) {
    throw new Error(
      "Generated participant identity does not match its artifacts",
    );
  }
  return {
    api,
    participant: artifact,
    referencedApis: participant.referencedApis.map(checkedObject),
  };
}

export type AdminDeploymentRpc = <M extends TrellisTestAdminRpcMethod>(
  method: M,
  input: AdminRpcInput<M>,
) => Promise<AdminRpc[M]["output"]>;

export type AdminDeploymentContext = {
  defaultDeployment: string;
  createdDeployments: Map<string, Promise<void>>;
  deploymentBindingRevisions: Map<string, number>;
  deploymentIds: Map<string, string>;
  installedParticipants: Map<string, { digest: string; revision: number }>;
  protocolApis: Map<string, JsonObject>;
  rpc: AdminDeploymentRpc;
};

function deploymentKey(kind: "service" | "device", deployment: string): string {
  return `${kind}:${deployment}`;
}

/** @internal Creates a service or device deployment. */
export async function createDeployment(
  context: AdminDeploymentContext,
  args: {
    deployment?: string;
    kind?: "service" | "device";
    reviewMode?: "none" | "required";
  } = {},
): Promise<void> {
  const deployment = args.deployment ?? context.defaultDeployment;
  const kind = args.kind ?? "service";
  const key = deploymentKey(kind, deployment);
  const existing = context.createdDeployments.get(key);
  if (existing !== undefined) return existing;
  const promise = (async () => {
    const created = await context.rpc("authDeploymentsCreate", {
      displayName: deployment,
      expiresAt: null,
      idempotencyKey: ulid(),
      kind,
      participantId: null,
      portalId: null,
      requiresDeviceDelegation: false,
      reviewMode: args.kind === "device" ? args.reviewMode ?? "none" : null,
    });
    context.deploymentIds.set(deployment, created.deployment.deploymentId);
    context.createdDeployments.set(key, Promise.resolve());
  })();
  context.createdDeployments.set(key, promise);
  void promise.catch(() => {
    if (context.createdDeployments.get(key) === promise) {
      context.createdDeployments.delete(key);
    }
  });
  await promise;
}

/** @internal Installs a participant and atomically replaces its deployment GrantBinding. */
export async function applyParticipant(
  context: AdminDeploymentContext,
  args: { deployment?: string; contract: TrellisTestParticipantLike },
): Promise<TrellisTestParticipantApproval> {
  const startedAt = performance.now();
  const deployment = args.deployment ?? context.defaultDeployment;
  if (!context.deploymentIds.has(deployment)) {
    await createDeployment(context, { deployment });
  }
  const deploymentId = context.deploymentIds.get(deployment);
  if (!deploymentId) {
    throw new Error(`Trellis deployment '${deployment}' was not created`);
  }

  const artifacts = participantPresentation(args.contract);
  const referencedApis = new Map(context.protocolApis);
  for (const api of artifacts.referencedApis) {
    referencedApis.set(String(api.id), api);
  }
  const applied = await context.rpc("authDeploymentsApply", {
    apiArtifacts: [artifacts.api, ...referencedApis.values()],
    deploymentId,
    expectedRevision: context.deploymentBindingRevisions.get(deploymentId) ?? 0,
    idempotencyKey: ulid(),
    participantArtifact: artifacts.participant,
  });
  if (applied.binding) {
    context.deploymentBindingRevisions.set(
      deploymentId,
      applied.binding.revision,
    );
    context.installedParticipants.set(String(artifacts.participant.id), {
      digest: String(artifacts.participant.digest),
      revision: applied.binding.installedRevision,
    });
  }
  context.protocolApis.set(String(artifacts.api.id), artifacts.api);
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - startedAt,
    {
      deployment,
      participantId: String(artifacts.participant.id),
      operation: "approve_contract",
      phase: "apply",
    },
  );
  return {
    participantId: String(artifacts.participant.id),
    installedRevision: Number(applied.binding?.installedRevision),
    deploymentId,
    binding: applied.binding,
  };
}

/** @internal Installs a participant definition without creating a deployment or grants. */
export async function installParticipant(
  context: AdminDeploymentContext,
  args: { contract: TrellisTestParticipantLike },
): Promise<TrellisTestParticipantApproval> {
  const artifacts = participantPresentation(args.contract);
  const participantId = String(artifacts.participant.id);
  const digest = String(artifacts.participant.digest);
  const current = context.installedParticipants.get(participantId);
  if (current?.digest === digest) {
    return { participantId, installedRevision: current.revision };
  }
  const installed = await context.rpc("authParticipantsInstall", {
    apiArtifacts: [artifacts.api, ...artifacts.referencedApis],
    expectedRevision: current?.revision ?? 0,
    idempotencyKey: ulid(),
    participantArtifact: artifacts.participant,
  });
  const revision = installed.participant.revision;
  context.installedParticipants.set(participantId, { digest, revision });
  return { participantId, installedRevision: revision };
}

/** @internal Provisions a service instance key after installing its participant. */
export async function provisionServiceInstance(
  context: AdminDeploymentContext,
  args: { deployment?: string; contract: TrellisTestParticipantLike },
): Promise<TrellisTestServiceKey> {
  const deployment = args.deployment ?? context.defaultDeployment;
  const approved = await applyParticipant(context, args);
  if (!approved.deploymentId) {
    throw new Error("deployment apply returned no deployment ID");
  }
  const identitySeed = generateSessionSeed();
  const auth = await createAuth({ sessionKeySeed: identitySeed });
  const provisioned = await context.rpc("authServiceInstancesProvision", {
    deploymentId: approved.deploymentId,
    idempotencyKey: ulid(),
    identityPublicKey: auth.sessionKey,
    instanceId: null,
    participantId: approved.participantId,
  });
  return {
    seed: identitySeed,
    deploymentId: approved.deploymentId,
    instanceId: provisioned.instance.instanceId,
    participantId: approved.participantId,
  };
}

/** @internal Runs the service registration sequence used by test services. */
export function registerService(
  context: AdminDeploymentContext,
  args: { deployment?: string; contract: TrellisTestParticipantLike },
): Promise<TrellisTestServiceKey> {
  return provisionServiceInstance(context, args);
}

/** @internal Provisions a service instance without changing its installed participant. */
export async function provisionServiceInstanceOnly(
  context: AdminDeploymentContext,
  args: { deployment?: string },
): Promise<{ seed: string; sessionKey: string }> {
  const deployment = args.deployment ?? context.defaultDeployment;
  const seed = generateSessionSeed();
  const auth = await createAuth({ sessionKeySeed: seed });
  const deploymentId = context.deploymentIds.get(deployment);
  if (!deploymentId) {
    throw new Error(`Trellis deployment '${deployment}' was not created`);
  }
  await context.rpc("authServiceInstancesProvision", {
    deploymentId,
    idempotencyKey: ulid(),
    identityPublicKey: auth.sessionKey,
    instanceId: null,
    participantId: null,
  });
  return { seed, sessionKey: auth.sessionKey };
}
