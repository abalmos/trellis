import { assert, assertEquals } from "@std/assert";
import { Result } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createOperationsFixture } from "./_fixture.ts";

const CASE_ID =
  "operations.live-updates-are-typed-ordered-and-transient" as const;
const fixture = createOperationsFixture(CASE_ID);

liveTrellisTest({
  name:
    "operations.live-updates-are-typed-ordered-and-transient observes ordered typed updates without persisting them",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    let releaseUpdates!: () => void;
    const updatesReady = new Promise<void>((resolve) => {
      releaseUpdates = resolve;
    });
    let releaseTerminal!: () => void;
    const terminalReady = new Promise<void>((resolve) => {
      releaseTerminal = resolve;
    });

    try {
      await service.handleEntityProcess(async ({ input, op }) => {
        await op.started().orThrow();
        await updatesReady;
        await op.emitUpdate({ message: input.message, step: 1 }).orThrow();
        await op.emitUpdate({ message: input.message, step: 2 }).orThrow();
        await terminalReady;
        return Result.ok({ message: "updates-complete", done: true });
      });

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });
      const callbackUpdates: number[] = [];
      const ref = await client.entityProcess({
        message: fixture.message,
      }).onUpdate((event) => {
        callbackUpdates.push(event.update.step);
      }).start().orThrow();

      const optedEvents = await ref.watch({ updates: true }).orThrow();
      const optedIterator = optedEvents[Symbol.asyncIterator]();
      const optedInitial = await optedIterator.next();
      assert(!optedInitial.done);

      const defaultEvents = await ref.watch().orThrow();
      const defaultIterator = defaultEvents[Symbol.asyncIterator]();
      const defaultInitial = await defaultIterator.next();
      assert(!defaultInitial.done);
      assert(defaultInitial.value.type !== "update");

      let observedTwoUpdates!: () => void;
      const twoUpdates = new Promise<void>((resolve) => {
        observedTwoUpdates = resolve;
      });
      const optedResult = (async () => {
        const updates: Array<{ message: string; step: number }> = [];
        let terminalOutput: { message: string; done: boolean } | undefined;
        while (true) {
          const next = await optedIterator.next();
          if (next.done) break;
          if (next.value.type === "update") {
            updates.push(next.value.update);
            if (updates.length === 2) observedTwoUpdates();
          }
          if (next.value.type === "completed") {
            terminalOutput = next.value.snapshot.output;
          }
        }
        return { updates, terminalOutput };
      })();
      const defaultTypes = (async () => {
        const types = [defaultInitial.value.type];
        while (true) {
          const next = await defaultIterator.next();
          if (next.done) return types;
          types.push(next.value.type);
        }
      })();

      releaseUpdates();
      await twoUpdates;

      const running = await ref.get().orThrow();
      assertEquals(running.state, "running");
      assertEquals(running.progress, undefined);
      assertEquals(running.output, undefined);
      assertEquals("update" in running, false);

      releaseTerminal();
      const terminal = await ref.wait().orThrow();
      const opted = await optedResult;
      const nonOptedTypes = await defaultTypes;
      const expectedOutput = { message: "updates-complete", done: true };

      assertEquals(opted.updates, [
        { message: fixture.message, step: 1 },
        { message: fixture.message, step: 2 },
      ]);
      assertEquals(callbackUpdates, [1, 2]);
      assertEquals(opted.terminalOutput, expectedOutput);
      assertEquals(nonOptedTypes.includes("update"), false);
      assertEquals(terminal.output, expectedOutput);
      assertEquals("update" in terminal, false);

      const persisted = await ref.get().orThrow();
      assertEquals(persisted.output, expectedOutput);
      assertEquals("update" in persisted, false);

      const lateEvents = await ref.watch({ updates: true }).orThrow();
      const lateTypes: string[] = [];
      for await (const event of lateEvents) lateTypes.push(event.type);
      assertEquals(lateTypes, ["completed"]);
    } finally {
      releaseUpdates();
      releaseTerminal();
      await service.stop().catch(() => undefined);
    }
  },
});
