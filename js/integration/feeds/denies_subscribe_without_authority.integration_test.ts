import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createFeedsFixture } from "./_fixture.ts";

const CASE_ID = "feeds.denies-subscribe-without-authority" as const;
const fixture = createFeedsFixture(CASE_ID);

liveTrellisTest({
  name:
    "feeds.denies-subscribe-without-authority rejects an unauthorized feed subscribe",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.contracts.approve({ contract: fixture.serviceContract });
    const client = await runtime.connectClient({
      name: fixture.unauthorizedClientName,
      contract: fixture.unauthorizedClientContract,
    });

    assertEquals("entityLive" in client, false);
  },
});
