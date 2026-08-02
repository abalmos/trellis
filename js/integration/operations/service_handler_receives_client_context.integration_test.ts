import { assert, assertEquals, assertExists } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createOperationsFixture } from "./_fixture.ts";

const CASE_ID = "operations.service-handler-receives-client-context" as const;
const fixture = createOperationsFixture(CASE_ID);

liveTrellisTest({
  name:
    "operations.service-handler-receives-client-context passes caller metadata and service client to the handler",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);

    try {
      await service.handleEntityProcess(({ input, caller, client }) => {
        assertExists(client);
        assertExists(caller);
        assertEquals(input.message, fixture.message);
        if (caller.type !== "verified") {
          throw new Error("expected a verified caller");
        }
        assertEquals(caller.participant.kind, "app");
        assertEquals(caller.participant.id, fixture.clientContract.CONTRACT.id);
        assert(caller.sessionId.length > 0);
        return Result.ok({ message: caller.type, done: true });
      });

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });
      const ref = await client.entityProcess({
        message: fixture.message,
      }).start().orThrow();

      const terminal = await ref.wait().orThrow();
      assertEquals(terminal.state, "completed");
      assertEquals(terminal.output?.done, true);
      assertEquals(terminal.output?.message, "verified");
    } finally {
      await service.stop();
    }
  },
});
