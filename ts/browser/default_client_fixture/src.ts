import {
  ClientAuthHandledError,
  TrellisClient,
} from "../../packages/trellis/index.ts";
import { clientA, clientB, clientC } from "./contract.ts";

const status = document.querySelector("#status");
if (!(status instanceof HTMLElement)) {
  throw new Error("status element is missing");
}

const config = await fetch("./config.json").then((response) =>
  response.json()
) as {
  trellisUrl: string;
};
const parameters = new URL(location.href).searchParams;
const contract = parameters.get("participant") === "c"
  ? clientC
  : parameters.get("participant") === "b"
  ? clientB
  : clientA;
const manual = parameters.get("manual") === "1";

try {
  const client = await TrellisClient.connect({
    trellisUrl: config.trellisUrl,
    contract,
    participant: {
      id: contract.CONTRACT_ID,
      artifactDigest: contract.CONTRACT_DIGEST,
    },
    auth: { redirectTo: location.href },
    ...(manual
      ? {
        onAuthRequired: async ({ loginUrl }: { loginUrl: string }) => {
          status.textContent = "auth-required";
          Object.assign(globalThis, { authFixture: { loginUrl } });
          return { status: "handled" as const };
        },
      }
      : {}),
  }).orThrow();
  const me = await client.authSessionsMe({}).orThrow();
  status.textContent = "connected";
  Object.assign(globalThis, {
    authFixture: {
      me,
      logout: async () => {
        await client.logout();
        status.textContent = "logged-out";
      },
    },
  });
} catch (error) {
  if (!(error instanceof ClientAuthHandledError)) {
    const cause = error instanceof Error && error.cause instanceof Error
      ? `: ${error.cause.message}`
      : "";
    status.textContent = `error: ${
      error instanceof Error ? error.message : String(error)
    }${cause}`;
    throw error;
  }
  if (!manual) status.textContent = "redirecting";
}
