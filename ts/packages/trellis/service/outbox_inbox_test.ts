import { createClient } from "@libsql/client";
import { assertEquals, assertRejects } from "@std/assert";
import { toFileUrl } from "@std/path";
import { sql } from "drizzle-orm";
import { drizzle } from "npm:drizzle-orm@0.44.7/libsql";
import { createDrizzleSqlExecutor } from "./drizzle.ts";
import {
  createSqliteOutboxSchema,
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
