import type { IdentityGrantRecord } from "../schemas.ts";

export type EffectiveApproval =
  | { kind: "stored_approval"; answer: "approved" | "denied" }
  | { kind: "deployment_grant"; answer: "approved" }
  | { kind: "none"; answer: "none" };

export function getAppOrigin(redirectTo: string): string | undefined {
  try {
    return new URL(redirectTo).origin;
  } catch {
    return undefined;
  }
}

export function effectiveApproval(args: {
  storedApproval: IdentityGrantRecord | null;
  deploymentGrantApproved?: boolean;
  matchedPolicies: [];
}): EffectiveApproval {
  if (args.storedApproval) {
    return {
      kind: "stored_approval",
      answer: args.storedApproval.answer,
    };
  }
  if (args.deploymentGrantApproved) {
    return { kind: "deployment_grant", answer: "approved" };
  }
  return { kind: "none", answer: "none" };
}

export function missingCapabilities(args: {
  requiredCapabilities: string[];
  effectiveCapabilities: string[];
}): string[] {
  return args.requiredCapabilities.filter((capability) =>
    !capabilitySatisfied(args.effectiveCapabilities, capability)
  );
}

function capabilitySatisfied(
  effectiveCapabilities: string[],
  required: string,
) {
  if (effectiveCapabilities.includes(required)) return true;
  return required === "trellis.auth::device.review" &&
    effectiveCapabilities.some((capability) =>
      capability.startsWith("trellis.auth::device.review.")
    );
}

export function userDelegationAllowed(args: {
  active: boolean;
  explicitCapabilities: string[];
  delegatedCapabilities: string[];
  storedApproval: IdentityGrantRecord | null;
  matchedPolicies: [];
}): boolean {
  if (!args.active) return false;
  const resolvedApproval = effectiveApproval({
    storedApproval: args.storedApproval,
    matchedPolicies: args.matchedPolicies,
  });
  return resolvedApproval.answer === "approved" &&
    args.delegatedCapabilities.every((capability) =>
      args.explicitCapabilities.includes(capability)
    );
}
