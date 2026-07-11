import type { TrellisContractV1 } from "@qlever-llc/trellis/contracts";

import { analyzeActiveContractCompatibility } from "../catalog/uses.ts";
import { computeAuthorityNeedsDelta } from "./authority_needs_decision.ts";
import type {
  AuthorityNeedSet,
  AuthorityNeedSetResource,
  DeploymentAuthorityPlanBreakingChange,
} from "./schemas.ts";

export type DeploymentAuthorityPlanClassification = "update" | "migration";

export type DeploymentAuthorityPlanClassificationResult = {
  classification: DeploymentAuthorityPlanClassification;
  approvalRequired: boolean;
  desiredChange: AuthorityNeedSet;
};

export type ContractCompatibilityFailure = {
  message: string;
  latestAcceptedContractDigest: string;
  breakingChanges: DeploymentAuthorityPlanBreakingChange[];
};

type ContractLookup = {
  getContract(
    digest: string,
    options?: { includeInactive?: boolean },
  ): Promise<TrellisContractV1 | undefined>;
};

/** Returns the implementation-offer lineage for one service deployment contract. */
export function serviceOfferLineageKey(
  deploymentId: string,
  contractId: string,
): string {
  return JSON.stringify(["service", deploymentId, contractId]);
}

function resourceKey(
  resource: Pick<AuthorityNeedSetResource, "kind" | "alias">,
) {
  return `${resource.kind}:${resource.alias}`;
}

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function unknownEquals(left: unknown, right: unknown): boolean {
  if (Object.is(left, right)) return true;
  if (Array.isArray(left) || Array.isArray(right)) {
    if (!Array.isArray(left) || !Array.isArray(right)) return false;
    return left.length === right.length &&
      left.every((entry, index) => unknownEquals(entry, right[index]));
  }
  if (!isRecord(left) || !isRecord(right)) return false;
  const leftKeys = Object.keys(left).sort();
  const rightKeys = Object.keys(right).sort();
  return leftKeys.length === rightKeys.length &&
    leftKeys.every((key, index) =>
      key === rightKeys[index] && unknownEquals(left[key], right[key])
    );
}

function requestedResourceMap(requested: AuthorityNeedSet) {
  const resources = new Map<string, AuthorityNeedSetResource>();
  for (const resource of requested.resources) {
    resources.set(resourceKey(resource), resource);
  }
  return resources;
}

function numberValue(
  definition: unknown,
  key: string,
): number | undefined {
  if (!isRecord(definition)) return undefined;
  const value = definition[key];
  return typeof value === "number" ? value : undefined;
}

function isLimitRelaxedOrSame(
  current: number | undefined,
  requested: number | undefined,
): boolean {
  if (current === undefined) return requested === undefined;
  if (requested === undefined) return true;
  return requested >= current;
}

function isTtlRelaxedOrSame(
  current: number | undefined,
  requested: number | undefined,
): boolean {
  const currentTtl = current ?? 0;
  const requestedTtl = requested ?? 0;
  if (currentTtl === 0) return requestedTtl === 0;
  if (requestedTtl === 0) return true;
  return requestedTtl >= currentTtl;
}

function isBackoffRelaxedOrSame(current: unknown, requested: unknown): boolean {
  if (current === undefined) return true;
  if (requested === undefined) return false;
  if (!Array.isArray(current) || !Array.isArray(requested)) {
    return unknownEquals(current, requested);
  }
  return current.length <= requested.length &&
    current.every((value, index) => {
      const requestedValue = requested[index];
      return typeof value === "number" && typeof requestedValue === "number" &&
        requestedValue >= value;
    });
}

function hasChangedUnsafeDefinitionFields(
  current: Record<string, unknown>,
  requested: Record<string, unknown>,
  safeKeys: readonly string[],
): boolean {
  const safe = new Set(safeKeys);
  for (
    const key of new Set([...Object.keys(current), ...Object.keys(requested)])
  ) {
    if (!safe.has(key) && !unknownEquals(current[key], requested[key])) {
      return true;
    }
  }
  return false;
}

function resourceDefinitionChangeRequiresMigration(
  current: AuthorityNeedSetResource,
  requested: AuthorityNeedSetResource,
): boolean {
  if (current.kind !== requested.kind) return true;
  const currentDefinition = current.definition;
  const requestedDefinition = requested.definition;
  if (unknownEquals(currentDefinition, requestedDefinition)) return false;
  if (!isRecord(currentDefinition) || !isRecord(requestedDefinition)) {
    return true;
  }

  if (current.kind === "kv" || current.kind === "store") {
    if (
      hasChangedUnsafeDefinitionFields(currentDefinition, requestedDefinition, [
        "type",
        "history",
        "ttlMs",
        "maxValueBytes",
        "maxObjectBytes",
        "maxTotalBytes",
        "schema",
        "purpose",
        "docs",
        "description",
        "displayName",
      ])
    ) return true;
    return !(
      unknownEquals(currentDefinition.schema, requestedDefinition.schema) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "history"),
        numberValue(requestedDefinition, "history"),
      ) &&
      isTtlRelaxedOrSame(
        numberValue(currentDefinition, "ttlMs"),
        numberValue(requestedDefinition, "ttlMs"),
      ) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "maxValueBytes"),
        numberValue(requestedDefinition, "maxValueBytes"),
      ) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "maxObjectBytes"),
        numberValue(requestedDefinition, "maxObjectBytes"),
      ) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "maxTotalBytes"),
        numberValue(requestedDefinition, "maxTotalBytes"),
      )
    );
  }

  if (current.kind === "jobs" || current.kind === "event-consumer") {
    if (
      hasChangedUnsafeDefinitionFields(currentDefinition, requestedDefinition, [
        "type",
        "payload",
        "result",
        "filterSubjects",
        "replayPolicy",
        "ordering",
        "maxDeliver",
        "ackWaitMs",
        "backoffMs",
        "purpose",
        "docs",
        "description",
        "displayName",
      ])
    ) return true;
    return !(
      unknownEquals(currentDefinition.payload, requestedDefinition.payload) &&
      unknownEquals(currentDefinition.result, requestedDefinition.result) &&
      unknownEquals(
        currentDefinition.filterSubjects,
        requestedDefinition.filterSubjects,
      ) &&
      unknownEquals(
        currentDefinition.replayPolicy,
        requestedDefinition.replayPolicy,
      ) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "maxDeliver"),
        numberValue(requestedDefinition, "maxDeliver"),
      ) &&
      isLimitRelaxedOrSame(
        numberValue(currentDefinition, "ackWaitMs"),
        numberValue(requestedDefinition, "ackWaitMs"),
      ) &&
      isBackoffRelaxedOrSame(
        currentDefinition.backoffMs,
        requestedDefinition.backoffMs,
      )
    );
  }

  return true;
}

function hasResourceRemoval(
  current: AuthorityNeedSet,
  requested: AuthorityNeedSet,
): boolean {
  const requestedResources = requestedResourceMap(requested);
  return current.resources.some((resource) =>
    !requestedResources.has(resourceKey(resource))
  );
}

function changedResourceDefinitions(
  current: AuthorityNeedSet,
  requested: AuthorityNeedSet,
): Array<{ resource: AuthorityNeedSetResource; requiresMigration: boolean }> {
  const requestedResources = requestedResourceMap(requested);
  return current.resources.flatMap((resource) => {
    const requestedResource = requestedResources.get(resourceKey(resource));
    if (
      requestedResource === undefined ||
      unknownEquals(resource.definition, requestedResource.definition)
    ) {
      return [];
    }
    return [{
      resource: requestedResource,
      requiresMigration: resourceDefinitionChangeRequiresMigration(
        resource,
        requestedResource,
      ),
    }];
  });
}

function hasNewResource(
  current: AuthorityNeedSet,
  desiredChange: AuthorityNeedSet,
): boolean {
  const currentResources = requestedResourceMap(current);
  return desiredChange.resources.some((resource) =>
    !currentResources.has(resourceKey(resource))
  );
}

/** Classifies a requested deployment authority desired-state change. */
export function classifyDeploymentAuthorityPlan(
  current: AuthorityNeedSet,
  requested: AuthorityNeedSet,
): DeploymentAuthorityPlanClassificationResult {
  const desiredChange = computeAuthorityNeedsDelta(current, requested);
  const definitionChanges = changedResourceDefinitions(current, requested);
  const classification = hasResourceRemoval(current, requested) ||
      definitionChanges.some((change) => change.requiresMigration)
    ? "migration"
    : "update";

  return {
    classification,
    approvalRequired: classification === "migration" ||
      desiredChange.contracts.length > 0 ||
      desiredChange.capabilities.length > 0 ||
      hasNewResource(current, desiredChange),
    desiredChange: {
      ...desiredChange,
      resources: [
        ...desiredChange.resources,
        ...definitionChanges.map((change) => change.resource),
      ],
    },
  };
}

/** Checks whether a same-contract digest replacement is active-compatible. */
export async function evaluateSameContractCompatibility(input: {
  contracts: ContractLookup;
  latestAcceptedContractDigest: string | undefined;
  presentedDigest: string;
  presentedContract: TrellisContractV1;
}): Promise<ContractCompatibilityFailure | null> {
  const currentDigest = input.latestAcceptedContractDigest;
  if (!currentDigest || currentDigest === input.presentedDigest) return null;

  const currentContract = await input.contracts.getContract(currentDigest, {
    includeInactive: true,
  });
  if (!currentContract) {
    return {
      message: `previous service contract digest '${currentDigest}' is unknown`,
      latestAcceptedContractDigest: currentDigest,
      breakingChanges: [{
        kind: "digest-incompatible",
        target: {
          kind: "digest",
          contractId: input.presentedContract.id,
          contractDigest: currentDigest,
        },
        reason:
          `Previous service contract digest '${currentDigest}' is unknown.`,
      }],
    };
  }

  const analysis = analyzeActiveContractCompatibility([
    { digest: currentDigest, contract: currentContract },
    { digest: input.presentedDigest, contract: input.presentedContract },
  ]);
  if (analysis.compatible) return null;

  const message = analysis.message ??
    "Active compatible digests are incompatible";
  return {
    message,
    latestAcceptedContractDigest: currentDigest,
    breakingChanges: analysis.breakingChanges.length > 0
      ? analysis.breakingChanges
      : [{
        kind: "digest-incompatible",
        target: {
          kind: "digest",
          contractId: input.presentedContract.id,
          contractDigest: input.presentedDigest,
        },
        reason: message,
      }],
  };
}
