import { assert, assertEquals } from "@std/assert";
import { Result, UnexpectedError } from "@qlever-llc/trellis";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createOperationsFixture } from "./_fixture.ts";

const CASE_ID = "operations.callback-terminal-and-resume-live" as const;
const fixture = createOperationsFixture(CASE_ID, { cancelable: true });

function deferred(): { promise: Promise<void>; resolve: () => void } {
  let resolve!: () => void;
  const promise = new Promise<void>((done) => {
    resolve = done;
  });
  return { promise, resolve };
}

liveTrellisTest({
  name:
    "operations.callback-terminal-and-resume-live observes callbacks, terminal states, and resumed refs",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const service = await fixture.connectService(runtime);
    const cancelledGate = deferred();
    const callbackMessage = `${fixture.message}:callbacks`;
    const failedMessage = `${fixture.message}:failed`;
    const cancelledMessage = `${fixture.message}:cancelled`;

    try {
      await service.handle.operation.entity.process(async ({ input, op }) => {
        await op.started().orThrow();
        if (input.message === failedMessage) {
          await op.fail(new UnexpectedError({ cause: new Error("boom") }))
            .orThrow();
          return op.defer();
        }
        if (input.message === cancelledMessage) {
          await cancelledGate.promise;
          return op.defer();
        }
        await op.progress({ message: input.message, step: 1 }).orThrow();
        return Result.ok({ message: `${input.message}:done`, done: true });
      });

      const client = await runtime.connectClient({
        name: fixture.clientName,
        contract: fixture.clientContract,
      });

      const callbackOrder: string[] = [];
      const callbackRef = await client.operation.entity.process.input({
        message: callbackMessage,
      })
        .onAccepted((event) => {
          callbackOrder.push(event.type);
        })
        .onStarted((event) => {
          callbackOrder.push(event.type);
        })
        .onCompleted((event) => {
          callbackOrder.push(event.type);
        })
        .onEvent((event) => {
          callbackOrder.push(`${event.type}:event`);
        })
        .start().orThrow();
      const callbackTerminal = await callbackRef.wait().orThrow();
      assertEquals(callbackTerminal.state, "completed");
      assertEquals(callbackTerminal.output, {
        message: `${callbackMessage}:done`,
        done: true,
      });
      assert(
        callbackOrder.indexOf("accepted") <
          callbackOrder.indexOf("accepted:event"),
      );
      assert(
        callbackOrder.indexOf("completed") <
          callbackOrder.indexOf("completed:event"),
      );

      const resumed = client.operation.entity.process.resume({
        id: callbackRef.id,
        service: callbackRef.service,
        operation: callbackRef.operation,
      });
      const resumedSnapshot = await resumed.get().orThrow();
      assertEquals(resumedSnapshot.state, "completed");
      assertEquals(resumedSnapshot.output, callbackTerminal.output);

      const failedCallbacks: string[] = [];
      const failedRef = await client.operation.entity.process.input({
        message: failedMessage,
      })
        .onFailed((event) => {
          failedCallbacks.push(event.snapshot.state);
        })
        .onEvent((event) => {
          failedCallbacks.push(`${event.type}:event`);
        })
        .start().orThrow();
      const failedTerminal = await failedRef.wait().orThrow();
      assertEquals(failedTerminal.state, "failed");
      const resumedFailed = client.operation.entity.process.resume({
        id: failedRef.id,
        service: failedRef.service,
        operation: failedRef.operation,
      });
      const resumedFailedSnapshot = await resumedFailed.get().orThrow();
      assertEquals(resumedFailedSnapshot.state, "failed");
      assert(
        failedCallbacks.indexOf("failed") <
          failedCallbacks.indexOf("failed:event"),
      );

      const cancelledCallbacks: string[] = [];
      const cancelledRef = await client.operation.entity.process.input({
        message: cancelledMessage,
      })
        .onCancelled((event) => {
          cancelledCallbacks.push(event.snapshot.state);
        })
        .onEvent((event) => {
          cancelledCallbacks.push(`${event.type}:event`);
        })
        .start().orThrow();
      const cancelledSnapshot = await cancelledRef.cancel().orThrow();
      assertEquals(cancelledSnapshot.state, "cancelled");
      cancelledGate.resolve();
      const cancelledTerminal = await cancelledRef.wait().orThrow();
      assertEquals(cancelledTerminal.state, "cancelled");
      const resumedCancelled = client.operation.entity.process.resume({
        id: cancelledRef.id,
        service: cancelledRef.service,
        operation: cancelledRef.operation,
      });
      const resumedCancelledSnapshot = await resumedCancelled.get().orThrow();
      assertEquals(resumedCancelledSnapshot.state, "cancelled");
      assert(
        cancelledCallbacks.indexOf("cancelled") <
          cancelledCallbacks.indexOf("cancelled:event"),
      );
    } finally {
      cancelledGate.resolve();
      await service.stop();
    }
  },
});
