import { type PortalFlowState, PortalFlowStateSchema } from "../protocol.ts";
import type { ApprovalDecision } from "../schemas.ts";
import type { AuthConfig } from "./login.ts";
import { Value } from "typebox/value";

export type { PortalFlowState } from "../protocol.ts";
export type { ApprovalDecision } from "../schemas.ts";

function authBaseUrl(config: AuthConfig): string {
  return config.authUrl.replace(/\/$/, "");
}

type BrowserFlowWire = {
  flowId: string;
  state: string;
  providers: string[];
  registrationEnabled: boolean;
  federatedRegistrationEnabled: boolean;
  consentView: {
    participant: {
      id: string;
      digest: string;
      displayName: string;
      description: string;
    };
    required: { permissions: unknown[]; capabilities: string[] };
    optionalBundles?: {
      id: string;
      api: string;
      permissions: Record<string, unknown>[];
    }[];
  };
  consentViewDigest: string;
  user?: {
    origin: string;
    id: string;
    name?: string;
    email?: string;
    image?: string;
  } | null;
  redirectTarget?: string | null;
};

function approval(wire: BrowserFlowWire) {
  const capabilities: Record<
    string,
    { displayName: string; description: string }
  > = {};
  for (const name of wire.consentView.required.capabilities) {
    capabilities[`capability:${name}`] = {
      displayName: name,
      description: "Required by this application.",
    };
  }
  wire.consentView.required.permissions.forEach((permission, index) => {
    capabilities[`permission:${index}`] = {
      displayName: "Required permission",
      description: JSON.stringify(permission),
    };
  });
  return {
    contractId: wire.consentView.participant.id,
    contractDigest: wire.consentView.participant.digest,
    displayName: wire.consentView.participant.displayName,
    description: wire.consentView.participant.description,
    capabilities,
  };
}

function portalState(wire: BrowserFlowWire): PortalFlowState {
  const evidence = approval(wire);
  let state: unknown;
  if (wire.state === "choose_provider") {
    const federatedProviders = wire.providers.filter((id) => id !== "local")
      .map(
        (id) => ({ id, displayName: id }),
      );
    state = {
      status: "choose_provider",
      flowId: wire.flowId,
      providers: wire.providers.map((id) => ({ id, displayName: id })),
      app: {
        contractId: evidence.contractId,
        contractDigest: evidence.contractDigest,
        displayName: evidence.displayName,
        description: evidence.description,
      },
      registration: {
        localIdentity: { available: wire.registrationEnabled },
        federatedIdentity: {
          available: wire.federatedRegistrationEnabled,
          providers: federatedProviders,
        },
      },
    };
  } else if (
    wire.state === "authenticated" || wire.state === "approval_required"
  ) {
    if (!wire.user) {
      throw new Error("Authenticated portal flow has no user profile");
    }
    state = {
      status: "approval_required",
      flowId: wire.flowId,
      consentViewDigest: wire.consentViewDigest,
      optionalBundles: (wire.consentView.optionalBundles ?? []).map((
        bundle,
      ) => ({
        id: bundle.id,
        apiId: bundle.api,
        permissions: bundle.permissions,
      })),
      user: wire.user,
      approval: evidence,
    };
  } else if (wire.state === "approval_denied") {
    state = {
      status: "approval_denied",
      flowId: wire.flowId,
      approval: evidence,
      ...(wire.redirectTarget ? { returnLocation: wire.redirectTarget } : {}),
    };
  } else if (wire.state === "approved" || wire.state === "consumed") {
    if (!wire.redirectTarget) {
      throw new Error("Completed portal flow has no redirect target");
    }
    state = { status: "redirect", location: wire.redirectTarget };
  } else {
    state = {
      status: "expired",
      ...(wire.redirectTarget ? { returnLocation: wire.redirectTarget } : {}),
    };
  }
  return Value.Parse(PortalFlowStateSchema, state) as PortalFlowState;
}

async function fetchBrowserFlowWire(
  config: AuthConfig,
  flowId: string,
): Promise<BrowserFlowWire> {
  const response = await fetch(
    `${authBaseUrl(config)}/auth/flow/${encodeURIComponent(flowId)}`,
  );
  if (!response.ok) {
    throw new Error(`Failed to load portal flow (${response.status})`);
  }
  return await response.json() as BrowserFlowWire;
}

export function portalFlowIdFromUrl(url: URL): string | null {
  return url.searchParams.get("flowId");
}

export async function fetchPortalFlowState(
  config: AuthConfig,
  flowId: string,
): Promise<PortalFlowState> {
  return portalState(await fetchBrowserFlowWire(config, flowId));
}

export function portalProviderLoginUrl(
  config: AuthConfig,
  providerId: string,
  flowId: string,
): string {
  const base = `${authBaseUrl(config)}/auth/login/${
    encodeURIComponent(providerId)
  }`;
  return `${base}?flowId=${encodeURIComponent(flowId)}`;
}

export async function submitPortalApproval(
  config: AuthConfig,
  flowId: string,
  decision: ApprovalDecision,
  selectedOptionalBundles: readonly string[] = [],
): Promise<PortalFlowState> {
  const flow = await fetchBrowserFlowWire(config, flowId);
  const response = await fetch(
    `${authBaseUrl(config)}/auth/flow/${encodeURIComponent(flowId)}/approval`,
    {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify({
        approved: decision === "approved",
        consentViewDigest: flow.consentViewDigest,
        selectedOptionalBundles,
        idempotencyKey: crypto.randomUUID(),
      }),
    },
  );

  if (!response.ok) {
    throw new Error(`Approval request failed (${response.status})`);
  }

  return await fetchPortalFlowState(config, flowId);
}

export function portalRedirectLocation(
  state: PortalFlowState | null,
): string | null {
  if (state?.status === "redirect") return state.location;
  if (state?.status === "approval_denied") return state.returnLocation ?? null;
  if (state?.status === "expired") return state.returnLocation ?? null;
  return null;
}
