import { assertEquals, assertThrows } from "@std/assert";

import { LiveSessionManager } from "./manager.ts";

Deno.test("NX09 same-epoch resume keeps generation; reconnect does not revive", () => {
  const live = new LiveSessionManager(2);
  assertEquals(live.generation(), 1);
  assertEquals(live.isAvailable(), true);
  const first = live.admitConsumer();
  const generation = live.generation();
  live.resume();
  assertEquals(live.generation(), generation, "resume is not a reconnect");
  assertEquals(live.isAvailable(), true);
  const second = live.admitConsumer();
  first[Symbol.dispose]();
  second[Symbol.dispose]();

  live.suspend();
  const afterDisconnect = live.generation();
  assertEquals(afterDisconnect > generation, true);
  assertEquals(live.isAvailable(), false);
  assertThrows(() => live.admitConsumer());
  live.resume();
  assertEquals(live.generation(), afterDisconnect);
  assertEquals(live.isAvailable(), true);
  const replacement = live.admitConsumer();
  replacement[Symbol.dispose]();
});

Deno.test("NX10 failed setup releases the consumer permit", () => {
  const live = new LiveSessionManager(1);
  const permit = live.admitConsumer();
  assertEquals(live.consumerCount(), 1);
  assertThrows(() => live.admitConsumer());
  permit[Symbol.dispose]();
  assertEquals(live.consumerCount(), 0);
  const again = live.admitConsumer();
  again[Symbol.dispose]();
  live.stop();
  assertEquals(live.isAvailable(), false);
  assertThrows(() => live.admitConsumer());
});
