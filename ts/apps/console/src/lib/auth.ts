import { resolve } from "$app/paths";
import {
  clearSessionKey,
  completeSessionLogout,
  getOrCreateSessionKey,
} from "@qlever-llc/trellis/auth/browser";
import contract from "../../contract.ts";
import { APP_CONFIG, buildAppLoginUrl } from "./config.ts";

function storageScope(): string {
  if (!APP_CONFIG.authUrl) throw new Error("Trellis URL is not configured.");
  return `${
    new URL(APP_CONFIG.authUrl).origin
  }:${contract.CONTRACT_ID}:${contract.CONTRACT_DIGEST}`;
}

/** Converts auth callback reason codes into console-facing messages. */
export function formatConsoleAuthError(error: string): string {
  switch (error) {
    case "approval_denied":
      return "App access was denied.";
    case "flow_expired":
      return "This sign-in request expired. Start sign-in again.";
    default:
      return error;
  }
}

/** Logs out the current Console browser session. */
export async function signOut(): Promise<never> {
  return await completeSessionLogout({
    handle: await getOrCreateSessionKey({ storageScope: storageScope() }),
    returnTo: buildConsoleLoginUrl({ redirectTo: "/profile" }),
  });
}

/** Clears the Console browser session before restarting authentication. */
export async function resetSession(): Promise<void> {
  await clearSessionKey({ storageScope: storageScope() });
}

function toUrl(location: URL | Location): URL {
  return location instanceof URL
    ? new URL(location.toString())
    : new URL(location.href);
}

export function resolveConsolePath(
  path: string,
  location: URL | Location = globalThis.location,
): string {
  const url = new URL(path, toUrl(location));
  const appBase = resolve("/").replace(/\/$/, "");
  const currentUrl = toUrl(location);
  if (url.origin !== currentUrl.origin) return url.toString();
  if (appBase && url.pathname === appBase) {
    return `${appBase}/${url.search}${url.hash}`;
  }
  if (appBase && url.pathname.startsWith(`${appBase}/`)) {
    return `${appBase}${
      url.pathname.slice(appBase.length)
    }${url.search}${url.hash}`;
  }
  return `${appBase}${url.pathname}${url.search}${url.hash}`;
}

export function getConsoleRedirectTarget(
  location: URL | Location = globalThis.location,
  fallback = "/profile",
): string {
  const currentUrl = toUrl(location);
  const redirectTo = currentUrl.searchParams.get("redirectTo");
  if (!redirectTo) return resolveConsolePath(fallback, currentUrl);
  const redirectUrl = new URL(redirectTo, currentUrl);
  return redirectUrl.origin === currentUrl.origin
    ? resolveConsolePath(redirectTo, currentUrl)
    : resolveConsolePath(fallback, currentUrl);
}

export function buildConsoleLoginUrl(options: {
  redirectTo: string;
  location?: URL | Location;
  authError?: string;
}): string {
  const location = options.location ?? globalThis.location;
  return buildAppLoginUrl(
    resolveConsolePath(options.redirectTo, location),
    location,
    options.authError,
    resolve("/login"),
  );
}
