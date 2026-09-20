import { assertEquals } from "@std/assert";

import { LiveSessionManager } from "./manager.ts";
import { ConsumerCore, LiveSubscription } from "./subscription.ts";
import {
  LiveCancellation,
  type LiveCloseReceipt,
  LiveEnd,
  LiveStreamError,
} from "./types.ts";

function noopClose(): Promise<LiveCloseReceipt> {
  return Promise.resolve({
    end: new LiveEnd("cancelled"),
    remote: "not-required",
    cleanup: "unknown",
  });
}

Deno.test("NX07 TS abnormal end discards queued items", () => {
  const core = new ConsumerCore<number>("session");
  assertEquals(core.admit({ value: 1, encodedLen: 1 }), true);
  core.commitEnd(
    new LiveEnd(
      "authorization_lost",
      new LiveStreamError("revoked", "revoked"),
    ),
  );
  core.discardQueue();
  assertEquals(core.hasQueued(), false);
  assertEquals(core.consumedSeq(), 0n);
});

Deno.test("an active complete end drains queued items then completes", async () => {
  const core = new ConsumerCore<number>("session");
  const sub = new LiveSubscription(
    core,
    new LiveCancellation(),
    noopClose,
  );
  assertEquals(sub.activated, false);
  core.setPhase("draining");
  assertEquals(core.admit({ value: 9, encodedLen: 1 }), true);
  core.setPendingEnd(new LiveEnd("complete"));
  const seen: number[] = [];
  for await (const value of sub) seen.push(value);
  assertEquals(sub.activated, true);
  assertEquals(seen, [9]);
});

Deno.test("prepared handle does not yield before activation", async () => {
  const core = new ConsumerCore<number>("session");
  const sub = new LiveSubscription(
    core,
    new LiveCancellation(),
    noopClose,
  );
  assertEquals(core.admit({ value: 7, encodedLen: 1 }), true);
  const iterator = sub[Symbol.asyncIterator]();
  const pending = iterator.next();
  core.setPhase("active");
  const first = await pending;
  assertEquals(first.done, false);
  assertEquals(first.value, 7);
});

Deno.test("consumer permit releases on dispose", () => {
  const manager = new LiveSessionManager();
  const first = manager.admitConsumer();
  const second = manager.admitConsumer();
  assertEquals(manager.consumerCount(), 2);
  first[Symbol.dispose]();
  assertEquals(manager.consumerCount(), 1);
  second[Symbol.dispose]();
  assertEquals(manager.consumerCount(), 0);
});
