import { TrellisClient } from "@qlever-llc/trellis";
import {
  createDeviceActivationController,
  type DeviceActivationAuth,
  type DeviceActivationOperationRef,
} from "@qlever-llc/trellis-svelte";
import { participant } from "../../.trellis/ts/participants/app-portal/mod.ts";
import { trellisUrl } from "./portal_config.ts";

type PortalAuthState = DeviceActivationAuth;

function createPortalAuthState(
  onCallback: (flowId: string) => void,
): PortalAuthState {
  return {
    async init() {},
    async handleCallback(callbackUrl) {
      const flowId = new URL(callbackUrl).searchParams.get("flowId");
      if (!flowId) return null;

      onCallback(flowId);
      return null;
    },
    async signIn(options) {
      const redirectTo = new URL(
        options?.redirectTo ?? "/login",
        window.location.href,
      ).toString();
      await TrellisClient.connect({
        trellisUrl,
        participant,
        auth: { redirectTo, context: options?.context },
        onAuthRequired: ({ loginUrl }) => {
          window.location.href = loginUrl;
          throw new Error("Browser authentication redirect started");
        },
      }).orThrow();
    },
  };
}

/**
 * Creates the device activation controller used by the portal route.
 */
export function createPortalDeviceActivationController() {
  let callbackFlowId: string | undefined;
  const authState = createPortalAuthState((flowId) => {
    callbackFlowId = flowId;
  });

  return createDeviceActivationController({
    authState,
    createClient: async (authUrlState) => {
      const trellis = await TrellisClient.connect({
        trellisUrl,
        auth: {
          currentUrl: authUrlState.currentUrl,
          redirectTo: authUrlState.redirectTo,
          flowId: callbackFlowId,
        },
        onAuthRequired: () => ({ status: "handled" }),
        participant,
      }).orThrow();
      callbackFlowId = undefined;

      return {
        async activateDevice(input): Promise<DeviceActivationOperationRef> {
          return await trellis.authDeviceUserAuthoritiesResolve(input)
            .start()
            .orThrow();
        },
      };
    },
    sessionStorage: typeof window === "undefined"
      ? undefined
      : window.sessionStorage,
  });
}
