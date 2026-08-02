import { assertEquals, assertStringIncludes } from "@std/assert";
import {
  normalizeJwtConfig,
  renderNatsConfig,
  renderNscScript,
} from "../src/nats_bootstrap.ts";

Deno.test("nats config renders localhost ports and the JetStream store dir", () => {
  const config = renderNatsConfig({
    serverName: "trellis-test",
    ports: { nats: 4222, http: 8222, websocket: 8080 },
    storeDir: "/tmp/work/nats/data",
  });
  assertStringIncludes(config, "server_name: trellis-test");
  assertStringIncludes(config, "listen: 127.0.0.1:4222");
  assertStringIncludes(config, "http: 127.0.0.1:8222");
  assertStringIncludes(config, "listen: 127.0.0.1:8080");
  assertStringIncludes(config, "store_dir: /tmp/work/nats/data");
  assertStringIncludes(config, "include ./jwt.conf");
  assertEquals(config.includes("0.0.0.0"), false);
});

Deno.test("nsc script renders local outDir paths instead of /work", () => {
  const script = renderNscScript("/tmp/trellis-nats", 1);
  assertEquals(script.includes("/work"), false);
  assertStringIncludes(script, "WORK_DIR='/tmp/trellis-nats'");
  assertStringIncludes(script, 'export NKEYS_PATH="$WORK_DIR/.nkeys"');
  assertStringIncludes(script, 'command nsc -H "$WORK_DIR/.nsc" "$@"');
  assertStringIncludes(script, '> "$WORK_DIR/creds/system.creds"');
  assertStringIncludes(script, '> "$WORK_DIR/creds/auth-auth.creds"');
  assertStringIncludes(
    script,
    '--config-file "$WORK_DIR/generated/jwt.conf"',
  );
  assertStringIncludes(script, 'cat > "$WORK_DIR/generated/metadata.json"');
});

Deno.test("nsc script quotes outDirs containing shell metacharacters", () => {
  const script = renderNscScript("/tmp/it's/nats", 1);
  assertStringIncludes(script, "WORK_DIR='/tmp/it'\\''s/nats'");
});

Deno.test("nsc script renders one tenant pair per tenant id", () => {
  const script = renderNscScript("/tmp/trellis-nats", 2);
  assertStringIncludes(script, "AUTH_ACCOUNT_NAME='AUTH_0'");
  assertStringIncludes(script, "AUTH_ACCOUNT_NAME='AUTH_1'");
  assertStringIncludes(script, '> "$WORK_DIR/creds/auth-auth-0.creds"');
  assertStringIncludes(script, '> "$WORK_DIR/creds/auth-auth-1.creds"');
});

Deno.test("normalizeJwtConfig rewrites the resolver dir to the outDir", () => {
  const normalized = normalizeJwtConfig(
    "operator: eyJhbGciOiJlZDI1NTE5LW5rZXkifQ\nresolver {\n    type: full\n    dir: './jwt'\n    allow_delete: false\n}\n",
    "/tmp/trellis-nats",
  );
  assertEquals(
    normalized,
    "operator: eyJhbGciOiJlZDI1NTE5LW5rZXkifQ\nresolver {\n    type: full\n    dir: /tmp/trellis-nats/data/jwt\n    allow_delete: false\n}\n",
  );
});
