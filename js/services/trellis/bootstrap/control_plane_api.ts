import { getContractRuntime } from "@qlever-llc/trellis/internal/contract-runtime";
import trellisAuth from "../contracts/trellis_auth.ts";
import trellisCore from "../contracts/trellis_core.ts";
import trellisState from "../contracts/trellis_state.ts";

const trellisAuthApi = getContractRuntime(trellisAuth);
const trellisCoreApi = getContractRuntime(trellisCore);
const trellisStateApi = getContractRuntime(trellisState);

const CONTROL_PLANE_APIS = [
  trellisCoreApi,
  trellisAuthApi,
  trellisStateApi,
] as const;

const OWNED_API_KINDS = ["rpc", "operations", "events", "subjects"] as const;
const TRELLIS_API_KINDS = ["rpc", "operations", "events", "subjects"] as const;

function assertNoOverlap(
  kind: string,
  left: Record<string, unknown>,
  right: Record<string, unknown>,
) {
  for (const key of Object.keys(left)) {
    if (key in right) {
      throw new Error(
        `Duplicate ${kind} key '${key}' in Trellis control-plane API`,
      );
    }
  }
}

function subjectOf(value: unknown): string | undefined {
  return typeof value === "object" && value !== null && "subject" in value &&
      typeof value.subject === "string"
    ? value.subject
    : undefined;
}

function assertNoConflictingOverlap(
  kind: string,
  left: Record<string, unknown>,
  right: Record<string, unknown>,
) {
  for (const [key, leftValue] of Object.entries(left)) {
    const rightValue = right[key];
    if (rightValue === undefined) {
      continue;
    }

    const leftSubject = subjectOf(leftValue);
    const rightSubject = subjectOf(rightValue);
    if (leftSubject !== undefined && leftSubject === rightSubject) {
      continue;
    }

    throw new Error(
      `Duplicate ${kind} key '${key}' in Trellis control-plane API`,
    );
  }
}

function assertComposableApi() {
  for (
    let leftIndex = 0;
    leftIndex < CONTROL_PLANE_APIS.length;
    leftIndex += 1
  ) {
    for (
      let rightIndex = leftIndex + 1;
      rightIndex < CONTROL_PLANE_APIS.length;
      rightIndex += 1
    ) {
      const left = CONTROL_PLANE_APIS[leftIndex];
      const right = CONTROL_PLANE_APIS[rightIndex];
      for (const kind of OWNED_API_KINDS) {
        assertNoOverlap(
          kind === "operations" ? "operation" : kind.slice(0, -1),
          left.ownedApi[kind],
          right.ownedApi[kind],
        );
      }
      for (const kind of TRELLIS_API_KINDS) {
        assertNoConflictingOverlap(
          kind === "operations" ? "operation" : kind.slice(0, -1),
          left.api[kind] ?? {},
          right.api[kind] ?? {},
        );
      }
    }
  }
}

assertComposableApi();

export const trellisControlPlaneApi = {
  owned: {
    rpc: {
      ...trellisCoreApi.ownedApi.rpc,
      ...trellisAuthApi.ownedApi.rpc,
      ...trellisStateApi.ownedApi.rpc,
    },
    operations: {
      ...trellisCoreApi.ownedApi.operations,
      ...trellisAuthApi.ownedApi.operations,
      ...trellisStateApi.ownedApi.operations,
    },
    events: {
      ...trellisCoreApi.ownedApi.events,
      ...trellisAuthApi.ownedApi.events,
      ...trellisStateApi.ownedApi.events,
    },
    subjects: {
      ...trellisCoreApi.ownedApi.subjects,
      ...trellisAuthApi.ownedApi.subjects,
      ...trellisStateApi.ownedApi.subjects,
    },
  },
  trellis: {
    rpc: {
      ...trellisCoreApi.api.rpc,
      ...trellisAuthApi.api.rpc,
      ...trellisStateApi.api.rpc,
    },
    operations: {
      ...trellisCoreApi.api.operations,
      ...trellisAuthApi.api.operations,
      ...trellisStateApi.api.operations,
    },
    events: {
      ...trellisCoreApi.api.events,
      ...trellisAuthApi.api.events,
      ...trellisStateApi.api.events,
    },
    subjects: {
      ...trellisCoreApi.api.subjects,
      ...trellisAuthApi.api.subjects,
      ...trellisStateApi.api.subjects,
    },
  },
} as const;
