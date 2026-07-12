import { assert } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createFeedsFixture } from "./_fixture.ts";

const CASE_ID = "feeds.abort-stops-client-subscription" as const;
const fixture = createFeedsFixture(CASE_ID);

liveTrellisTest({
  name: "feeds.abort-stops-client-subscription stops the feed stream on abort",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);

    try {
      let providerStopped = false;
      let providerStarts = 0;
      await service.handleEntityLive(async ({ signal }) => {
        providerStarts++;
        await new Promise<void>((resolve) => {
          signal.addEventListener("abort", () => {
            providerStopped = true;
            resolve();
          }, { once: true });
        });
      });

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });
      const controller = new AbortController();

      const stream = await client.entityLive({ topic: fixture.topic },
      { signal: controller.signal },).orThrow();

      controller.abort();

      let terminated = false;
      const timeout = new Promise<void>((_, reject) =>
        setTimeout(
          () => reject(new Error("stream did not terminate after abort")),
          5000,
        )
      );
      const iterate = (async () => {
        for await (const _ of stream) {
          // drain
        }
        terminated = true;
      })();

      await Promise.race([iterate, timeout]);
      assert(terminated, "feed stream should terminate after abort");

      await Promise.race([
        (async () => {
          while (!providerStopped) {
            await new Promise((resolve) => setTimeout(resolve, 10));
          }
        })(),
        new Promise<void>((_, reject) =>
          setTimeout(
            () =>
              reject(new Error("provider did not receive feed cancellation")),
            5000,
          )
        ),
      ]);
      assert(providerStopped, "feed provider should stop after client abort");

      const alreadyAborted = new AbortController();
      alreadyAborted.abort();
      const abortedResult = await client.entityLive({ topic: fixture.topic },
      { signal: alreadyAborted.signal },);
      assert(
        abortedResult.isErr(),
        "an already-aborted feed should be rejected",
      );
      assert(
        providerStarts === 1,
        "an already-aborted feed should not reach the provider",
      );
    } finally {
      await service.stop();
    }
  },
});
