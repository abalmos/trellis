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
  method: TrellisTestAdminRpcMethod | "completeClientAuth",
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
  throw new Error(
    isRecord(body) && typeof body.error === "string"
      ? body.error
      : `Trellis test admin RPC proxy returned ${response.status}`,
  );
}
