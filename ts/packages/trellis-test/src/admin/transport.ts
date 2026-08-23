import { getBuiltinRpcError } from "@qlever-llc/trellis/errors";

import type { TrellisTestAdminRpcMethod } from "./methods.ts";

export function isRecord(value: unknown): value is Record<string, unknown> {
  return value !== null && typeof value === "object" && !Array.isArray(value);
}

export async function postJson(
  url: string,
  body: Record<string, unknown>,
): Promise<unknown> {
  const response = await fetch(url, {
    method: "POST",
    headers: {
      "content-type": "application/json",
      origin: new URL(url).origin,
    },
    body: JSON.stringify(body),
  });
  if (!response.ok) {
    const text = await response.text().catch(() => "");
    throw new Error(
      `Trellis HTTP request failed (${response.status}) for ${url}${
        text ? `: ${text}` : ""
      }`,
    );
  }
  return await response.json();
}

export async function postAdminRpc(
  proxy: { url: string; token: string },
  method:
    | TrellisTestAdminRpcMethod
    | "completeClientAuth"
    | "resetAcceptedIntegrationAuthorities",
  input: unknown,
): Promise<unknown> {
  const response = await fetch(proxy.url, {
    method: "POST",
    headers: {
      authorization: `Bearer ${proxy.token}`,
      "content-type": "application/json",
    },
    body: JSON.stringify({ method, input }),
    signal: AbortSignal.timeout(190_000),
  });
  const body: unknown = await response.json();
  if (response.ok && isRecord(body) && body.ok === true) return body.output;
  const message = isRecord(body) && typeof body.error === "string"
    ? body.error
    : undefined;
  if (message !== undefined) {
    let serialized: unknown;
    try {
      serialized = JSON.parse(message);
    } catch {
      // Fall through to the transport-level error below.
    }
    if (
      isRecord(serialized) && typeof serialized.type === "string"
    ) {
      const descriptor = getBuiltinRpcError(serialized.type);
      if (descriptor !== undefined) {
        throw descriptor.fromSerializable(serialized);
      }
    }
  }
  throw new Error(
    message !== undefined
      ? message
      : `Trellis test admin RPC proxy returned ${response.status}`,
  );
}
