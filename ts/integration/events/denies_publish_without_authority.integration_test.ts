import { assertEquals } from "@std/assert";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createEventsFixture } from "./_fixture.ts";

const CASE_ID = "events.denies-publish-without-authority" as const;
const fixture = createEventsFixture(CASE_ID);

liveTrellisTest({
  name:
    "events.denies-publish-without-authority rejects a subscribe-only client publish",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    await runtime.services.createInstance({
      name: `${fixture.subscribeOnlyName}-provider`,
      contract: fixture.serviceContract,
    });
    const client = await runtime.connectClient({
      name: fixture.subscribeOnlyName,
      contract: fixture.subscribeOnlyClientContract,
    });

    assertEquals("publishEntityChanged" in client, false);
  },
});
