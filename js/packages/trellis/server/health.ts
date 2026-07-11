/**
 * Health check types and utility functions for Trellis services.
 *
 * This module provides a standardized way to implement health checks
 * for services, with support for individual check results and
 * aggregated health status (healthy, degraded, unhealthy).
 *
 * @module
 */

import type { JsonValue } from "../contracts.ts";
import type { HealthHeartbeatSample } from "../sdk/_generated/health/types.ts";
import { ulid } from "ulid";

type MaybePromise<T> = T | Promise<T>;

/**
 * Result of a single health check.
 */
export type HealthCheckResult = {
  /** Name of the health check */
  name: string;
  /** Status of the check: "ok" if passed, "failed" if not */
  status: "ok" | "failed";
  /** Error message if the check failed with an error */
  error?: string;
  /** Optional short human-readable summary */
  summary?: string;
  /** Optional structured metadata for the check */
  info?: Record<string, JsonValue>;
  /** Time in milliseconds the check took to execute */
  latencyMs: number;
};

export type ServiceHealthInfo = {
  version?: string;
  info?: Record<string, JsonValue>;
};

export type ServiceHealthCheck = {
  status: "ok" | "failed";
  summary?: string;
  info?: Record<string, JsonValue>;
  error?: string;
};

export type ServiceHealthCheckFn = () => MaybePromise<ServiceHealthCheck>;
export type ServiceHealthInfoFn = () => MaybePromise<
  ServiceHealthInfo | undefined
>;

function summarizeHealthStatus(
  results: readonly Pick<HealthCheckResult, "status">[],
): HealthHeartbeatSample["reportedStatus"] {
  const allOk = results.every((r) => r.status === "ok");
  const anyOk = results.some((r) => r.status === "ok");
  return allOk ? "healthy" : anyOk ? "degraded" : "unhealthy";
}

function summarizeHealthChecks(
  results: readonly HealthCheckResult[],
): string | undefined {
  const failedCount = results.filter((r) => r.status === "failed").length;
  if (failedCount === 0) {
    return undefined;
  }

  return `${failedCount} check${failedCount === 1 ? "" : "s"} failing`;
}

function annotateServiceHealthCheck(
  result: HealthCheckResult,
  metadata: { service: string; contractId: string; contractDigest: string },
): HealthCheckResult {
  if (result.status !== "failed") {
    return result;
  }

  return {
    ...result,
    info: {
      ...(result.info ?? {}),
      service: metadata.service,
      contractId: metadata.contractId,
      contractDigest: metadata.contractDigest,
    },
  };
}

function detectRuntime(): {
  runtime: HealthHeartbeatSample["participant"]["runtime"];
  runtimeVersion?: string;
} {
  const maybeDeno = Reflect.get(globalThis, "Deno") as
    | { version?: { deno?: string } }
    | undefined;
  if (maybeDeno?.version?.deno) {
    return { runtime: "deno", runtimeVersion: maybeDeno.version.deno };
  }

  const maybeProcess = Reflect.get(globalThis, "process") as
    | { version?: string }
    | undefined;
  if (typeof maybeProcess?.version === "string") {
    return { runtime: "node", runtimeVersion: maybeProcess.version };
  }

  return { runtime: "unknown" };
}

/** Runs one service health check and records latency/error details. */
export async function runServiceHealthCheck(
  name: string,
  check: ServiceHealthCheckFn,
): Promise<HealthCheckResult> {
  const start = performance.now();
  try {
    const result = await check();
    const latencyMs = performance.now() - start;
    return {
      name,
      status: result.status,
      error: result.error,
      summary: result.summary,
      info: result.info,
      latencyMs,
    };
  } catch (error) {
    const latencyMs = performance.now() - start;
    const message = error instanceof Error ? error.message : String(error);
    return {
      name,
      status: "failed",
      error: message,
      summary: message,
      info: {
        errorType: error instanceof Error ? error.name : typeof error,
      },
      latencyMs,
    };
  }
}

/** Runs service health checks and returns individual heartbeat check entries. */
export async function runAllServiceHealthChecks(
  checks: Record<string, ServiceHealthCheckFn>,
): Promise<HealthCheckResult[]> {
  return await Promise.all(
    Object.entries(checks).map(([name, fn]) => runServiceHealthCheck(name, fn)),
  );
}

/** Builds a private health transport sample from participant metadata and checks. */
export function createHealthHeartbeatSample(args: {
  serviceName: string;
  kind?: HealthHeartbeatSample["participant"]["kind"];
  instanceId: string;
  contractId: string;
  contractDigest: string;
  startedAt: string;
  publishIntervalMs: number;
  checks: HealthCheckResult[];
  info?: ServiceHealthInfo;
}): HealthHeartbeatSample {
  const runtime = detectRuntime();
  const summary = summarizeHealthChecks(args.checks);

  return {
    sample: {
      id: ulid(),
      time: new Date().toISOString(),
    },
    participant: {
      name: args.serviceName,
      kind: args.kind ?? "service",
      instanceId: args.instanceId,
      contractId: args.contractId,
      contractDigest: args.contractDigest,
      startedAt: args.startedAt,
      publishIntervalMs: args.publishIntervalMs,
      runtime: runtime.runtime,
      ...(runtime.runtimeVersion
        ? { runtimeVersion: runtime.runtimeVersion }
        : {}),
      ...(args.info?.version ? { version: args.info.version } : {}),
      ...(args.info?.info ? { info: args.info.info } : {}),
    },
    reportedStatus: summarizeHealthStatus(args.checks),
    ...(summary ? { summary } : {}),
    checks: args.checks,
  };
}

/** Mutable health sample state owned by a connected service or device. */
export class ServiceHealth {
  readonly serviceName: string;
  readonly kind: HealthHeartbeatSample["participant"]["kind"];
  readonly instanceId: string;
  readonly contractId: string;
  readonly contractDigest: string;
  readonly startedAt: string;
  readonly publishIntervalMs: number;

  #checks = new Map<string, ServiceHealthCheckFn>();
  #info?: ServiceHealthInfoFn;

  constructor(args: {
    serviceName: string;
    kind?: HealthHeartbeatSample["participant"]["kind"];
    instanceId?: string;
    contractId: string;
    contractDigest: string;
    publishIntervalMs: number;
  }) {
    this.serviceName = args.serviceName;
    this.kind = args.kind ?? "service";
    this.instanceId = args.instanceId ?? ulid();
    this.contractId = args.contractId;
    this.contractDigest = args.contractDigest;
    this.startedAt = new Date().toISOString();
    this.publishIntervalMs = args.publishIntervalMs;
  }

  setInfo(info: ServiceHealthInfo | ServiceHealthInfoFn): void {
    if (typeof info === "function") {
      this.#info = info;
      return;
    }

    this.#info = () => info;
  }

  add(name: string, check: ServiceHealthCheckFn): () => void {
    this.#checks.set(name, check);
    return () => {
      this.#checks.delete(name);
    };
  }

  async checks(): Promise<HealthCheckResult[]> {
    const results = await runAllServiceHealthChecks(
      Object.fromEntries(this.#checks),
    );
    return results.map((result) =>
      annotateServiceHealthCheck(result, {
        service: this.serviceName,
        contractId: this.contractId,
        contractDigest: this.contractDigest,
      })
    );
  }

  async sample(): Promise<HealthHeartbeatSample> {
    const checks = await this.checks();
    const info = this.#info ? await this.#info() : undefined;
    return createHealthHeartbeatSample({
      serviceName: this.serviceName,
      kind: this.kind,
      instanceId: this.instanceId,
      contractId: this.contractId,
      contractDigest: this.contractDigest,
      startedAt: this.startedAt,
      publishIntervalMs: this.publishIntervalMs,
      checks,
      info,
    });
  }
}
