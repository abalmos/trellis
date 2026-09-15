import {
  recordTrellisDuration,
  type TrellisDurationMetricAttributes,
  type TrellisDurationMetricName,
} from "./metrics.ts";

/** Exactly-once duration observation for one unit of work. */
export interface TelemetryObservation {
  /** Records the observation with one bounded outcome value. */
  finish(outcome: string, extra?: TrellisDurationMetricAttributes): void;
  /** Records the configured cancelled/interrupted outcome once. */
  cancel(): void;
}

/**
 * Starts one duration observation.
 *
 * Callers use `try/finally` so `?`/throw paths cannot silently skip the
 * observation, matching the Rust observation guard boundary.
 */
export function startObservation(
  metric: TrellisDurationMetricName,
  attributes: TrellisDurationMetricAttributes,
  dropOutcome = "cancelled",
): TelemetryObservation {
  const startedAt = performance.now();
  let finished = false;
  const record = (
    outcome: string,
    extra?: TrellisDurationMetricAttributes,
  ): void => {
    if (finished) return;
    finished = true;
    recordTrellisDuration(
      metric,
      performance.now() - startedAt,
      { ...attributes, ...extra, outcome },
    );
  };
  return {
    finish: (outcome, extra) => record(outcome, extra),
    cancel: () => record(dropOutcome),
  };
}

/**
 * Runs one async unit under a duration observation with the same boundary.
 *
 * The original value or exception is always preserved.
 */
export async function withObservation<T>(
  metric: TrellisDurationMetricName,
  attributes: TrellisDurationMetricAttributes,
  run: () => Promise<T>,
  options?: {
    /** Outcome derivation for a successful result; defaults to `ok`. */
    outcomeOf?: (value: T) => string;
    /** Outcome recorded when the unit throws; defaults to `error`. */
    errorOutcome?: string;
    /** Outcome recorded when an abort signal cancels the unit. */
    signal?: AbortSignal;
  },
): Promise<T> {
  const observation = startObservation(metric, attributes);
  try {
    const value = await run();
    if (options?.signal?.aborted) {
      observation.cancel();
    } else {
      observation.finish(options?.outcomeOf?.(value) ?? "ok");
    }
    return value;
  } catch (error) {
    observation.finish(options?.errorOutcome ?? "error");
    throw error;
  }
}
