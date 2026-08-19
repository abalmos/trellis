import {
  type Client,
  createClient,
  type InArgs,
  type Transaction,
} from "@libsql/client";
import { assertEquals, assertRejects } from "@std/assert";
import {
  createSqliteOutboxSchema,
  type SqlExecutor,
} from "@qlever-llc/trellis/service";
import { assertEventCaptured } from "@qlever-llc/trellis-test";
import { liveTrellisTest, runtimeScopeForCase } from "../_support/runtime.ts";
import { createOutboxFixture } from "./_fixture.ts";

const CASE_ID = "outbox.typescript-sql-commit-rollback" as const;
const fixture = createOutboxFixture(CASE_ID);

function sqlArgs(params: readonly unknown[]): InArgs {
  return params.map((value) => {
    if (
      value === null || typeof value === "string" ||
      typeof value === "number" ||
      typeof value === "bigint" || value instanceof Uint8Array
    ) return value;
    throw new TypeError(`Unsupported SQLite parameter: ${typeof value}`);
  });
}

function executor(client: Client | Transaction): SqlExecutor {
  return {
    async query(sql, params) {
      const result = await client.execute({ sql, args: sqlArgs(params) });
      return result.rows.map((row) => ({ ...row }));
    },
    async execute(sql, params) {
      await client.execute({ sql, args: sqlArgs(params) });
    },
  };
}

liveTrellisTest({
  name:
    "outbox.typescript-sql-commit-rollback dispatches committed records and nothing rolled back",
  scope: runtimeScopeForCase(CASE_ID),
  async fn(runtime) {
    const identity = await fixture.registerService(runtime);
    const capture = await runtime.captureEvents({
      name: fixture.captureName,
      contract: fixture.serviceContract,
      events: [fixture.serviceContract.RecordChanged.subscribe],
    });
    const service = await fixture.connectService(runtime, identity);
    const dbDir = await Deno.makeTempDir({
      prefix: "trellis-outbox-integration-",
    });
    const db = createClient({ url: `file:${dbDir}/outbox.sqlite` });
    const baseExecutor = executor(db);

    try {
      for (const statement of createSqliteOutboxSchema()) {
        await baseExecutor.execute(statement, []);
      }
      await baseExecutor.execute(
        "CREATE TABLE business_records (id TEXT PRIMARY KEY, value TEXT NOT NULL)",
        [],
      );
      const outbox = service.createSqlOutbox<Transaction>({
        dialect: "sqlite",
        executor: baseExecutor,
        transaction: async (work) => {
          const tx = await db.transaction("write");
          try {
            const result = await work({ tx, executor: executor(tx) });
            await tx.commit();
            return result;
          } catch (error) {
            await tx.rollback();
            throw error;
          }
        },
      });

      const committed = await outbox.transaction(async ({ tx, event, job }) => {
        await tx.execute({
          sql: "INSERT INTO business_records (id, value) VALUES (?, ?)",
          args: ["committed", "yes"],
        });
        const first = await event.record.changed.enqueue({
          id: "first",
          value: "one",
        })
          .orThrow();
        const second = await event.record.changed.enqueue({
          id: "second",
          value: "two",
        })
          .orThrow();
        const submission = await job.processRecord.submit({ id: "job" })
          .orThrow();
        return { first, second, submission };
      }).orThrow();

      await assertRejects(() =>
        outbox.transaction(async ({ tx, event }) => {
          await tx.execute({
            sql: "INSERT INTO business_records (id, value) VALUES (?, ?)",
            args: ["rolled-back", "no"],
          });
          await event.record.changed.enqueue({
            id: "rolled-back",
            value: "never",
          })
            .orThrow();
          throw new Error("rollback integration transaction");
        }).orThrow()
      );

      await assertEventCaptured(
        capture,
        "Record.Changed",
        (record) => record.payload.id === "first",
      );
      await assertEventCaptured(
        capture,
        "Record.Changed",
        (record) => record.payload.id === "second",
      );
      const jobOutcome = await runtime.waitFor(
        () =>
          outbox.jobSubmissionOutcome(committed.submission.submissionId)
            .orThrow(),
      );
      assertEquals(jobOutcome?.kind, "accepted");
      await new Promise((resolve) => setTimeout(resolve, 250));
      assertEquals(
        capture.all("Record.Changed").map((record) => record.payload.id).sort(),
        [
          "first",
          "second",
        ],
      );
      assertEquals(
        await baseExecutor.query(
          "SELECT id FROM business_records ORDER BY id",
          [],
        ),
        [{ id: "committed" }],
      );
      assertEquals(
        await baseExecutor.query(
          "SELECT state, count(*) AS count FROM trellis_outbox GROUP BY state",
          [],
        ),
        [{ state: "dispatched", count: 3 }],
      );
      assertEquals(committed.first.state, "pending");
      assertEquals(committed.second.state, "pending");
    } finally {
      db.close();
      await Deno.remove(dbDir, { recursive: true });
      await Promise.allSettled([capture.stop(), service.stop()]);
    }
  },
});
