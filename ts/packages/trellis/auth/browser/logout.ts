import type { AsyncResult, BaseError } from "@qlever-llc/result";
import type {
  AuthSessionsLogoutInput,
  AuthSessionsLogoutOutput,
} from "../../internal_sdk/generated/auth/mod.ts";
import { clearSessionKey, type SessionKeyHandle } from "./session.ts";

type LogoutLocation =
  & Pick<Location, "href">
  & Partial<Pick<Location, "assign">>;

/** Generated Auth.Sessions.Logout call exposed by a connected runtime. */
export type ConnectedSessionLogout = (
  input: AuthSessionsLogoutInput,
) => AsyncResult<AuthSessionsLogoutOutput, BaseError>;

export type CompleteSessionLogoutArgs = {
  handle: SessionKeyHandle;
  connected?: ConnectedSessionLogout;
  returnTo?: string;
  location?: LogoutLocation;
};

/** Revoke a connected session through generated Auth control or clear locally. */
export async function logoutSession(args: {
  handle: SessionKeyHandle;
  connected?: ConnectedSessionLogout;
}): Promise<{ success: true }> {
  if (args.connected) {
    await args.connected({}).orThrow();
  } else {
    await clearSessionKey({ persistence: args.handle.persistence });
  }
  return { success: true };
}

/** Complete session logout without an HTTP control-plane fallback. */
export async function completeSessionLogout(
  args: CompleteSessionLogoutArgs,
): Promise<never> {
  try {
    if (args.connected) {
      await logoutSession({ handle: args.handle, connected: args.connected });
    }
  } finally {
    try {
      await clearSessionKey({ persistence: args.handle.persistence });
    } catch {
      // Temporary/test runtimes may not provide IndexedDB.
    }
  }

  const target = args.returnTo ?? "/";
  const location = args.location ?? globalThis.location;
  if (typeof location.assign === "function") {
    location.assign(target);
  } else {
    location.href = target;
  }
  throw new Error("Redirecting after logout");
}
