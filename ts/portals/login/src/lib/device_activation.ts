import {
  getOrCreateSessionKey,
  type SessionKeyHandle,
  TrellisClient,
} from "@qlever-llc/trellis";
import {
  createDeviceActivationController,
  type DeviceActivationAuth,
  type DeviceActivationOperationRef,
} from "@qlever-llc/trellis-svelte";
import { contract } from "../../contract.ts";
import { trellisUrl } from "./config.ts";

type PortalAuthState = Omit<DeviceActivationAuth, "init"> & {
  init(): Promise<SessionKeyHandle>;
};

const participant = {
  id: contract.CONTRACT_ID,
  artifactDigest: contract.CONTRACT_DIGEST,
};

function createPortalAuthState(
  onCallback: (flowId: string) => void,
): PortalAuthState {
  let handle: SessionKeyHandle | null = null;

  async function init(): Promise<SessionKeyHandle> {
    handle ??= await getOrCreateSessionKey();
    return handle;
  }

  return {
    init,
    async handleCallback(callbackUrl) {
      const flowId = new URL(callbackUrl).searchParams.get("flowId");
      if (!flowId) return null;

      onCallback(flowId);
      return null;
    },
    async signIn(options) {
      const redirectTo = new URL(
        options?.redirectTo ?? "/_trellis/portal/users/login",
        window.location.href,
      ).toString();
      await TrellisClient.connect({
        trellisUrl,
        contract,
        participant,
        auth: { handle: await init(), redirectTo, context: options?.context },
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
          handle: await authState.init(),
          currentUrl: authUrlState.currentUrl,
          redirectTo: authUrlState.redirectTo,
          flowId: callbackFlowId,
        },
        onAuthRequired: () => ({ status: "handled" }),
        contract,
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
