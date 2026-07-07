<script lang="ts">
  import { compactDuration } from "../format";

  type Latency = {
    count: number;
    p50Ms?: number;
    p95Ms?: number;
    maxMs?: number;
  };

  type SummaryGroup = {
    key: string;
    label: string;
    total: number;
    byState: { [k: string]: number };
    running?: number;
    queued?: number;
    failed?: number;
    dead?: number;
    slow?: number;
    failureRate?: number;
    runtime: Latency;
    queueWait: Latency;
    oldestCreatedAt?: string;
    latestUpdatedAt?: string;
  };

  type Bucket = {
    start: string;
    end: string;
    groups: Array<{
      key: string;
      label: string;
      submitted: number;
      started: number;
      completed: number;
      failed: number;
      retried: number;
      dead: number;
      cancelled: number;
      dismissed: number;
      runtime: Latency;
      queueWait: Latency;
    }>;
  };

  type Props = {
    summary: SummaryGroup[];
    buckets: Bucket[];
    selectedKey?: string | null;
    onSelect?: (key: string | null) => void;
  };

  let { summary, buckets, selectedKey = null, onSelect }: Props = $props();

  const sparklineByKey = $derived.by(() => {
    const map = new Map<string, number[]>();
    for (const bucket of buckets) {
      for (const group of bucket.groups) {
        if (!map.has(group.key)) map.set(group.key, []);
        const failures = group.failed + group.dead + group.retried;
        const list = map.get(group.key);
        if (list) list.push(failures);
      }
    }
    return map;
  });

  const sortedSummary = $derived.by(() =>
    [...summary].sort((left, right) => {
      const leftPressure = (left.failed ?? 0) + (left.dead ?? 0) * 2;
      const rightPressure = (right.failed ?? 0) + (right.dead ?? 0) * 2;
      if (rightPressure !== leftPressure) return rightPressure - leftPressure;
      return right.total - left.total;
    }),
  );

  function severity(failed: number, total: number): string {
    if (total === 0) return "badge-ghost";
    const rate = failed / total;
    if (rate >= 0.25) return "badge-error";
    if (rate >= 0.1) return "badge-warning";
    return "badge-success";
  }

  function formatMs(value: number | undefined): string {
    if (value === undefined) return "—";
    return compactDuration(value);
  }

  function handleSelect(key: string | null) {
    if (onSelect) onSelect(key);
  }

  function handleKey(event: KeyboardEvent, key: string) {
    if (event.key !== "Enter" && event.key !== " ") return;
    event.preventDefault();
    handleSelect(key);
  }
</script>

<div class="jobs-matrix">
  <header class="jobs-matrix-header">
    <h3>Job-type health</h3>
    <p class="jobs-matrix-help">One row per job type. Click a row to scope the charts and filter the jobs table to that type.</p>
  </header>
  {#if sortedSummary.length === 0}
    <p class="jobs-matrix-empty">No job types reported for this window.</p>
  {:else}
    <div class="jobs-matrix-table" role="table" aria-label="Job type health">
      <div class="jobs-matrix-row jobs-matrix-row-head" role="row">
        <span role="columnheader">Job type</span>
        <span role="columnheader" class="ta-right">Backlog</span>
        <span role="columnheader" class="ta-right">Running</span>
        <span role="columnheader" class="ta-right">Failed</span>
        <span role="columnheader" class="ta-right">DLQ</span>
        <span role="columnheader" class="ta-right">Queue p95</span>
        <span role="columnheader" class="ta-right">Runtime p95</span>
        <span role="columnheader">Trend</span>
      </div>
      {#each sortedSummary as group (group.key)}
        {@const isSelected = selectedKey === group.key}
        {@const spark = sparklineByKey.get(group.key) ?? []}
        {@const failedTotal = (group.failed ?? 0) + (group.dead ?? 0)}
        {@const sparkMax = Math.max(...spark, 1)}
        <div
          class={['jobs-matrix-row', isSelected && 'selected']}
          role="row"
          tabindex="0"
          onclick={() => handleSelect(isSelected ? null : group.key)}
          onkeydown={(event) => handleKey(event, group.key)}
        >
          <span role="cell" class="jobs-matrix-label">
            <span class="jobs-matrix-key">{group.label}</span>
            <span class={['badge', 'badge-sm', severity(failedTotal, group.total)]}>
              {group.failureRate === undefined ? "—" : `${(group.failureRate * 100).toFixed(1)}%`}
            </span>
          </span>
          <span role="cell" class="ta-right tabular-nums">{group.queued ?? 0}</span>
          <span role="cell" class="ta-right tabular-nums">{group.running ?? 0}</span>
          <span role="cell" class="ta-right tabular-nums">{group.failed ?? 0}</span>
          <span role="cell" class="ta-right tabular-nums">{group.dead ?? 0}</span>
          <span role="cell" class="ta-right tabular-nums">{formatMs(group.queueWait.p95Ms)}</span>
          <span role="cell" class="ta-right tabular-nums">{formatMs(group.runtime.p95Ms)}</span>
          <span role="cell" class="jobs-matrix-sparkline">
            {#if spark.length === 0}
              <span class="jobs-matrix-sparkline-empty">no data</span>
            {:else}
              <svg viewBox="0 0 {spark.length * 4} 20" preserveAspectRatio="none" aria-hidden="true">
                <polyline
                  points={spark
                    .map((value, index) => `${index * 4},${20 - (value / sparkMax) * 18 - 1}`)
                    .join(" ")}
                  fill="none"
                  stroke="oklch(0.62 0.18 25)"
                  stroke-width="1.5"
                  stroke-linejoin="round"
                  stroke-linecap="round"
                />
              </svg>
            {/if}
          </span>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .jobs-matrix {
    background: color-mix(in oklab, var(--color-base-100) 92%, var(--color-base-200));
    border: 1px solid color-mix(in oklab, var(--color-base-300) 82%, transparent);
    border-radius: var(--radius-box, 1rem);
    display: grid;
    gap: 0.5rem;
    padding: 0.9rem 1rem;
  }

  .jobs-matrix-header {
    display: grid;
    gap: 0.1rem;
  }

  .jobs-matrix-header h3 {
    font-size: 0.78rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
  }

  .jobs-matrix-help {
    font-size: 0.72rem;
    color: color-mix(in oklab, var(--color-base-content) 55%, transparent);
  }

  .jobs-matrix-empty {
    font-size: 0.78rem;
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-style: italic;
  }

  .jobs-matrix-table {
    display: grid;
    gap: 0.2rem;
  }

  .jobs-matrix-row {
    align-items: center;
    border-radius: var(--radius-field, 0.5rem);
    cursor: pointer;
    display: grid;
    gap: 0.5rem;
    grid-template-columns: minmax(0, 2.4fr) repeat(6, minmax(0, 0.9fr)) minmax(0, 1.4fr);
    padding: 0.45rem 0.55rem;
    transition: background-color 120ms ease-out;
  }

  .jobs-matrix-row:hover {
    background: color-mix(in oklab, var(--color-base-200) 60%, transparent);
  }

  .jobs-matrix-row.selected {
    background: color-mix(in oklab, var(--color-accent) 12%, transparent);
    outline: 1px solid color-mix(in oklab, var(--color-accent) 46%, transparent);
    outline-offset: -1px;
  }

  .jobs-matrix-row:focus-visible {
    box-shadow: var(--trellis-focus-ring);
    outline: none;
  }

  .jobs-matrix-row-head {
    background: color-mix(in oklab, var(--color-base-200) 70%, transparent);
    cursor: default;
    font-size: 0.66rem;
    font-weight: 700;
    letter-spacing: 0.05em;
    text-transform: uppercase;
    color: color-mix(in oklab, var(--color-base-content) 55%, transparent);
  }

  .jobs-matrix-row-head:hover {
    background: color-mix(in oklab, var(--color-base-200) 70%, transparent);
  }

  .jobs-matrix-label {
    align-items: center;
    display: flex;
    gap: 0.5rem;
    min-width: 0;
  }

  .jobs-matrix-key {
    font-family: ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, "Liberation Mono", "Courier New", monospace;
    font-size: 0.78rem;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .ta-right {
    text-align: right;
  }

  .jobs-matrix-sparkline {
    align-items: center;
    display: flex;
    height: 1.4rem;
  }

  .jobs-matrix-sparkline svg {
    display: block;
    height: 100%;
    width: 100%;
  }

  .jobs-matrix-sparkline-empty {
    color: color-mix(in oklab, var(--color-base-content) 40%, transparent);
    font-size: 0.66rem;
    font-style: italic;
  }

  @media (max-width: 900px) {
    .jobs-matrix-row {
      grid-template-columns: minmax(0, 1.6fr) repeat(3, minmax(0, 0.7fr)) minmax(0, 1.2fr);
      grid-template-areas:
        "label label sparkline sparkline"
        "backlog running failed dlq"
        "qp95 rp95 qp95 rp95";
      row-gap: 0.25rem;
    }

    .jobs-matrix-row-head {
      display: none;
    }
  }
</style>
