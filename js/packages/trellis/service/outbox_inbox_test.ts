import { assertEquals } from "@std/assert";

import { SqlOutboxRepository } from "./outbox_inbox.ts";

Deno.test("SqlOutboxRepository reads decoded PostgreSQL outcomes", async () => {
  const repository = new SqlOutboxRepository({
    execute: () => Promise.resolve(),
    query: () =>
      Promise.resolve([{
        id: "submission-1",
        kind: "job.create",
        name: "sync",
        subject: "trellis.jobs.svc.sync.job-1.created",
        payload: "{}",
        headers: "{}",
        state: "dispatched",
        attempts: 0,
        created_at: "2026-07-10T00:00:00.000Z",
        updated_at: "2026-07-10T00:00:01.000Z",
        next_attempt_at: null,
        last_error: null,
        outcome: { kind: "rejected", reason: "queue-depth" },
      }]),
  }, "postgres");

  assertEquals((await repository.get("submission-1"))?.outcome, {
    kind: "rejected",
    reason: "queue-depth",
  });
});
