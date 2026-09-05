import { FileAuthorizationContextStore } from "@qlever-llc/trellis/auth/file";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { participant } from "./.trellis/ts/participants/acme-orders-service/mod.ts";
import { createOrder } from "./service.ts";

// Environment variables are this example's configuration choice, not a Trellis requirement.
function requiredEnv(name: string): string {
  const value = Deno.env.get(name);
  if (!value) throw new Error(`Missing ${name}`);
  return value;
}

const service = await TrellisService.connect({
  participant,
  name: "orders-service",
  trellisUrl: requiredEnv("TRELLIS_URL"),
  identity: {
    seed: requiredEnv("TRELLIS_IDENTITY_SEED"),
    deploymentId: requiredEnv("TRELLIS_DEPLOYMENT"),
    instanceId: requiredEnv("TRELLIS_INSTANCE"),
    participantId: requiredEnv("TRELLIS_PARTICIPANT_ID"),
    participantArtifactDigest: requiredEnv(
      "TRELLIS_PARTICIPANT_ARTIFACT_DIGEST",
    ),
    participantNeedsDigest: requiredEnv("TRELLIS_PARTICIPANT_NEEDS_DIGEST"),
  },
  authorizationContextStore: new FileAuthorizationContextStore(
    "./trellis-context.json",
  ),
}).orThrow();

const stop = () => {
  void service.stop();
};
Deno.addSignalListener("SIGINT", stop);
Deno.addSignalListener("SIGTERM", stop);
try {
  await service.handleOrdersCreate(createOrder);
  console.log("Orders service connected; press Ctrl-C to stop.");
  await service.wait();
} finally {
  Deno.removeSignalListener("SIGINT", stop);
  Deno.removeSignalListener("SIGTERM", stop);
  await service.stop();
}
