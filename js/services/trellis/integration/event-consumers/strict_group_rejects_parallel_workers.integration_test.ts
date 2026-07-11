import { assert } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { TrellisService } from "@qlever-llc/trellis/service/deno";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createEventConsumersFixture } from "./_fixture.ts";

const CASE_ID = "event-consumers.strict-group-rejects-parallel-workers";
const fixture = createEventConsumersFixture(CASE_ID);

liveTrellisTest({
  name: CASE_ID,
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.contracts.approve({ contract: fixture.sourceContract });
    const consumerKey = await runtime.registerService({
      name: fixture.consumerName,
      contract: fixture.dependencyConsumerContract,
    });
    const consumer = await TrellisService.connect({
      trellisUrl: runtime.trellisUrl,
      contract: fixture.dependencyConsumerContract,
      name: fixture.consumerName,
      sessionKeySeed: consumerKey.seed,
      telemetry: false,
      server: { log: false },
    }).orThrow();

    try {
      const result = await consumer.event.source.pinged.listen(
        () => Result.ok(undefined),
        {},
        { group: "ingest", concurrency: 2 },
      );
      assert(result.isErr());
    } finally {
      await consumer.stop();
    }
  },
});
