<script lang="ts">
  import { compactDuration } from "../format";
  import { Line, LineY, Plot } from "svelteplot";

  type Latency = {
    count: number;
    p50Ms?: number;
    p95Ms?: number;
    maxMs?: number;
  };

  type BucketGroup = {
    key: string;
    label: string;
    failed: number;
    dead: number;
    runtime: Latency;
    queueWait: Latency;
  };

  type Bucket = {
    start: string;
    end: string;
    groups: BucketGroup[];
  };

  type Props = {
    buckets: Bucket[];
    selectedKey: string;
  };

  let { buckets, selectedKey }: Props = $props();

  const failures = $derived.by(() =>
    buckets.map((bucket) => {
      const group = bucket.groups.find((entry) => entry.key === selectedKey);
      return {
        time: new Date(bucket.start).getTime(),
        failed: group?.failed ?? 0,
        dead: group?.dead ?? 0,
      };
    }),
  );

  const latency = $derived.by(() =>
    buckets.map((bucket) => {
      const group = bucket.groups.find((entry) => entry.key === selectedKey);
      return {
        time: new Date(bucket.start).getTime(),
        runtimeP50: group?.runtime.p50Ms ?? 0,
        runtimeP95: group?.runtime.p95Ms ?? 0,
        queueWaitP95: group?.queueWait.p95Ms ?? 0,
      };
    }),
  );

  const hasData = $derived(
    failures.some((row) => row.failed + row.dead > 0) ||
      latency.some((row) => row.runtimeP95 + row.queueWaitP95 > 0),
  );

  const latestRuntime = $derived(latency.at(-1)?.runtimeP95 ?? 0);
  const latestQueue = $derived(latency.at(-1)?.queueWaitP95 ?? 0);

  function formatMs(value: number | undefined): string {
    if (!value || value <= 0) return "—";
    return compactDuration(value);
  }
</script>

<div class="jobs-scoped">
  <header class="jobs-scoped-header">
    <h3>Job type · <span class="jobs-scoped-key">{selectedKey}</span></h3>
    <p class="jobs-scoped-help">Failure pressure and runtime/queue latency for the selected job type over the active window.</p>
  </header>

  <div class="jobs-scoped-stats">
    <div><span>Runtime p95</span><strong>{formatMs(latestRuntime)}</strong></div>
    <div><span>Queue wait p95</span><strong>{formatMs(latestQueue)}</strong></div>
  </div>

  {#if !hasData}
    <p class="jobs-scoped-empty">No activity recorded for this job type in the selected window.</p>
  {:else}
    <div class="jobs-scoped-plot">
      <Plot
        marginTop={6}
        marginRight={6}
        marginBottom={28}
        marginLeft={48}
        x={{ grid: true, type: "utc" }}
        y={{ grid: true, label: "failures / bucket" }}
        height={150}
      >
        <LineY {...({ x: "time" } as Record<string, unknown>)} data={failures} y="failed" stroke="oklch(0.62 0.18 25)" strokeWidth={1.6} />
        <LineY {...({ x: "time" } as Record<string, unknown>)} data={failures} y="dead" stroke="oklch(0.55 0.22 320)" strokeWidth={1.2} />
      </Plot>
      <div class="jobs-scoped-legend">
        <span><span class="dot" style="background: oklch(0.62 0.18 25)"></span> failed</span>
        <span><span class="dot" style="background: oklch(0.55 0.22 320)"></span> dead</span>
      </div>
    </div>

    <div class="jobs-scoped-plot">
      <Plot
        marginTop={6}
        marginRight={6}
        marginBottom={28}
        marginLeft={48}
        x={{ grid: true, type: "utc" }}
        y={{ grid: true, label: "ms" }}
        height={150}
      >
        <Line {...({ x: "time" } as Record<string, unknown>)} data={latency} y="queueWaitP95" stroke="oklch(0.55 0.18 60)" strokeWidth={1.6} />
        <Line {...({ x: "time" } as Record<string, unknown>)} data={latency} y="runtimeP50" stroke="oklch(0.7 0.13 145)" strokeWidth={1.2} strokeDasharray="3 3" />
        <Line {...({ x: "time" } as Record<string, unknown>)} data={latency} y="runtimeP95" stroke="oklch(0.55 0.18 250)" strokeWidth={1.6} />
      </Plot>
      <div class="jobs-scoped-legend">
        <span><span class="dot" style="background: oklch(0.55 0.18 60)"></span> queue wait p95</span>
        <span><span class="dot" style="background: oklch(0.55 0.18 250)"></span> runtime p95</span>
        <span><span class="dot" style="background: oklch(0.7 0.13 145)"></span> runtime p50</span>
      </div>
    </div>
  {/if}
</div>

<style>
  .jobs-scoped {
    background: color-mix(in oklab, var(--color-base-100) 92%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 82%, transparent);
    border-radius: var(--radius-box, 1rem);
    display: grid;
    gap: 0.75rem;
    padding: 0.9rem 1rem;
  }

  .jobs-scoped-header {
    display: grid;
    gap: 0.1rem;
  }

  .jobs-scoped-header h3 {
    font-size: 0.78rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
  }

  .jobs-scoped-key {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    color: color-mix(in oklab, var(--color-base-content) 90%, transparent);
  }

  .jobs-scoped-help {
    font-size: 0.72rem;
    color: color-mix(in oklab, var(--color-base-content) 55%, transparent);
  }

  .jobs-scoped-stats {
    display: grid;
    gap: 0.5rem;
    grid-template-columns: repeat(2, minmax(0, 1fr));
  }

  .jobs-scoped-stats > div {
    background: color-mix(in oklab, var(--color-base-100) 78%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 78%, transparent);
    border-radius: var(--radius-field, 0.5rem);
    display: grid;
    gap: 0.1rem;
    padding: 0.5rem 0.65rem;
  }

  .jobs-scoped-stats span {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-size: 0.66rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .jobs-scoped-stats strong {
    font-size: 0.95rem;
    font-variant-numeric: tabular-nums;
  }

  .jobs-scoped-plot :global(svg) {
    display: block;
    width: 100%;
  }

  .jobs-scoped-legend {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    font-size: 0.7rem;
    color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  }

  .jobs-scoped-legend .dot {
    display: inline-block;
    height: 0.45rem;
    margin-right: 0.25rem;
    vertical-align: middle;
    width: 0.45rem;
    border-radius: 0.1rem;
  }

  .jobs-scoped-empty {
    font-size: 0.78rem;
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-style: italic;
  }
</style>
