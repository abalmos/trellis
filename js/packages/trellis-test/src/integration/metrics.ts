export const TRELLIS_TEST_METRICS_ENV = "TRELLIS_TEST_METRICS_PATH";

export type TrellisTestProcessKind =
  | "nats"
  | "trellis"
  | "jobs"
  | "eventlog";

/** Appends one process-start event to the active integration metrics stream. */
export async function recordTrellisTestProcessStart(
  process: TrellisTestProcessKind,
  detail?: string,
): Promise<void> {
  const path = Deno.env.get(TRELLIS_TEST_METRICS_ENV);
  if (path === undefined) return;
  await Deno.writeTextFile(
    path,
    `${
      JSON.stringify({
        event: "process-start",
        process,
        detail,
        pid: Deno.pid,
        at: new Date().toISOString(),
      })
    }\n`,
    { append: true, create: true },
  );
}

/** Appends one duration measurement to the active integration metrics stream. */
export async function recordTrellisTestDuration(
  metric: string,
  durationMs: number,
  attributes?: Readonly<Record<string, unknown>>,
): Promise<void> {
  const path = Deno.env.get(TRELLIS_TEST_METRICS_ENV);
  if (path === undefined) return;
  await Deno.writeTextFile(
    path,
    `${
      JSON.stringify({
        event: "duration",
        metric,
        durationMs,
        attributes,
        pid: Deno.pid,
        at: new Date().toISOString(),
      })
    }\n`,
    { append: true, create: true },
  );
}

/** Reads the active integration metrics stream before its workdir is removed. */
export async function readTrellisTestMetrics(
  path: string,
): Promise<readonly Record<string, unknown>[]> {
  try {
    return (await Deno.readTextFile(path)).trim().split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line) as Record<string, unknown>);
  } catch (error) {
    if (error instanceof Deno.errors.NotFound) return [];
    throw error;
  }
}

/** Counts process starts by kind for release-gate reporting. */
export function summarizeTrellisTestProcessStarts(
  events: readonly Record<string, unknown>[],
): Readonly<Record<TrellisTestProcessKind, number>> {
  const counts: Record<TrellisTestProcessKind, number> = {
    nats: 0,
    trellis: 0,
    jobs: 0,
    eventlog: 0,
  };
  for (const event of events) {
    if (
      event.event === "process-start" &&
      typeof event.process === "string" &&
      event.process in counts
    ) {
      counts[event.process as TrellisTestProcessKind] += 1;
    }
  }
  return counts;
}

/** Returns the slowest recorded integration phases. */
export function summarizeTrellisTestDurations(
  events: readonly Record<string, unknown>[],
  limit = 20,
): readonly Record<string, unknown>[] {
  const groups = new Map<string, {
    metric: string;
    attributes: unknown;
    count: number;
    totalMs: number;
    maxMs: number;
  }>();
  for (const event of events) {
    if (
      event.event !== "duration" || typeof event.metric !== "string" ||
      typeof event.durationMs !== "number"
    ) continue;
    const key = JSON.stringify([event.metric, event.attributes]);
    const group = groups.get(key) ?? {
      metric: event.metric,
      attributes: event.attributes,
      count: 0,
      totalMs: 0,
      maxMs: 0,
    };
    group.count += 1;
    group.totalMs += event.durationMs;
    group.maxMs = Math.max(group.maxMs, event.durationMs);
    groups.set(key, group);
  }
  return [...groups.values()].map((group) => ({
    metric: group.metric,
    attributes: group.attributes,
    count: group.count,
    averageMs: group.totalMs / group.count,
    maxMs: group.maxMs,
  })).sort((left, right) => right.maxMs - left.maxMs).slice(0, limit);
}
