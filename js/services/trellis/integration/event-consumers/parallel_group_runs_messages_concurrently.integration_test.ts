import { assert, assertEquals } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createEventConsumersFixture } from "./_fixture.ts";

const CASE_ID = "event-consumers.parallel-group-runs-messages-concurrently";
const fixture = createEventConsumersFixture(CASE_ID);

liveTrellisTest({
  name: CASE_ID,
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.contracts.approve({ contract: fixture.sourceContract });
    const consumerKey = await runtime.registerService({
      name: fixture.consumerName,
      contract: fixture.parallelConsumerContract,
    });
    const publisher = await runtime.connectClient({
      name: fixture.publisherName,
      contract: fixture.sourcePublisherContract,
    });
    const consumer = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: fixture.parallelConsumerContract,
      name: fixture.consumerName,
      sessionKeySeed: consumerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();
    let active = 0;
    let maxActive = 0;
    let releaseFirst!: () => void;
    const firstBlocked = new Promise<void>((resolve) => {
      releaseFirst = resolve;
    });

    try {
      await consumer.onSourcePinged(
        async ({ event: event }) => {
          active += 1;
          maxActive = Math.max(maxActive, active);
          if (event.value === "first") await firstBlocked;
          active -= 1;
          return Result.ok(undefined);
        },
        {},
        { group: "ingest", concurrency: 2 },
      ).orThrow();
      const conflicting = await consumer.onSourcePinged(
        () => Result.ok(undefined),
        {},
        { group: "ingest", concurrency: 3 },
      );
      assert(conflicting.isErr());

      await publisher.publishSourcePinged({
        id: fixture.eventId,
        value: "first",
      }).orThrow();
      await runtime.waitFor(() => active === 1);
      await publisher.publishSourcePinged({
        id: fixture.secondEventId,
        value: "second",
      }).orThrow();
      await runtime.waitFor(() => maxActive === 2);
      assertEquals(maxActive, 2);
    } finally {
      releaseFirst();
      await consumer.stop();
    }
  },
});
