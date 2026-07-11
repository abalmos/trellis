import { assert, assertEquals } from "@std/assert";
import { join } from "@std/path";
import { connect, credsAuthenticator } from "@nats-io/transport-deno";
import type { HealthHeartbeatSample } from "@qlever-llc/trellis/sdk/health";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createHealthFixture } from "./_fixture.ts";

const CASE_ID = "health.service-publishes-authorized-sample" as const;
const fixture = createHealthFixture(CASE_ID);

liveTrellisTest({
  name:
    "health.service-publishes-authorized-sample publishes on the authorized transport subject",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const nc = await connect({
      servers: runtime.natsUrl,
      authenticator: credsAuthenticator(
        await Deno.readFile(
          join(runtime.workdir, "nats", "creds", "trellis-auth.creds"),
        ),
      ),
    });
    const subscription = nc.subscribe("health.v1.heartbeat.>");
    const observed: Array<{
      subject: string;
      sample: HealthHeartbeatSample;
    }> = [];
    const listener = (async () => {
      for await (const message of subscription) {
        observed.push({
          subject: message.subject,
          sample: JSON.parse(new TextDecoder().decode(message.data)),
        });
      }
    })();
    const service = await fixture.setupService(runtime);

    try {
      const heartbeat = await runtime.waitFor(
        () =>
          observed.find((entry) =>
            entry.sample.participant.contractId ===
              fixture.serviceContract.CONTRACT_ID
          ),
        { timeoutMs: 10_000, intervalMs: 25 },
      );
      assert(heartbeat.subject.startsWith("health.v1.heartbeat.service."));
      assertEquals(heartbeat.subject.split(".").length, 9);
      assertEquals(heartbeat.sample.participant.name, fixture.serviceName);
      assertEquals(heartbeat.sample.participant.kind, "service");
      assertEquals(heartbeat.sample.participant.runtime, "deno");
      assertEquals(heartbeat.sample.reportedStatus, "healthy");
      assert(
        heartbeat.sample.checks.some((check) =>
          check.name === "nats" && check.status === "ok"
        ),
      );
    } finally {
      await service.stop();
      subscription.unsubscribe();
      await listener;
      await nc.close();
    }
  },
});
