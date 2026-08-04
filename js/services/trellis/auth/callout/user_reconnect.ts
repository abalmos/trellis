import type { CapabilityGroupLoader } from "../capability_groups.ts";
import { resolveCapabilities } from "../capability_groups.ts";
import {
  delegatedCapabilitiesForApprovalPlan,
  delegatedPublishSubjectsForApprovalPlan,
  delegatedSubscribeSubjectsForApprovalPlan,
  type UserContractApprovalPlan,
} from "../approval/plan.ts";
import type {
  AuthorityNeedSet,
  UserProjectionEntry,
  UserSession,
} from "../schemas.ts";

export type UserReconnectFailureReason =
  | "approval_required"
  | "contract_changed"
  | "insufficient_permissions"
  | "user_inactive"
  | "user_not_found";

export type ResolveUserReconnectResult =
  | { ok: true; session: UserSession }
  | { ok: false; reason: UserReconnectFailureReason };

export async function resolveUserReconnectSession(args: {
  session: UserSession;
  presentedContractDigest: string;
  loadUserProjection: (
    userId: string,
  ) => Promise<UserProjectionEntry | null>;
  capabilityGroupStorage?: CapabilityGroupLoader;
  approvalPlan?: UserContractApprovalPlan;
}): Promise<ResolveUserReconnectResult> {
  if (args.presentedContractDigest !== args.session.contractDigest) {
    return { ok: false, reason: "contract_changed" };
  }

  const projection = await args.loadUserProjection(args.session.userId);
  if (!projection) {
    return { ok: false, reason: "user_not_found" };
  }
  if (!projection.active) {
    return { ok: false, reason: "user_inactive" };
  }

  const resolvedCapabilities = await resolveCapabilities(
    projection,
    args.capabilityGroupStorage,
  );
  const requiredCapabilities = requiredCapabilityKeys(
    args.session.identityAuthorityNeeds,
  ) ?? args.session.delegatedCapabilities;
  if (
    args.session.approvalSource !== "deployment_grant" &&
    !requiredCapabilities.every((capability) =>
      capabilitySatisfied(resolvedCapabilities, capability)
    )
  ) {
    return { ok: false, reason: "insufficient_permissions" };
  }

  return {
    ok: true,
    session: {
      ...args.session,
      approvalSource: args.session.approvalSource ?? "stored_approval",
      ...narrowDelegatedSessionAuthority(
        args.session,
        resolvedCapabilities,
        args.approvalPlan,
      ),
    },
  };
}

function capabilitySatisfied(
  effectiveCapabilities: readonly string[],
  required: string,
): boolean {
  if (effectiveCapabilities.includes(required)) return true;
  return required === "trellis.auth::device.review" &&
    effectiveCapabilities.some((capability) =>
      capability.startsWith("trellis.auth::device.review.")
    );
}

function isAuthorityNeedSet(value: unknown): value is AuthorityNeedSet {
  return !!value && typeof value === "object" &&
    "contracts" in value && Array.isArray(value.contracts) &&
    "surfaces" in value && Array.isArray(value.surfaces) &&
    "capabilities" in value && Array.isArray(value.capabilities) &&
    "resources" in value && Array.isArray(value.resources);
}

function requiredCapabilityKeys(value: unknown): string[] | undefined {
  if (!isAuthorityNeedSet(value)) return undefined;
  const keys: string[] = [];
  for (const need of value.capabilities) {
    if (typeof need === "string") {
      keys.push(need);
      continue;
    }
    if (
      need && typeof need === "object" && "capability" in need &&
      typeof need.capability === "string"
    ) {
      keys.push(need.capability);
    }
  }
  return keys;
}

function intersectSorted(left: string[], right: string[]): string[] {
  const rightSet = new Set(right);
  return left.filter((value) => rightSet.has(value)).sort((a, b) =>
    a.localeCompare(b)
  );
}

function narrowDelegatedSessionAuthority(
  session: UserSession,
  resolvedCapabilities: string[],
  approvalPlan: UserContractApprovalPlan | undefined,
): Pick<
  UserSession,
  | "delegatedCapabilities"
  | "delegatedPublishSubjects"
  | "delegatedSubscribeSubjects"
> {
  if (session.approvalSource === "deployment_grant") {
    return {
      delegatedCapabilities: session.delegatedCapabilities,
      delegatedPublishSubjects: session.delegatedPublishSubjects,
      delegatedSubscribeSubjects: session.delegatedSubscribeSubjects,
    };
  }

  if (!approvalPlan) {
    return {
      delegatedCapabilities: intersectSorted(
        session.delegatedCapabilities,
        resolvedCapabilities,
      ),
      delegatedPublishSubjects: session.delegatedPublishSubjects,
      delegatedSubscribeSubjects: session.delegatedSubscribeSubjects,
    };
  }

  return {
    delegatedCapabilities: intersectSorted(
      session.delegatedCapabilities,
      delegatedCapabilitiesForApprovalPlan(approvalPlan, resolvedCapabilities),
    ),
    delegatedPublishSubjects: intersectSorted(
      session.delegatedPublishSubjects,
      delegatedPublishSubjectsForApprovalPlan(
        approvalPlan,
        resolvedCapabilities,
      ),
    ),
    delegatedSubscribeSubjects: intersectSorted(
      session.delegatedSubscribeSubjects,
      delegatedSubscribeSubjectsForApprovalPlan(
        approvalPlan,
        resolvedCapabilities,
      ),
    ),
  };
}
