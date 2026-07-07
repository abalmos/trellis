<script lang="ts">
  import { formatDate, compactDuration } from "../format";

  type Attempt = {
    try: number;
    startedAt: string;
    endedAt?: string;
    state?: string;
    error?: {
      fingerprint: string;
      message: string;
      stack?: string;
      type?: string;
    };
  };

  type Props = {
    attempts: Attempt[];
  };

  let { attempts }: Props = $props();

  const sortedAttempts = $derived(
    [...attempts].sort((a, b) => a.try - b.try)
  );

  const maxDuration = $derived(() => {
    if (sortedAttempts.length === 0) return 1;
    let max = 0;
    for (const attempt of sortedAttempts) {
      const start = new Date(attempt.startedAt).getTime();
      const end = attempt.endedAt ? new Date(attempt.endedAt).getTime() : Date.now();
      const duration = Math.max(0, end - start);
      if (duration > max) max = duration;
    }
    return max || 1;
  });

  function attemptDuration(attempt: Attempt): number {
    const start = new Date(attempt.startedAt).getTime();
    const end = attempt.endedAt ? new Date(attempt.endedAt).getTime() : Date.now();
    return Math.max(0, end - start);
  }

  function attemptBarColor(state?: string): string {
    switch (state) {
      case "completed": return "bg-success";
      case "failed": return "bg-error";
      case "active": return "bg-info";
      case "pending": return "bg-base-300";
      case "retry": return "bg-warning";
      case "cancelled": return "bg-neutral";
      default: return "bg-base-300";
    }
  }

  function attemptBarWidth(attempt: Attempt): string {
    const duration = attemptDuration(attempt);
    const max = maxDuration();
    const pct = max > 0 ? (duration / max) * 100 : 0;
    return `${Math.max(2, Math.min(pct, 100)).toFixed(1)}%`;
  }

  function attemptLabel(attempt: Attempt): string {
    return compactDuration(attemptDuration(attempt));
  }
</script>

<div class="job-attempt-timeline">
  {#if sortedAttempts.length === 0}
    <p class="text-sm text-base-content/60">No attempts recorded.</p>
  {:else}
    <div class="timeline-bars">
      {#each sortedAttempts as attempt (attempt.try)}
        <div class="attempt-bar-wrapper">
          <div class="attempt-bar-track">
            <div
              class={["attempt-bar", attemptBarColor(attempt.state)]}
              style="width: {attemptBarWidth(attempt)}"
              title="Attempt {attempt.try}: {attempt.state ?? 'unknown'} — {attemptLabel(attempt)}"
            ></div>
          </div>
          <div class="attempt-meta">
            <span class="attempt-number">{attempt.try}</span>
            <span class="attempt-label">{attemptLabel(attempt)}</span>
            {#if attempt.error}
              <span class="attempt-error-mark" title="{attempt.error.message}">✕</span>
            {:else if attempt.state === "completed"}
              <span class="attempt-success-mark">✓</span>
            {/if}
          </div>
        </div>
      {/each}
    </div>
    <div class="timeline-legend">
      <span class="legend-dot bg-success"></span> Completed
      <span class="legend-dot bg-info"></span> Active
      <span class="legend-dot bg-warning"></span> Retry
      <span class="legend-dot bg-error"></span> Failed
      <span class="legend-dot bg-base-300"></span> Pending
    </div>
  {/if}
</div>

<style>
  .job-attempt-timeline {
    display: grid;
    gap: 0.75rem;
  }

  .timeline-bars {
    display: flex;
    align-items: flex-end;
    gap: 0.5rem;
    padding: 0.5rem 0;
  }

  .attempt-bar-wrapper {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
    flex: 1;
    min-width: 0;
  }

  .attempt-bar-track {
    width: 100%;
    height: 2rem;
    background: color-mix(in oklab, var(--color-base-300) 30%, transparent);
    border-radius: 0.25rem;
    overflow: hidden;
    display: flex;
    align-items: flex-end;
  }

  .attempt-bar {
    height: 100%;
    border-radius: 0.25rem;
    transition: width 300ms ease-out;
    min-width: 2px;
  }

  .attempt-meta {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.1rem;
    font-size: 0.72rem;
    line-height: 1.2;
  }

  .attempt-number {
    font-weight: 700;
    color: color-mix(in oklab, var(--color-base-content) 70%, transparent);
  }

  .attempt-label {
    color: color-mix(in oklab, var(--color-base-content) 50%, transparent);
    font-variant-numeric: tabular-nums;
  }

  .attempt-error-mark {
    color: var(--color-error);
    font-weight: 700;
  }

  .attempt-success-mark {
    color: var(--color-success);
    font-weight: 700;
  }

  .timeline-legend {
    display: flex;
    flex-wrap: wrap;
    gap: 0.75rem;
    font-size: 0.72rem;
    color: color-mix(in oklab, var(--color-base-content) 60%, transparent);
  }

  .legend-dot {
    display: inline-block;
    width: 0.6rem;
    height: 0.6rem;
    border-radius: 0.15rem;
    vertical-align: middle;
    margin-right: 0.25rem;
  }
</style>
