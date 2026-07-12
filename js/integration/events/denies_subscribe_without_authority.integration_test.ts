import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createEventsFixture } from "./_fixture.ts";

const CASE_ID = "events.denies-subscribe-without-authority" as const;
const fixture = createEventsFixture(CASE_ID);

liveTrellisTest({
  name:
    "events.denies-subscribe-without-authority does not deliver events to a publish-only client",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.contracts.approve({ contract: fixture.serviceContract });
    const publishOnlyClient = await runtime.connectClient({
      name: fixture.publishOnlyName,
      contract: fixture.publishOnlyClientContract,
    });
    assertEquals("onEntityChanged" in publishOnlyClient, false);
  },
});
