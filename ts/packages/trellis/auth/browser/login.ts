import { Value } from "typebox/value";
import {
  type NativeProtocolContract,
  nativeProtocolPresentation,
} from "../../contract_support/protocol_artifacts.ts";
import {
  type AuthStartRequest,
  AuthStartRequestSchema,
  type AuthStartResponse,
  AuthStartResponseSchema,
  AuthStartWireResponseSchema,
  type BindResponse,
  BindResponseSchema,
  type BindSuccessResponse,
} from "../schemas.ts";
import {
  sessionNkeyFromPublicKey,
  sessionProofRequestDigest,
  signSessionProof,
} from "../session_proof.ts";
import { bindFlowSig, getPublicSessionKey } from "./session.ts";
import type { SessionKeyHandle } from "./session.ts";

export type AuthConfig = {
  authUrl: string;
};

export type {
  AuthStartFlowResponse,
  AuthStartRequest,
  AuthStartResponse,
  BindResponse,
  BindSuccessResponse,
  ContractApproval,
  SentinelCreds,
} from "../schemas.ts";

type BuildLoginUrlFlatArgs = {
  authUrl: string;
  redirectTo: string;
  handle: SessionKeyHandle;
  contract: NativeProtocolContract;
};

type StartAuthRequestArgs = {
  authUrl: string;
  redirectTo: string;
  handle: SessionKeyHandle;
  contract: NativeProtocolContract;
};

export async function buildLoginUrl(
  args: BuildLoginUrlFlatArgs,
): Promise<string> {
  const response = await startAuthRequest({
    authUrl: args.authUrl,
    redirectTo: args.redirectTo,
    handle: args.handle,
    contract: args.contract,
  });
  if (response.status !== "flow_started") {
    throw new Error("Auth request completed without starting a browser flow");
  }
  return response.loginUrl;
}

export async function startAuthRequest(
  args: StartAuthRequestArgs,
): Promise<AuthStartResponse> {
  const presentation = nativeProtocolPresentation(args.contract);
  const participantDigest = args.contract.CONTRACT_DIGEST;
  const participantEvidence = {
    participantId: args.contract.CONTRACT_ID,
    participantArtifactDigest: participantDigest,
    participantArtifact: presentation.participant,
    referencedApiArtifacts: [presentation.api, ...presentation.referencedApis],
  };
  const requestId = crypto.randomUUID();
  const issuedAt = Date.now();
  const sessionPublicKey = getPublicSessionKey(args.handle);
  const sessionNkey = sessionNkeyFromPublicKey(sessionPublicKey);
  const request = {
    requestId,
    issuedAt,
    sessionPublicKey,
    sessionNkey,
    ...participantEvidence,
    redirectTarget: args.redirectTo,
    proof: { format: "trellis.session-proof.v1", signature: "" },
  } satisfies AuthStartRequest;
  const requestDigest = await sessionProofRequestDigest(request);
  request.proof = await signSessionProof(
    {
      purpose: "userAuthRequest",
      requestId,
      issuedAt,
      sessionPublicKey,
      sessionNkey,
      participantId: participantEvidence.participantId,
      participantDigest,
      redirectTarget: args.redirectTo,
      requestDigest,
    },
    args.handle.privateKey,
    sessionPublicKey,
  );
  Value.Parse(AuthStartRequestSchema, request);

  const response = await fetch(`${args.authUrl}/auth/requests`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });
  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Auth request failed: ${response.status} ${text}`);
  }

  const wire = Value.Parse(AuthStartWireResponseSchema, await response.json());
  return Value.Parse(AuthStartResponseSchema, {
    status: "flow_started",
    flowId: wire.flowId,
    loginUrl: wire.portalUrl,
  }) as AuthStartResponse;
}

export function isBindSuccessResponse(
  response: BindResponse,
): response is BindSuccessResponse {
  return response.status === "bound";
}

export async function bindFlow(
  config: AuthConfig,
  handle: SessionKeyHandle,
  flowId: string,
): Promise<BindResponse> {
  const sessionKey = getPublicSessionKey(handle);
  const sig = await bindFlowSig(handle, flowId);

  const response = await fetch(
    `${config.authUrl}/auth/flow/${encodeURIComponent(flowId)}/bind`,
    {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({ sessionKey, sig }),
    },
  );

  if (!response.ok) {
    const text = await response.text();
    throw new Error(`Bind failed: ${response.status} ${text}`);
  }

  const payload = await response.json();
  if (payload && typeof payload === "object" && payload.status === "expired") {
    throw new Error("Bind failed: expired");
  }

  return Value.Parse(BindResponseSchema, payload) as BindResponse;
}
