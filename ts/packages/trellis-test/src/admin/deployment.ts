import { createAuth } from "@qlever-llc/trellis";
import {
  type NativeProtocolPresentation,
  nativeProtocolPresentation,
} from "@qlever-llc/trellis/contracts";

import { generateSessionSeed } from "../control_plane_config.ts";
import type {
  TrellisTestAuthorityPlanClassification,
  TrellisTestContractApproval,
  TrellisTestContractLike,
  TrellisTestServiceKey,
} from "../types.ts";
import { waitFor } from "../wait.ts";
import { recordTrellisDuration } from "./metrics.ts";
import type {
  AdminRpc,
  AdminRpcInput,
  TrellisTestAdminRpcMethod,
} from "./methods.ts";

export type AdminDeploymentRpc = <M extends TrellisTestAdminRpcMethod>(
  method: M,
  input: AdminRpcInput<M>,
) => Promise<AdminRpc[M]["output"]>;

export type AdminDeploymentContext = {
  defaultDeployment: string;
  reconciliationMs: number;
  autoAccept: ReadonlySet<TrellisTestAuthorityPlanClassification>;
  createdDeployments: Map<string, Promise<void>>;
  deploymentIds: Map<string, string>;
  authorityIds: Map<string, string>;
  protocolApis: Map<string, NativeProtocolPresentation["api"]>;
  rpc: AdminDeploymentRpc;
};

function deploymentKey(deployment: string): string {
  return `service:${deployment}`;
}

function isAuthorityPlanClassification(
  value: string,
): value is TrellisTestAuthorityPlanClassification {
  return value === "initial" || value === "update" || value === "migration";
}

/** @internal Creates a service deployment through Auth.Deployments.Create. */
export async function createDeployment(
  context: AdminDeploymentContext,
  args: {
    deployment?: string;
    kind?: "service" | "device";
  } = {},
): Promise<void> {
  const deployment = args.deployment ?? context.defaultDeployment;
  const key = deploymentKey(deployment);
  const existing = context.createdDeployments.get(key);
  if (existing !== undefined) return existing;
  const startedAt = performance.now();
  const promise = (async () => {
    const created = await context.rpc("authDeploymentsCreate", {
      displayName: deployment,
      expiresAt: null,
      idempotencyKey: crypto.randomUUID(),
      kind: args.kind ?? "service",
      participantId: null,
      portalId: null,
      requiresDeviceDelegation: false,
      reviewMode: args.kind === "device" ? "none" : null,
    });
    context.deploymentIds.set(deployment, created.deployment.deploymentId);
    context.createdDeployments.set(key, Promise.resolve());
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - startedAt,
      { operation: "register_service", phase: "create_deployment" },
    );
  })();
  context.createdDeployments.set(key, promise);
  void promise.catch(() => {
    if (context.createdDeployments.get(key) === promise) {
      context.createdDeployments.delete(key);
    }
  });
  await promise;
}

/** @internal Plans, accepts, reconciles, and waits for a contract authority change. */
export async function approveContract(
  context: AdminDeploymentContext,
  args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    allowPlanClassifications?:
      readonly TrellisTestAuthorityPlanClassification[];
  },
): Promise<TrellisTestContractApproval> {
  const totalStartedAt = performance.now();
  const deployment = args.deployment ?? context.defaultDeployment;
  await createDeployment(context, { deployment });
  const deploymentId = context.deploymentIds.get(deployment);
  if (!deploymentId) {
    throw new Error(`Trellis deployment '${deployment}' was not created`);
  }
  const artifacts = nativeProtocolPresentation(args.contract);
  const referencedApis = new Map(context.protocolApis);
  for (const api of artifacts.referencedApis) {
    referencedApis.set(String(api.id), api);
  }
  const planStartedAt = performance.now();
  const planned = await context.rpc("authDeploymentAuthorityPlan", {
    deploymentId,
    expiresAt: null,
    idempotencyKey: crypto.randomUUID(),
    participantArtifact: artifacts.participant,
    referencedApiArtifacts: [artifacts.api, ...referencedApis.values()],
  });
  context.protocolApis.set(String(artifacts.api.id), artifacts.api);
  const classification = planned.proposal.classification;
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - planStartedAt,
    {
      deployment,
      participantId: String(artifacts.participant.id),
      operation: "approve_contract",
      phase: "plan",
      planClassification: classification,
    },
  );
  if (!isAuthorityPlanClassification(classification)) {
    throw new Error(
      `Trellis test runtime received unsupported authority plan classification '${classification}'`,
    );
  }
  const allowed = args.allowPlanClassifications === undefined
    ? context.autoAccept
    : new Set(args.allowPlanClassifications);
  if (!allowed.has(classification)) {
    throw new Error(
      `Trellis test runtime cannot auto-accept '${classification}' authority plans; allowed classifications: ${
        [...allowed].join(", ") || "none"
      }`,
    );
  }
  if (
    artifacts.participant.kind !== "service" &&
    artifacts.participant.kind !== "device"
  ) {
    return {
      planId: planned.proposal.proposalId,
      classification,
      participantId: planned.proposal.participantId,
      participantDigest: planned.proposal.participantArtifactDigest,
      participantNeedsDigest: planned.proposal.participantNeedsDigest,
      deploymentId,
    };
  }
  const acceptStartedAt = performance.now();
  if (classification !== "migration") {
    await context.rpc("authDeploymentAuthorityAcceptUpdate", {
      expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
      idempotencyKey: crypto.randomUUID(),
      proposalId: planned.proposal.proposalId,
      reason: null,
    });
  } else {
    await context.rpc("authDeploymentAuthorityAcceptMigration", {
      expectedBaseAuthorityVersion: planned.proposal.baseAuthorityVersion,
      idempotencyKey: crypto.randomUUID(),
      proposalId: planned.proposal.proposalId,
      reason:
        "Approved by TrellisTestRuntime for an isolated integration test.",
    });
  }
  const authority = await context.rpc("authDeploymentAuthorityList", {
    cursor: undefined,
    deploymentId,
    limit: 1,
    state: "accepted",
  });
  const authorityId = authority.entries[0]?.authorityId;
  if (!authorityId) {
    throw new Error(
      `Trellis deployment '${deployment}' has no accepted authority`,
    );
  }
  context.authorityIds.set(deployment, authorityId);
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - acceptStartedAt,
    {
      deployment,
      participantId: String(artifacts.participant.id),
      operation: "approve_contract",
      phase: "accept",
      planClassification: classification,
    },
  );
  await reconcile(context, deployment, "approveContract.reconcile");
  await waitReady(context, deployment, "approveContract.waitReady");
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - totalStartedAt,
    {
      deployment,
      participantId: String(artifacts.participant.id),
      operation: "approve_contract",
      phase: "total",
      planClassification: classification,
    },
  );
  return {
    planId: planned.proposal.proposalId,
    classification,
    participantId: planned.proposal.participantId,
    participantDigest: planned.proposal.participantArtifactDigest,
    participantNeedsDigest: planned.proposal.participantNeedsDigest,
    deploymentId,
  };
}

/** @internal Triggers deployment-authority reconciliation. */
export async function reconcile(
  context: AdminDeploymentContext,
  deployment: string,
  label = "reconcile",
): Promise<void> {
  const startedAt = performance.now();
  const authorityId = context.authorityIds.get(deployment);
  if (!authorityId) {
    throw new Error(
      `Trellis deployment '${deployment}' has no accepted authority`,
    );
  }
  await context.rpc("authDeploymentAuthorityReconcile", {
    authorityId,
    expectedVersion: null,
    idempotencyKey: crypto.randomUUID(),
  });
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - startedAt,
    { operation: label, phase: "reconcile" },
  );
}

/** @internal Waits until materialized deployment authority is current. */
export async function waitReady(
  context: AdminDeploymentContext,
  deployment: string,
  label = "waitReady",
): Promise<void> {
  const startedAt = performance.now();
  const authorityId = context.authorityIds.get(deployment);
  if (!authorityId) {
    throw new Error(
      `Trellis deployment '${deployment}' has no accepted authority`,
    );
  }
  await waitFor(async () => {
    const pollStartedAt = performance.now();
    const result = await context.rpc("authDeploymentAuthorityGet", {
      authorityId,
    });
    const materialized = result.authority.materialization;
    recordTrellisDuration(
      "trellis.admin.workflow.duration",
      performance.now() - pollStartedAt,
      { operation: `${label}.poll`, phase: "wait_ready" },
    );
    if (materialized?.state === "error") {
      throw new Error(
        `Trellis deployment '${deployment}' reconciliation failed${
          materialized.error ? `: ${materialized.error}` : ""
        }`,
      );
    }
    if (
      materialized?.state === "available" &&
      materialized.authorityVersion === result.authority.version &&
      materialized.reconciledAt !== null
    ) {
      return true;
    }
    return false;
  }, { timeoutMs: context.reconciliationMs });
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - startedAt,
    { operation: label, phase: "wait_ready" },
  );
}

/** @internal Provisions a service instance key after approving its contract. */
export async function provisionServiceInstance(
  context: AdminDeploymentContext,
  args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    sessionKeySeed?: string;
  },
): Promise<TrellisTestServiceKey> {
  const startedAt = performance.now();
  const deployment = args.deployment ?? context.defaultDeployment;
  const approved = await approveContract(context, {
    deployment,
    contract: args.contract,
  });
  const identitySeed = args.sessionKeySeed ?? generateSessionSeed();
  const auth = await createAuth({ sessionKeySeed: identitySeed });
  const deploymentId = context.deploymentIds.get(deployment);
  if (!deploymentId) {
    throw new Error(`Trellis deployment '${deployment}' was not created`);
  }
  const provisioned = await context.rpc("authServiceInstancesProvision", {
    deploymentId,
    idempotencyKey: crypto.randomUUID(),
    identityPublicKey: auth.sessionKey,
    instanceId: null,
    participantId: approved.participantId,
  });
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - startedAt,
    { operation: "provision_service", phase: "total" },
  );
  return {
    seed: identitySeed,
    sessionSeed: generateSessionSeed(),
    sessionKey: auth.sessionKey,
    deploymentId,
    instanceId: provisioned.instance.instanceId,
    participantId: approved.participantId,
    participantArtifactDigest: approved.participantDigest,
    participantNeedsDigest: approved.participantNeedsDigest,
  };
}

/** @internal Runs the full service registration sequence used by test services. */
export async function registerService(
  context: AdminDeploymentContext,
  args: {
    deployment?: string;
    contract: TrellisTestContractLike;
    sessionKeySeed?: string;
  },
): Promise<TrellisTestServiceKey> {
  const startedAt = performance.now();
  const deployment = args.deployment ?? context.defaultDeployment;
  const key = await provisionServiceInstance(context, {
    deployment,
    contract: args.contract,
    sessionKeySeed: args.sessionKeySeed,
  });
  await reconcile(
    context,
    deployment,
    "registerService.postProvision.reconcile",
  );
  await waitReady(
    context,
    deployment,
    "registerService.postProvision.waitReady",
  );
  recordTrellisDuration(
    "trellis.admin.workflow.duration",
    performance.now() - startedAt,
    { operation: "register_service", phase: "total" },
  );
  return key;
}

/** @internal Lists deployment authority plans. */
export async function listAuthorityPlans(
  context: AdminDeploymentContext,
  args: {
    deploymentId?: string;
    state?: "pending" | "accepted" | "rejected" | "superseded" | "expired";
    limit?: number;
    cursor?: string;
  },
): Promise<{ entries: unknown[]; nextCursor: string | null }> {
  return await context.rpc("authDeploymentAuthorityPlansList", {
    deploymentId: args.deploymentId,
    state: args.state,
    limit: args.limit ?? 20,
    cursor: args.cursor,
  });
}

/** @internal Rejects a pending deployment authority plan. */
export async function rejectAuthorityPlan(
  context: AdminDeploymentContext,
  args: { planId: string; reason?: string },
): Promise<unknown> {
  return await context.rpc("authDeploymentAuthorityReject", {
    proposalId: args.planId,
    reason: args.reason ?? null,
    idempotencyKey: crypto.randomUUID(),
  });
}

/** @internal Accepts a pending deployment authority update plan. */
export async function acceptAuthorityUpdate(
  context: AdminDeploymentContext,
  args: { planId: string; expectedDesiredVersion?: number },
): Promise<unknown> {
  return await context.rpc("authDeploymentAuthorityAcceptUpdate", {
    proposalId: args.planId,
    expectedBaseAuthorityVersion: args.expectedDesiredVersion ?? null,
    reason: null,
    idempotencyKey: crypto.randomUUID(),
  });
}

/** @internal Accepts a pending deployment authority migration plan. */
export async function acceptAuthorityMigration(
  context: AdminDeploymentContext,
  args: {
    planId: string;
    acknowledgement: string;
    expectedDesiredVersion?: number;
  },
): Promise<unknown> {
  return await context.rpc("authDeploymentAuthorityAcceptMigration", {
    proposalId: args.planId,
    expectedBaseAuthorityVersion: args.expectedDesiredVersion ?? null,
    reason: args.acknowledgement,
    idempotencyKey: crypto.randomUUID(),
  });
}

/** @internal Provisions a service instance without changing deployment authority. */
export async function provisionServiceInstanceOnly(
  context: AdminDeploymentContext,
  args: { deployment?: string; sessionKeySeed?: string },
): Promise<{ seed: string; sessionKey: string }> {
  const deployment = args.deployment ?? context.defaultDeployment;
  const seed = args.sessionKeySeed ?? generateSessionSeed();
  const auth = await createAuth({ sessionKeySeed: seed });
  const deploymentId = context.deploymentIds.get(deployment);
  if (!deploymentId) {
    throw new Error(`Trellis deployment '${deployment}' was not created`);
  }
  await context.rpc("authServiceInstancesProvision", {
    deploymentId,
    idempotencyKey: crypto.randomUUID(),
    identityPublicKey: auth.sessionKey,
    instanceId: null,
    participantId: null,
  });
  return { seed, sessionKey: auth.sessionKey };
}
