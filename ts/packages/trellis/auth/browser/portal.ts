import { type PortalFlowState, PortalFlowStateSchema } from "../protocol.ts";
import type { StaticDecode } from "typebox";
import { Type } from "typebox";
import { Value } from "typebox/value";

export type { PortalFlowState } from "../protocol.ts";
export type ApprovalDecision = "approved" | "denied";
export type AuthConfig = { authUrl: string };

function authBaseUrl(config: AuthConfig): string {
  return config.authUrl.replace(/\/$/, "");
}

const BrowserFlowWireSchema = Type.Object({
  flowId: Type.String({ minLength: 1 }),
  expiresAt: Type.Integer(),
  state: Type.Union([
    Type.Literal("choose_provider"),
    Type.Literal("authenticated"),
    Type.Literal("approval_required"),
    Type.Literal("approval_denied"),
    Type.Literal("approved"),
    Type.Literal("consumed"),
    Type.Literal("expired"),
  ]),
  providers: Type.Array(Type.String({ minLength: 1 })),
  registrationEnabled: Type.Boolean(),
  federatedRegistrationEnabled: Type.Boolean(),
  consentView: Type.Object({
    participant: Type.Object({
      id: Type.String({ minLength: 1 }),
      digest: Type.String({ minLength: 1 }),
      displayName: Type.String({ minLength: 1 }),
      description: Type.String(),
    }, { additionalProperties: false }),
    required: Type.Object({
      permissions: Type.Array(Type.Unknown()),
      capabilities: Type.Array(Type.String({ minLength: 1 })),
    }, { additionalProperties: false }),
    optionalBundles: Type.Optional(Type.Array(Type.Object({
      id: Type.String({ minLength: 1 }),
      apiId: Type.String({ minLength: 1 }),
      permissions: Type.Array(Type.Record(Type.String(), Type.Unknown())),
    }, { additionalProperties: false }))),
  }, { additionalProperties: false }),
  consentViewDigest: Type.String({ minLength: 1 }),
  user: Type.Optional(Type.Union([
    Type.Null(),
    Type.Object({
      origin: Type.String({ minLength: 1 }),
      id: Type.String({ minLength: 1 }),
      name: Type.Optional(Type.String({ minLength: 1 })),
      email: Type.Optional(Type.String({ minLength: 1 })),
      image: Type.Optional(Type.String({ minLength: 1 })),
    }, { additionalProperties: false }),
  ])),
  redirectTarget: Type.Optional(Type.Union([
    Type.Null(),
    Type.String({ minLength: 1 }),
  ])),
}, { additionalProperties: false });
type BrowserFlowWire = StaticDecode<typeof BrowserFlowWireSchema>;

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
        apiId: bundle.apiId,
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
    const location = new URL(wire.redirectTarget);
    location.searchParams.set("flowId", wire.flowId);
    state = { status: "redirect", location: location.toString() };
  } else if (wire.state === "expired") {
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
  return Value.Parse(BrowserFlowWireSchema, await response.json());
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
