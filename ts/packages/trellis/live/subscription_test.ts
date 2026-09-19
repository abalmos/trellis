import { assertEquals } from "@std/assert";

import { LiveSessionManager } from "./manager.ts";
import { ConsumerCore, LiveSubscription } from "./subscription.ts";
import { LiveCancellation, LiveEnd, LiveStreamError } from "./types.ts";

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
  assertEquals(core.consumedSeq(), 0);
});

Deno.test("first next activates the prepared handle", async () => {
  const core = new ConsumerCore<number>("session");
  const sub = new LiveSubscription(core, new LiveCancellation());
  assertEquals(sub.activated, false);
  core.admit({ value: 9, encodedLen: 1 });
  core.commitEnd(new LiveEnd("complete"));
  const seen: number[] = [];
  for await (const value of sub) seen.push(value);
  assertEquals(sub.activated, true);
  assertEquals(seen, [9]);
});

Deno.test("consumer permit releases on dispose", () => {
  const manager = new LiveSessionManager(2);
  const first = manager.admitConsumer();
  const second = manager.admitConsumer();
  assertEquals(manager.consumerCount(), 2);
  first[Symbol.dispose]();
  assertEquals(manager.consumerCount(), 1);
  second[Symbol.dispose]();
  assertEquals(manager.consumerCount(), 0);
});
