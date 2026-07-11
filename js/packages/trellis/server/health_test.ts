import { assert, assertEquals } from "@std/assert";
import Value from "typebox/value";
import {
  createHealthHeartbeatSample,
  runAllServiceHealthChecks,
  runServiceHealthCheck,
  ServiceHealth,
} from "./health.ts";
import { HealthHeartbeatSampleSchema } from "../sdk/_generated/health/schemas.ts";
import { healthHeartbeatSubject } from "../health_transport.ts";

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

Deno.test("createHealthHeartbeatSample includes participant metadata", () => {
  const heartbeat = createHealthHeartbeatSample({
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

  assertEquals(heartbeat.participant.name, "activity");
  assertEquals(heartbeat.participant.instanceId, "instance-1");
  assertEquals(heartbeat.participant.version, "1.2.3");
  assertEquals(heartbeat.reportedStatus, "healthy");
});

Deno.test("healthHeartbeatSubject encodes authoritative identity tokens", () => {
  assertEquals(
    healthHeartbeatSubject({
      sessionKey: "session_key",
      participantKind: "service",
      contractId: "trellis.jobs@v1",
      contractDigest: "digest-alpha",
      deploymentId: "jobs.default",
      instanceId: "rust-1",
    }),
    "health.v1.heartbeat.service.dHJlbGxpcy5qb2JzQHYx.ZGlnZXN0LWFscGhh.am9icy5kZWZhdWx0.cnVzdC0x.session_key",
  );
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
  const heartbeat = await health.sample();
  const dbCheck = checks.find((check) => check.name === "db");

  assertEquals(heartbeat.reportedStatus, "degraded");
  assertEquals(checks.length, 2);
  assertEquals(dbCheck?.info?.service, "activity");
  assertEquals(dbCheck?.info?.contractId, "trellis.audit@v1");
  assertEquals(dbCheck?.info?.contractDigest, "digest");
  assertEquals(heartbeat.participant.contractId, "trellis.audit@v1");
  assertEquals(heartbeat.participant.version, "1.2.3");
});

Deno.test("health sample wire schema accepts additive fields", () => {
  assert(Value.Check(HealthHeartbeatSampleSchema, {
    sample: {
      id: "01J00000000000000000000000",
      time: "2026-01-01T00:00:00.000Z",
      source: "activity",
    },
    participant: {
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
    reportedStatus: "healthy",
    checks: [{
      name: "nats",
      status: "ok",
      latencyMs: 1,
      region: "primary",
    }],
    requestId: "req_123",
  }));
});
