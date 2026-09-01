import type { ClientAuthContinuation } from "@qlever-llc/trellis";
import { fetchPortalFlowState } from "@qlever-llc/trellis/auth/browser";

import { ADMIN_USERNAME } from "./methods.ts";
import { recordTrellisDuration } from "./metrics.ts";
import { postJson } from "./transport.ts";

export function flowIdFromUrl(url: string): string {
  const flowId = new URL(url).searchParams.get("flowId");
  if (!flowId) throw new Error(`Trellis auth URL is missing flowId: ${url}`);
  return flowId;
}

export function adminAccountTokenFromUrl(url: string): string {
  const token = new URL(url).searchParams.get("adminAccountToken");
  if (!token) {
    throw new Error(
      `Trellis administrator URL is missing adminAccountToken: ${url}`,
    );
  }
  return token;
}

export async function performLocalLogin(args: {
  trellisUrl: string;
  flowId: string;
  password: string;
}): Promise<void> {
  const startedAt = performance.now();
  try {
    await postJson(`${args.trellisUrl}/auth/login/local`, {
      flowId: args.flowId,
      username: ADMIN_USERNAME,
      password: args.password,
    });
  } finally {
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - startedAt,
      { phase: "local_login", authFlow: "local" },
    );
  }
}

export async function approveLocalFlowIfNeeded(args: {
  trellisUrl: string;
  flowId: string;
}): Promise<void> {
  const startedAt = performance.now();
  const initialFetchStartedAt = performance.now();
  const state = await fetchPortalFlowState(
    { authUrl: args.trellisUrl },
    args.flowId,
  );
  recordTrellisDuration(
    "trellis.auth.flow.duration",
    performance.now() - initialFetchStartedAt,
    { phase: "approval_fetch" },
  );
  if (state.status === "redirect") {
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - startedAt,
      { phase: "total" },
    );
    return;
  }
  if (state.status === "approval_required") {
    const approvalStartedAt = performance.now();
    await postJson(
      `${args.trellisUrl}/auth/flow/${
        encodeURIComponent(args.flowId)
      }/approval`,
      {
        approved: true,
        consentViewDigest: state.consentViewDigest,
        selectedOptionalBundles: [],
      },
    );
    const approved = await fetchPortalFlowState(
      { authUrl: args.trellisUrl },
      args.flowId,
    );
    recordTrellisDuration(
      "trellis.auth.flow.duration",
      performance.now() - approvalStartedAt,
      { phase: "approval_submit" },
    );
    if (approved.status === "redirect") {
      recordTrellisDuration(
        "trellis.auth.flow.duration",
        performance.now() - startedAt,
        { phase: "total" },
      );
      return;
    }
    throw new Error(
      `Trellis auth approval did not complete; portal state is '${approved.status}'`,
    );
  }
  throw new Error(
    `Trellis local login did not reach approval; portal state is '${state.status}'`,
  );
}

export async function completeLocalAuthFlow(args: {
  trellisUrl: string;
  loginUrl: string;
  password: string;
}): Promise<ClientAuthContinuation> {
  const startedAt = performance.now();
  const flowId = flowIdFromUrl(args.loginUrl);
  await performLocalLogin({
    trellisUrl: args.trellisUrl,
    flowId,
    password: args.password,
  });
  await approveLocalFlowIfNeeded({
    trellisUrl: args.trellisUrl,
    flowId,
  });
  recordTrellisDuration(
    "trellis.auth.flow.duration",
    performance.now() - startedAt,
    { phase: "total" },
  );
  return { status: "bound", flowId };
}
