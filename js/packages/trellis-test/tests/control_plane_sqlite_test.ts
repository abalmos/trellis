import { assertEquals } from "@std/assert";
import { TrellisControlPlaneSqlite } from "../src/control_plane_sqlite.ts";

Deno.test("TrellisControlPlaneSqlite queries and mutates a runtime database", async () => {
  const dbPath = await Deno.makeTempFile({ suffix: ".sqlite" });
  const sqlite = new TrellisControlPlaneSqlite(dbPath);

  try {
    await sqlite.execute(
      "create table auth_sessions (session_id text primary key, session_public_key text unique, value text)",
    );
    const inserted = await sqlite.execute(
      "insert into auth_sessions (session_id, session_public_key, value) values (?, ?, ?)",
      ["ses_1", "session-1", "before"],
    );
    assertEquals(inserted.rowsAffected, 1);

    const rows = await sqlite.query(
      "select session_id, session_public_key, value from auth_sessions where session_public_key = ?",
      ["session-1"],
    );
    assertEquals(rows, [{
      session_id: "ses_1",
      session_public_key: "session-1",
      value: "before",
    }]);

    const snapshot = await sqlite.takeSession("session-1");
    assertEquals(await sqlite.query("select * from auth_sessions"), []);
    assertEquals((await snapshot?.restore())?.rowsAffected, 1);
    assertEquals(await sqlite.query("select * from auth_sessions"), [{
      session_id: "ses_1",
      session_public_key: "session-1",
      value: "before",
    }]);

    const deleted = await sqlite.execute(
      "delete from auth_sessions where session_public_key = ?",
      ["session-1"],
    );
    assertEquals(deleted.rowsAffected, 1);
    assertEquals(await sqlite.query("select * from auth_sessions"), []);
  } finally {
    await Deno.remove(dbPath).catch(() => undefined);
  }
});
