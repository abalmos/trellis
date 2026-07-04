import { assert, assertEquals } from "@std/assert";
import Value from "typebox/value";
import {
  createHealthHeartbeat,
  type HealthCheckResult,
  runAllServiceHealthChecks,
  runServiceHealthCheck,
  ServiceHealth,
} from "./health.ts";
import { HealthHeartbeatSchema } from "./health_schemas.ts";

Deno.test("runServiceHealthCheck returns check results", async () => {
  const ok = await runServiceHealthCheck("db", () => ({ status: "ok" }));
  assertEquals(ok.name, "db");
  assertEquals(ok.status, "ok");
  assertEquals(typeof ok.latencyMs, "number");

  const failed = await runServiceHealthCheck("cache", () => ({
    status: "failed",
    summary: "connection timeout",
  }));
  assertEquals(failed.status, "failed");
  assertEquals(failed.summary, "connection timeout");
});

Deno.test("runServiceHealthCheck catches thrown check errors", async () => {
  const result = await runServiceHealthCheck("broken", () => {
    throw new Error("boom");
  });

  assertEquals(result.status, "failed");
  assertEquals(result.error, "boom");
  assertEquals(result.info?.errorType, "Error");
});

Deno.test("runAllServiceHealthChecks runs named checks", async () => {
  const results = await runAllServiceHealthChecks({
    db: () => ({ status: "ok" }),
    cache: () => ({ status: "failed", error: "down" }),
  });

  assertEquals(results.length, 2);
  assert(results.some((check) => check.name === "db" && check.status === "ok"));
  assert(
    results.some((check) =>
      check.name === "cache" && check.status === "failed"
    ),
  );
});

Deno.test("createHealthHeartbeat includes baseline service metadata", () => {
  const heartbeat = createHealthHeartbeat({
    serviceName: "activity",
    instanceId: "instance-1",
    contractId: "trellis.audit@v1",
    contractDigest: "digest",
    startedAt: "2026-01-01T00:00:00.000Z",
    publishIntervalMs: 30_000,
    checks: [{ name: "nats", status: "ok", latencyMs: 1 }],
    info: {
      version: "1.2.3",
      info: { build: "abc123" },
    },
  });

  assertEquals(heartbeat.service.name, "activity");
  assertEquals(heartbeat.service.instanceId, "instance-1");
  assertEquals(heartbeat.service.version, "1.2.3");
  assertEquals(heartbeat.status, "healthy");
});

Deno.test("ServiceHealth aggregates registered checks and info", async () => {
  const health = new ServiceHealth({
    serviceName: "activity",
    contractId: "trellis.audit@v1",
    contractDigest: "digest",
    publishIntervalMs: 30_000,
  });

  health.setInfo({
    version: "1.2.3",
    info: { build: "abc123" },
  });
  health.add("nats", () => ({ status: "ok" }));
  health.add("db", () => ({
    status: "failed",
    summary: "connection timeout",
  }));

  const checks = await health.checks();
  const heartbeat = await health.heartbeat();
  const dbCheck = checks.find((check) => check.name === "db");

  assertEquals(heartbeat.status, "degraded");
  assertEquals(checks.length, 2);
  assertEquals(dbCheck?.info?.service, "activity");
  assertEquals(dbCheck?.info?.contractId, "trellis.audit@v1");
  assertEquals(dbCheck?.info?.contractDigest, "digest");
  assertEquals(heartbeat.service.contractId, "trellis.audit@v1");
  assertEquals(heartbeat.service.version, "1.2.3");
});

Deno.test("health heartbeat wire schema accepts additive fields", () => {
  const checks: HealthCheckResult[] = [{
    name: "nats",
    status: "ok",
    latencyMs: 1,
  }];

  assert(Value.Check(HealthHeartbeatSchema, {
    service: {
      name: "activity",
      kind: "service",
      instanceId: "instance-1",
      contractId: "trellis.audit@v1",
      contractDigest: "digest",
      startedAt: "2026-01-01T00:00:00.000Z",
      publishIntervalMs: 30_000,
      runtime: "deno",
      region: "primary",
    },
    status: "healthy",
    checks,
    requestId: "req_123",
  }));
});
