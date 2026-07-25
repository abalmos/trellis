import { defineAppContract, defineServiceContract } from "@qlever-llc/trellis";
import * as health from "@qlever-llc/trellis/sdk/health";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import type { LiveTrellisRuntime } from "../_support/runtime.ts";
import {
  caseScopedContractId,
  caseScopedName,
  integrationSlug,
} from "../_support/names.ts";

export function createHealthFixture(caseId: string) {
  const slug = integrationSlug(caseId);
  const serviceContract = defineServiceContract({}, () => ({
    id: caseScopedContractId("trellis.integration.health-service", caseId),
    displayName: `Trellis Integration Health Service (${slug})`,
    description: "Service participant that emits Trellis health samples.",
  }));
  const serviceName = caseScopedName("health-fixture-service", caseId);
  const observerContract = defineAppContract(() => ({
    id: caseScopedContractId("trellis.integration.health-observer", caseId),
    displayName: `Trellis Integration Health Observer (${slug})`,
    description: "Reads the projected Trellis health lifecycle.",
    uses: [
      health.HealthQuery,
      health.HealthInspect,
      health.HealthMetrics,
      health.HealthWatch,
      health.HealthStatusChanged.subscribe,
    ],
  }));

  async function setupService(runtime: LiveTrellisRuntime) {
    const serviceKey = await runtime.registerService({
      name: serviceName,
      contract: serviceContract,
    });
    const service = await TrellisService.connect({
      authorizationContextEphemeral: true,
      trellisUrl: runtime.trellisUrl,
      contract: serviceContract,
      name: serviceName,
      identity: serviceKey,
      telemetry: false,
      server: { health: { publishIntervalMs: 1_000 } },
    }).orThrow();
    service.health.setInfo({ version: `0.0.0-health-${slug}` });
    service.health.add("fixture", () => ({
      status: "ok",
      summary: "fixture ready",
      info: { source: "health-integration", caseId },
    }));
    return service;
  }

  return { serviceContract, observerContract, serviceName, setupService };
}
