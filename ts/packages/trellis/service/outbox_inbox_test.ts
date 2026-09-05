import { createClient } from "@libsql/client";
import { assert, assertEquals, assertRejects } from "@std/assert";
import { toFileUrl } from "@std/path";
import { sql } from "drizzle-orm";
import { drizzle } from "npm:drizzle-orm@0.44.7/libsql";
import { createDrizzleSqlExecutor } from "./drizzle.ts";
import {
  createSqliteOutboxSchema,
  SqlInboxRepository,
  SqlOutboxRepository,
} from "./outbox_inbox.ts";

Deno.test("SQLite commits and rolls back business writes with outbox records", async () => {
  const dir = await Deno.makeTempDir();
  const client = createClient({ url: toFileUrl(`${dir}/outbox.db`).href });
  const db = drizzle(client);
  try {
    await db.run(sql`CREATE TABLE orders (id TEXT PRIMARY KEY)`);
    for (const statement of createSqliteOutboxSchema()) {
      await db.run(sql.raw(statement));
    }
    for (const commit of [true, false]) {
      const id = commit ? "committed" : "rolled-back";
      const transaction = db.transaction(async (tx) => {
        await tx.run(sql`INSERT INTO orders (id) VALUES (${id})`);
        const outbox = new SqlOutboxRepository(
          createDrizzleSqlExecutor(tx),
          "sqlite",
        );
        await outbox.enqueue({
          id,
          kind: "event.publish",
          name: "Orders.Created",
          subject: "orders.created",
          payload: JSON.stringify({ id }),
          headers: {},
        });
        if (!commit) {
          // A real constraint failure after both writes must roll back both rows.
          await tx.run(sql`INSERT INTO orders (id) VALUES (${id})`);
        }
      });
      if (commit) await transaction;
      else await assertRejects(() => transaction);
      assertEquals(
        (await db.all(sql`SELECT id FROM orders WHERE id = ${id}`)).length,
        commit ? 1 : 0,
      );
      const outbox = new SqlOutboxRepository(
        createDrizzleSqlExecutor(db),
        "sqlite",
      );
      assertEquals((await outbox.get(id))?.id, commit ? id : undefined);
    }
  } finally {
    client.close();
    await Deno.remove(dir, { recursive: true });
  }
});

Deno.test("SQLite inbox accepts a concurrent message ID exactly once", async () => {
  const dir = await Deno.makeTempDir();
  const clients = [0, 1].map(() =>
    createClient({ url: toFileUrl(`${dir}/inbox.db`).href })
  );
  try {
    const databases = clients.map((client) => drizzle(client));
    for (const statement of createSqliteOutboxSchema()) {
      await databases[0].run(sql.raw(statement));
    }
    const inboxes = databases.map((db) =>
      new SqlInboxRepository(createDrizzleSqlExecutor(db), "sqlite")
    );
    assertEquals(
      (await Promise.all(inboxes.map((inbox) => inbox.record("same-message"))))
        .sort(),
      [false, true],
    );
  } finally {
    for (const client of clients) client.close();
    await Deno.remove(dir, { recursive: true });
  }
});

Deno.test("SQLite outbox recovers abandoned claims without accepting stale completion", async () => {
  const dir = await Deno.makeTempDir();
  const url = toFileUrl(`${dir}/outbox.db`).href;
  const clients = [createClient({ url }), createClient({ url })];
  try {
    const databases = clients.map((client) => drizzle(client));
    for (const statement of createSqliteOutboxSchema()) {
      await databases[0].run(sql.raw(statement));
    }
    const repositories = databases.map((db) =>
      new SqlOutboxRepository(createDrizzleSqlExecutor(db), "sqlite")
    );
    await repositories[0].enqueue({
      id: "event",
      kind: "event.publish",
      name: "Created",
      subject: "events.v1.Created",
      payload: "{}",
      headers: {},
    });
    const now = new Date();
    const claims = (await Promise.all(repositories.map((repository) =>
      repository.claimDue(1, now)
    ))).flat();
    assertEquals(claims.length, 1);
    assertEquals(await repositories[1].claimDue(1, now), []);
    for (const client of clients) {
      client.close();
    }
    const reopened = createClient({ url });
    clients.push(reopened);
    const repository = new SqlOutboxRepository(
      createDrizzleSqlExecutor(drizzle(reopened)),
      "sqlite",
    );
    assert(claims[0].nextAttemptAt);
    const expired = new Date(claims[0].nextAttemptAt);
    const [current] = await repository.claimDue(1, expired);
    assert(current);
    assertEquals(await repository.markDispatched(claims[0], expired), false);
    assertEquals(
      await repository.markFailed(claims[0], {
        error: "stale",
        now: expired,
        nextAttemptAt: expired,
      }),
      false,
    );
    assertEquals(await repository.claimDue(1, expired), []);
    assertEquals(await repository.markDispatched(current, expired), true);
    assertEquals((await repository.get("event"))?.state, "dispatched");
  } finally {
    for (const client of clients) client.close();
    await Deno.remove(dir, { recursive: true });
  }
});
